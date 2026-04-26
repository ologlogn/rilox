# native

Built-in functions injected into the global environment at interpreter startup. Each native function is a zero-size struct that implements `LoxCallable`.

## How natives are registered

In `Interpreter::new()`, `define_global` inserts every native as a `Value::Callable(Rc::new(...))` into the global `Environment`. From Lox code they look exactly like user-defined functions — there is no special call path.

```rust
globals.define("clock",     Value::Callable(Rc::new(ClockFn)));
globals.define("read_line", Value::Callable(Rc::new(ReadLineFn)));
globals.define("to_number", Value::Callable(Rc::new(ToNumberFn)));
globals.define("array",     Value::Callable(Rc::new(ArrayFn)));
globals.define("floor",     Value::Callable(Rc::new(MathFloorFn)));
```

---

## `LoxCallable` trait recap

```rust
pub trait LoxCallable: Debug {
    fn arity(&self) -> usize;
    fn call(&self, interpreter: &mut Interpreter, args: Vec<Value>) -> Result<Value, Error>;
    fn is_variadic(&self) -> bool { false }
}
```

Natives override `is_variadic` only when they accept a variable number of arguments (currently only `ArrayFn`). Fixed-arity natives rely on the interpreter's arity check before `call` is invoked.

---

## clock.rs

```rust
pub struct ClockFn;
```

Returns the number of seconds since the Unix epoch as `Value::Number(f64)`.

Uses `std::time::SystemTime::now().duration_since(UNIX_EPOCH).as_secs_f64()`. The `f64` gives sub-second precision while remaining a plain `Number` in Lox — no special time type needed.

Primary use case: benchmarking Lox programs (`var start = clock(); ...; print clock() - start;`).

---

## io.rs

```rust
pub struct ReadLineFn;
```

Flushes stdout before reading so any pending `print` output appears before the user types. Reads one line from stdin via `io::stdin().read_line(...)`, trims the trailing `\n`/`\r\n`, and returns `Value::String`.

Returns `Value::Nil` on EOF (0 bytes read) or IO error — lets Lox programs detect end-of-input with a nil check rather than throwing a runtime error.

---

## convert.rs

```rust
pub struct ToNumberFn;
```

Converts a single value to `Number`:

| Input | Output |
|-------|--------|
| `Number(n)` | identity |
| `String(s)` | `f64::parse(s.trim())`, or `Nil` on failure |
| `Boolean(true)` | `1.0` |
| `Boolean(false)` | `0.0` |
| anything else | `Nil` |

Returning `Nil` on unparseable strings (rather than a runtime error) lets callers do a nil-check to detect bad input without crashing the interpreter. Typical use: reading a number from stdin (`to_number(read_line())`).

---

## math.rs

```rust
pub struct MathFloorFn;
```

Wraps `f64::floor`. Returns `Error::RuntimeError` (not a process exit) if the argument is not a number — unlike most other runtime errors which call `runtime_error()` and `exit(70)` immediately. This means the error propagates as an `Err` through the call stack and is caught and reported at the call site.

---

## array.rs

Arrays are the most complex native, split into two structs:

### `ArrayFn` — constructor

```rust
pub struct ArrayFn;
// is_variadic → true
```

`array(1, 2, 3)` or `array()` both work. All arguments are collected into a `Vec<Value>` wrapped in `Rc<RefCell<...>>` and returned as `Value::Array`. The variadic flag bypasses the arity check in the interpreter.

### `ArrayMethod` — method dispatch

```rust
pub struct ArrayMethod {
    pub method_name: String,
    pub array: Rc<RefCell<Vec<Value>>>,
}
```

Array methods are not stored on the array value itself. Instead, when the interpreter evaluates `Expr::Get` on a `Value::Array`, it matches the property name against the known method list and constructs an `ArrayMethod` on the fly:

```rust
Value::Array(arr) => {
    if ["push", "pop", "len", "get", "set"].contains(&&*name.lexeme) {
        Value::Callable(Rc::new(ArrayMethod { method_name: ..., array: arr }))
    }
}
```

Each `ArrayMethod` holds an `Rc` clone of the underlying array, so calling `arr.push` or `arr.pop` mutates the same backing `Vec` that `arr` refers to — reference semantics, consistent with how instances work.

### Methods

| Method | Arity | Behaviour |
|--------|-------|-----------|
| `push(v)` | 1 | appends `v`, returns `nil` |
| `pop()` | 0 | removes and returns last element; runtime error on empty |
| `len()` | 0 | returns length as `Number` |
| `get(i)` | 1 | returns element at index `i` |
| `set(i, v)` | 2 | sets element at index `i` to `v`, returns new value |

Index validation (`get_index`) rejects non-integer and negative indices with a `RuntimeError`.

### Why not a native class?

A native class would require either a separate class-dispatch path or implementing `LoxCallable` on `LoxClass` differently. Attaching methods ad-hoc in `Expr::Get` is simpler and keeps the array's backing `Vec<Value>` directly accessible without going through instance field lookup.
