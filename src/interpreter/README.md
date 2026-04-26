# interpreter

The execution engine. Runs in two passes over the AST: resolve, then evaluate.

## Files

- `interpreter.rs` — tree-walk evaluator
- `resolver.rs` — static variable resolution pass
- `env.rs` — runtime scope / environment chain
- `value.rs` — runtime value types
- `function.rs` — callable trait + Lox function implementation
- `class.rs` — class definition and construction
- `instance.rs` — object instances and property access

---

## Two-pass architecture

```
parse → [AST] → resolver → [locals map] → interpreter
```

The resolver runs first and produces a `HashMap<*const Expr, (depth, index)>` that maps every local variable reference to its exact scope depth and slot index. The interpreter then uses this map at runtime for O(1) variable lookup — no string hashing in the hot path for local variables.

Global variables still use string-keyed `HashMap` lookup because they are not resolved (the resolver skips the global scope).

---

## resolver.rs

### What it resolves

The resolver walks every `Expr` and `Statement` once before execution. For each `Expr::Variable`, `Expr::Assignment`, `Expr::This`, and `Expr::Super` it calls `resolve_local`, which walks the scope stack from innermost outward looking for the name. When found it records `(depth, slot_index)` keyed by the raw pointer of the `Expr` node.

### Scope stack

Scopes are a `Vec<Vec<(String, bool)>>` — a stack of ordered lists. Each entry is `(name, is_ready)`. The `bool` starts as `false` (declared but not yet defined) and flips to `true` after the initializer is resolved. This catches `var x = x;` — reading a variable while `is_ready == false` is a static error.

Using an ordered `Vec` rather than a `HashMap` per scope is deliberate: the slot index assigned by the resolver must match the position in the environment's `Vec<Value>`. The resolver's `declare` pushes to the end; the index is `scope.len() - 1`. The environment's `define` also pushes to the end. The two stay in sync as long as both always append.

### Static errors caught

- `return` statement outside any function
- `return value` inside `init`
- `this` outside a class
- `super` outside a class or in a class with no superclass
- A class inheriting from itself
- Variable used in its own initializer (`var x = x`)
- Variable redeclared in the same local scope

### `FunctionType` and `ClassType` tracking

The resolver keeps `current_function: FunctionType` and `current_class: ClassType` on its own stack, saved/restored around each nested function or class body. This gives context for the static error checks above without needing to pass extra parameters through the recursion.

---

## env.rs

### Two storage modes

```rust
struct Environment {
    map: HashMap<String, Value>,  // globals only
    parent: Option<EnvRef>,
    values: Vec<Value>,           // locals only
}
```

When `parent.is_none()` (global scope), `define` writes to `map`. For any local scope it pushes to `values`. This two-tier design means:

- Global lookups: `HashMap` with string key (acceptable — globals are infrequent in hot loops)
- Local lookups: `values[index]` — direct array access, no hashing

### `get_at` / `assign_at`

```rust
fn ancestor(env: EnvRef, distance: usize) -> EnvRef { ... }
fn get_at(env: EnvRef, distance: usize, index: usize) -> Value { ... }
fn assign_at(env: EnvRef, distance: usize, index: usize, value: Value) { ... }
```

`ancestor` walks `distance` steps up the parent chain. `get_at` and `assign_at` then directly index into `ancestor.values[index]`. This is always correct because the resolver computed `distance` and `index` against the same scope structure that the interpreter constructs at runtime.

### `EnvRef = Rc<RefCell<Environment>>`

Environments are shared: closures hold an `Rc` to their defining scope, and multiple closures can share the same scope frame. `RefCell` provides interior mutability so assignments (`assign_at`) can mutate through a shared reference. The combination is safe in single-threaded Lox — there is no concurrent access.

---

## value.rs

```rust
pub enum Value {
    Number(f64),
    Boolean(bool),
    String(String),
    Nil,
    Callable(Rc<dyn LoxCallable>),
    Class(Rc<LoxClass>),
    Instance(Rc<RefCell<LoxInstance>>),
    Array(Rc<RefCell<Vec<Value>>>),
}
```

### Why `Rc<dyn LoxCallable>` for callables?

Functions are first-class values — they can be stored in variables, passed as arguments, returned from other functions, and stored in instance fields. `Rc` gives cheap cloning (just an atomic increment) without deep-copying the function body. `dyn LoxCallable` lets native functions and Lox functions share the same `Value` variant.

### `Instance` and `Array` use `Rc<RefCell<_>>`

Both need shared mutable state:
- The same instance may be referenced by multiple variables (`var a = obj; var b = obj; b.x = 1;` — `a.x` must also be `1`).
- Assigning to `arr.set(0, v)` must mutate the underlying vec even when the array is held via an immutable `Value`.

`Rc` provides shared ownership; `RefCell` provides runtime-checked mutation through a shared reference.

### `Class` is just `Rc<LoxClass>`

Class *definitions* are immutable after creation — methods don't change. `Rc` without `RefCell` is sufficient.

### Truthiness

```rust
Nil       → false
Boolean(b)→ b
Number(n) → n != 0.0
String(s) → !s.is_empty()
_         → true
```

Zero, empty string, and nil are falsy. Everything else — including class definitions, instances, and arrays — is truthy.

### Equality

`==` / `!=` use value equality, not reference equality. Two separate instances are never equal even if they have identical fields. This matches standard Lox semantics.

---

## function.rs

### `LoxCallable` trait

```rust
pub trait LoxCallable: Debug {
    fn arity(&self) -> usize;
    fn call(&self, interpreter: &mut Interpreter, args: Vec<Value>) -> Result<Value, Error>;
    fn is_variadic(&self) -> bool { false }
}
```

All callable things — Lox functions, native functions, class constructors — implement this trait. `is_variadic` defaults to `false`; native functions like `array(...)` override it to opt out of arity checking.

### `LoxFunction`

```rust
pub struct LoxFunction {
    params: Vec<Token>,
    body: Rc<Box<Statement>>,  // shared, never cloned
    closure: EnvRef,           // captured at definition time
    function_type: FunctionType,
}
```

`body` is `Rc<Box<Statement>>` — the AST node is shared with the class definition that owns it. `clone()` on a `LoxFunction` only bumps the reference count, not the AST.

`closure` is the environment *at the time the function was defined*, not at call time. This is what makes closures work: every call creates a new `Environment` with `closure` as the parent, so the function always sees the variables that were in scope when it was written.

### `bind(instance)`

When a method is retrieved from an instance, `bind` creates a fresh `Environment` parented to the function's closure, defines `this` in it, and returns a new `LoxFunction` using that environment as its closure. The original function is unchanged — binding produces a new value each time.

This is why `var f = obj.method; f();` works: `f` carries `this` baked into its closure.

### `init` always returns `this`

`init` is special: it should always return the instance, even if the body contains a bare `return`. The interpreter enforces this:

```rust
// in LoxFunction::call
if self.function_type == FunctionType::INIT {
    return Ok(Environment::get_at(self.closure.clone(), 0, 0));
}
```

`this` is always slot 0 of the closure (because `bind` defines it first). The resolver also rejects `return <value>` inside `init` as a static error.

### Return as control flow

`return` is implemented as `Err(Error::Return(val))`. The `Result` propagates up through `execute_stmt` until it reaches `LoxFunction::call`, which catches `Error::Return` and unwraps the value. This avoids needing a separate return-value register or unwinding mechanism.

---

## class.rs

### `LoxClass`

```rust
pub struct LoxClass {
    name: String,
    methods: HashMap<String, Rc<LoxFunction>>,
    superclass: Option<Rc<LoxClass>>,
}
```

`methods` maps method names to `Rc<LoxFunction>`. The `Rc` means the same function value is shared across all instances — methods are not copied per instance.

### `find_method` walks the superclass chain

```rust
pub fn find_method(&self, name: String) -> Option<Rc<LoxFunction>> {
    self.methods.get(&name).cloned().or_else(|| {
        self.superclass.as_ref()?.find_method(name)
    })
}
```

Method resolution is done at access time, not at class creation time. The chain is walked on every method lookup. This is simple and correct for Lox's single-inheritance model.

### Construction

Calling a class as a function (`Dog("Rex")`) invokes `LoxClass::call`:
1. Allocate a new `LoxInstance` with empty fields.
2. Look up `init` via `find_method` (which checks the superclass chain).
3. If found, `bind` it to the fresh instance and call it with the provided arguments.
4. Return `Value::Instance(instance)`.

The arity of a class is the arity of its `init` method (or 0 if there is none).

### `super` environment

When a class with a superclass is defined, the interpreter wraps all its methods in an extra environment frame containing `"super"`:

```
[super = <SuperClass>] → [outer closure env]
```

Later, when `bind` is called to create a method, `"this"` is injected one level above:

```
[this = <instance>] → [super = <SuperClass>] → [outer closure env]
```

The resolver knows `super` is at `distance` and `this` is at `distance - 1`. Both are always at slot index 0 of their respective frames.

---

## instance.rs

### `LoxInstance`

```rust
pub struct LoxInstance {
    pub class: Rc<LoxClass>,
    pub fields: HashMap<String, Value>,
}
```

Fields are stored per-instance in a `HashMap`. Lox allows adding new fields to an instance at any time (`obj.newField = value`), so there is no fixed field schema — this matches the dynamic nature of the language.

### `get` — fields shadow methods

```rust
pub fn get(instance_rc: &Rc<RefCell<Self>>, name: &Token) -> Value {
    if instance_rc.borrow().fields.contains_key(key) {
        return instance_rc.borrow().fields[key].clone();
    }
    if let Some(method) = instance_rc.borrow().class.find_method(...) {
        return method.bind(instance_rc.clone());
    }
    runtime_error(...);
}
```

Fields take priority over methods. If you store a value in `obj.foo`, it shadows any method named `foo` on the class. This is standard Lox behaviour.

`get` takes `&Rc<RefCell<Self>>` rather than `&self` because `bind` needs an `Rc<RefCell<LoxInstance>>` to inject as `this`. Passing the `Rc` directly avoids cloning the instance just to get a new `Rc` to it.
