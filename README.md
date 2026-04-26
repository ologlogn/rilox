# rilox

A Rust implementation of the [Lox](https://craftinginterpreters.com/) programming language — a dynamically-typed scripting language from Robert Nystrom's *Crafting Interpreters*.

## Features

**Language**
- Variables (`var`), numbers, booleans, strings, nil
- Arithmetic, comparison, logical operators
- Control flow: `if`/`else`, `while`, `for`
- First-class functions with closures and recursion
- `print` statement, single-line comments (`//`)

**OOP**
- Classes with `init()` constructors
- Instance methods and dynamic properties
- Single inheritance (`class B < A`)
- `this` and `super`

**Built-ins**
- `clock()` — Unix timestamp in seconds
- `read_line()` — read a line from stdin, returns `nil` on EOF
- `to_number(v)` — converts string/bool/number to number, `nil` on failure
- `floor(n)` — floor of a number
- `array(...)` — variadic array constructor; methods: `.push(v)`, `.pop()`, `.get(i)`, `.set(i, v)`, `.len()`

## Usage

```bash
# REPL
cargo run

# Run a script
cargo run --release path/to/script.lox
```

## Example

```lox
class Animal {
  init(name) {
    this.name = name;
  }
  speak() {
    print this.name + " makes a sound.";
  }
}

class Dog < Animal {
  speak() {
    print this.name + " barks.";
  }
}

var d = Dog("Rex");
d.speak(); // Rex barks.
```

## Implementation

**Pipeline:** source → lexer → parser (AST) → resolver → tree-walk interpreter

**Two-pass execution**
The resolver runs before the interpreter. It walks the AST and records each variable's *(scope depth, slot index)* so the interpreter can look up locals in O(1) without hash lookups. The resolver also catches static errors: `return` outside a function, `this`/`super` outside a class, a class inheriting from itself, and a variable referenced in its own initializer.

**Environment / scoping**
- Global scope stores variables in a `HashMap<String, Value>`.
- Local scopes store values in a `Vec<Value>` indexed by the slot the resolver computed — no string lookups at runtime.
- Scopes form a parent-chain via `Rc<RefCell<Environment>>`; `ancestor(env, depth)` walks the chain to the right scope frame.

**Value representation**
```
Number(f64)
Boolean(bool)
String(String)
Nil
Callable(Rc<dyn LoxCallable>)       // functions, native fns
Class(Rc<LoxClass>)                 // immutable class definition
Instance(Rc<RefCell<LoxInstance>>)  // mutable fields via RefCell
Array(Rc<RefCell<Vec<Value>>>)      // mutable array via RefCell
```

**Truthiness:** `nil` → false, `0.0` → false, `""` → false, everything else → true.

**Closures & method binding**
Functions capture their defining environment by `Rc` clone. When a method is retrieved from an instance (`.method`), a new `LoxFunction` is created with `this` bound in a fresh environment wrapping the closure — so methods can be stored in variables and called later.

**Inheritance**
`super` is injected as a variable one scope outside `this`. The resolver pushes a `super` scope when entering a subclass, and the interpreter walks up to it at call time.

## Package Details

### `lexer`
| File | Responsibility |
|------|---------------|
| `scanner.rs` | Converts source string into a `Vec<Token>`. Tracks `start`/`current`/`line` cursors. Handles single- and two-character operators with one-character lookahead, multi-line strings, `f64` number literals, identifiers, and keywords. |
| `token.rs` | Defines `TokenType` (37 variants), `Literal` (Number/String/Boolean/Nil), and `Token` (type + lexeme + optional literal + line). `keywords()` maps reserved words to their `TokenType`. |

### `parser`
| File | Responsibility |
|------|---------------|
| `parser.rs` | Recursive descent parser. Produces `Vec<Statement>` from tokens. Returns exit code 65 on parse error. |
| `expr.rs` | `Expr` enum — 12 variants: `Literal`, `Unary`, `Binary`, `Grouping`, `Variable`, `Assignment`, `Logical`, `Call`, `Get`, `Set`, `This`, `Super`. |
| `stmt.rs` | `Statement` enum — 9 variants: `ExpressionStmt`, `PrintStmt`, `VarStmt`, `BlockStmt`, `IfStmt`, `WhileStmt`, `FunctionStmt`, `ReturnStmt`, `ClassStmt`. Also defines `FunctionType` (NONE/FUNCTION/METHOD/INIT) and `ClassType` (NONE/CLASS/SUBCLASS) used by the resolver. |

### `interpreter`
| File | Responsibility |
|------|---------------|
| `interpreter.rs` | Tree-walk interpreter. Holds `globals` (always the root env), `env` (current scope), `locals` (resolver output). `eval_expr` dispatches on `Expr`, `execute_stmt` dispatches on `Statement`. Registers all native globals on construction. |
| `resolver.rs` | Single-pass semantic analyser. Walks the AST before execution and records each local variable as `(scope_depth, slot_index)` in a `HashMap<*const Expr, (usize, usize)>`. Catches static errors: `return` outside function, `this`/`super` outside class, self-inheritance, variable read in its own initializer, `return` with value inside `init`. |
| `env.rs` | `Environment` holds a `HashMap` for globals and an indexed `Vec<Value>` for locals (resolver pre-computes slot indices). `ancestor(env, depth)` walks the parent chain. `get_at`/`assign_at` do O(1) local access by depth + index. |
| `value.rs` | `Value` enum: `Number(f64)`, `Boolean(bool)`, `String`, `Nil`, `Callable(Rc<dyn LoxCallable>)`, `Class(Rc<LoxClass>)`, `Instance(Rc<RefCell<LoxInstance>>)`, `Array(Rc<RefCell<Vec<Value>>>)`. Implements `is_truthy` and `is_equal`. |
| `function.rs` | `LoxCallable` trait (`arity`, `call`, `is_variadic`). `LoxFunction` stores params, body (`Rc<Box<Statement>>` — shared, never cloned), closure env, and function type. `bind(instance)` creates a new env with `this` injected, returning a new `LoxFunction`. `init` always returns `this` regardless of return value. |
| `class.rs` | `LoxClass` stores name, `HashMap<String, Rc<LoxFunction>>` for methods, and optional `Rc<LoxClass>` superclass. `find_method` walks the superclass chain. Calling a class allocates a `LoxInstance` and invokes `init` if present. |
| `instance.rs` | `LoxInstance` stores a `Rc<LoxClass>` and a `HashMap<String, Value>` for fields. `get` checks fields first, then the class method chain, binding `this` on method access. |

### `native`
| File | Responsibility |
|------|---------------|
| `clock.rs` | `clock()` — returns `SystemTime::now()` as `f64` seconds since Unix epoch. |
| `io.rs` | `read_line()` — flushes stdout, reads one line from stdin, trims trailing newline. Returns `nil` on EOF or error. |
| `convert.rs` | `to_number(v)` — number → identity, string → `f64::parse`, bool → 1.0/0.0, anything else → `nil`. |
| `math.rs` | `floor(n)` — returns `f64::floor` of a number value. |
| `array.rs` | `array(...)` — variadic constructor returning `Value::Array(Rc<RefCell<Vec<Value>>>)`. Array methods (`push`, `pop`, `len`, `get`, `set`) are dispatched as `ArrayMethod` callables via `Expr::Get` on an array value. Indices must be non-negative integers. |

### `error.rs`
`Error` enum with two variants:
- `RuntimeError(String)` — printed to stderr; `runtime_error()` calls `process::exit(70)`.
- `Return(Value)` — used as a control-flow signal bubbled up through `execute_stmt` to unwind the call stack; caught in `LoxFunction::call`.

## Exit Codes

| Code | Meaning       |
|------|---------------|
| 0    | Success       |
| 65   | Parse error   |
| 70   | Runtime error |
