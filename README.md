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
- `read_line()` — read from stdin
- `to_number()` — type conversion
- `Array(...)` — variadic array constructor with methods: `.push(v)`, `.pop()`, `.get(i)`, `.set(i, v)`, `.len()`

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

## Project Structure

```
src/
  lexer/        # Scanner and token types
  parser/       # Recursive descent parser, AST nodes
  interpreter/  # Tree-walk interpreter, resolver, environment
  native/       # Built-in functions (clock, io, convert, array)
  error.rs      # Error types
resources/      # Example Lox scripts (algorithms, classes, closures, ...)
```

## Exit Codes

| Code | Meaning       |
|------|---------------|
| 0    | Success       |
| 65   | Parse error   |
| 70   | Runtime error |
