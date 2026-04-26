# parser

Turns a flat `Vec<Token>` into an AST represented as `Vec<Statement>`.

## Files

- `parser.rs` — recursive descent parser
- `expr.rs` — expression node types
- `stmt.rs` — statement node types, function/class type enums

---

## Design: recursive descent

Each grammar rule maps directly to a method. There is no separate grammar file — the Rust method call stack *is* the parse stack. This keeps the code easy to follow and error recovery straightforward: on a parse error, the parser returns `Err` immediately and the caller propagates it up.

---

## expr.rs

```rust
pub enum Expr {
    Literal(Literal),
    Unary     { operator: Token, right: Box<Expr> },
    Binary    { left: Box<Expr>, operator: Token, right: Box<Expr> },
    Grouping  { expr: Box<Expr> },
    Variable  (Token),
    Assignment{ name: Token, value: Box<Expr> },
    Logical   { left: Box<Expr>, operator: Token, right: Box<Expr> },
    Call      { callee: Box<Expr>, args: Vec<Expr> },
    Get       { object: Box<Expr>, name: Token },
    Set       { object: Box<Expr>, name: Token, value: Box<Expr> },
    This      { token: Token },
    Super     { keyword: Token, method: Token },
}
```

### Why `Box<Expr>` for children?

`Expr` is a recursive enum — without indirection Rust cannot compute its size. `Box` is the minimal-overhead fix: one heap allocation per node, pointer-sized slot in the parent.

### Why `Logical` is separate from `Binary`

`and`/`or` short-circuit: the right operand must not be evaluated if the left already determines the result. Keeping them as a distinct variant makes the interpreter's match arm explicit about that — there is no risk of accidentally evaluating both sides.

### `Get` vs `Set`

Property access (`obj.field`) and property assignment (`obj.field = val`) are separate variants rather than one node with an optional value. This makes the interpreter dispatch unambiguous and avoids an `Option<Box<Expr>>` in the common read path.

### `Super` stores both tokens

`Super { keyword, method }` keeps the `super` keyword token (for error reporting and resolver lookup) and the method name token separately. The resolver resolves the `keyword` token to find the superclass in the environment, not the method name.

### Pointer identity for the resolver

The resolver maps `*const Expr` (raw pointer) to `(depth, index)` pairs. This works because the AST is allocated once and never moved — the same pointer that the resolver saw is the same pointer the interpreter sees at runtime. No `NodeId` indirection is needed.

---

## stmt.rs

```rust
pub enum Statement {
    ExpressionStmt(Expr),
    PrintStmt(Expr),
    VarStmt(Token, Option<Expr>),                          // name, optional initializer
    BlockStmt(Vec<Statement>),
    IfStmt(Expr, Box<Statement>, Option<Box<Statement>>),  // cond, then, else?
    WhileStmt(Expr, Box<Statement>),
    FunctionStmt(Token, Vec<Token>, Rc<Box<Statement>>, FunctionType),
    ReturnStmt(Token, Option<Expr>),
    ClassStmt(Token, Vec<Statement>, Option<Expr>),        // name, methods, superclass?
}
```

### `Rc<Box<Statement>>` for function bodies

Function bodies are shared: a method defined in a class is referenced by every bound instance of that method without copying the AST. `Rc` provides cheap reference-counted sharing. `Box<Statement>` is there because `Statement` is recursive and needs heap allocation anyway; the `Rc` wraps the already-boxed value.

### `FunctionType` in `FunctionStmt`

```rust
pub enum FunctionType { NONE, FUNCTION, METHOD, INIT }
```

The parser tags every `fun` declaration at parse time with its `FunctionType`. This lets the resolver know immediately whether it is inside an `init` method without extra context threading. `NONE` is the sentinel used by the resolver to detect `return` outside a function.

### `ClassType` in the resolver

```rust
pub enum ClassType { NONE, CLASS, SUBCLASS }
```

Defined here (alongside `FunctionType`) because both are used by the resolver and the parser tags them at parse time. `SUBCLASS` enables the resolver to distinguish `super` usage in a class with vs. without a superclass.

### `for` desugaring

`for` loops are not a dedicated `Statement` variant. The parser desugars them into a `BlockStmt` containing the initializer, a `WhileStmt` with the condition, and the increment appended to the body. This keeps the interpreter and resolver ignorant of `for` entirely.
