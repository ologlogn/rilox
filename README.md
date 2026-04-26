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

## Usage

```bash
# REPL
cargo run

# Run a script
cargo run -- path/to/script.lox
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

## Project Structure

```
src/
  lexer/        # Scanner and token types
  parser/       # Recursive descent parser, AST nodes
  interpreter/  # Tree-walk interpreter, resolver, environment
  native/       # Built-in functions
  error.rs      # Error types
resources/      # Example and test Lox scripts
```

## Exit Codes

| Code | Meaning       |
|------|---------------|
| 0    | Success       |
| 65   | Parse error   |
| 70   | Runtime error |
