# lexer

Converts raw source text into a flat `Vec<Token>` that the parser consumes.

## Files

- `scanner.rs` — the scanning loop
- `token.rs` — token and literal types

---

## scanner.rs

### State

```rust
struct Scanner {
    source: Vec<char>,  // pre-collected so random access is O(1)
    tokens: Vec<Token>,
    start: usize,       // start of current lexeme
    current: usize,     // next character to consume
    line: usize,        // for error reporting
}
```

`source` is stored as `Vec<char>` rather than operating on a `&str` directly. This avoids byte-index arithmetic when peeking: `source[current]` always gives a Unicode scalar, not a UTF-8 byte, so multi-byte characters don't break the cursor math.

### Scanning loop

`scan_tokens` drives a `while !is_at_end()` loop. Each iteration snapshots `start = current`, then calls `scan_token` which advances `current` by one or more characters and appends a token. The loop ends with an explicit `EOF` token so the parser always has a clean terminator.

### One-character lookahead

Two-character operators (`!=`, `==`, `>=`, `<=`) are handled with `if_next_char_then_advance`: peek at `source[current]`, and only advance if it matches. This avoids backtracking entirely — the scanner is strictly left-to-right with at most one character of lookahead.

```rust
'!' => {
    if self.if_next_char_then_advance('=') {
        self.add_token(TokenType::BangEqual, None)
    } else {
        self.add_token(TokenType::Bang, None)
    }
}
```

### Comments

`//` is consumed by advancing until `\n` without emitting a token. Block comments are not supported — this matches the base Lox spec.

### Strings

Multi-line strings are supported: the scanner increments `line` for every `\n` it sees inside a string literal. On close `"` it slices `source[start+1..current-1]` (stripping the quotes) and stores a `Literal::String`.

Unterminated strings call `error(line, ...)` and return without a token. The parser will then fail on the unexpected `EOF`.

### Numbers

Numbers are parsed as `f64`. The scanner greedily consumes digits, then checks for a `.` followed by at least one digit before consuming the fractional part (`peek` + `peek_next` two-character lookahead). This prevents `42.` from being parsed as a float — the dot would be treated as a `Dot` token instead.

### Identifiers and keywords

Any `[a-zA-Z_][a-zA-Z0-9_]*` sequence is first checked against the keyword table in `token.rs`. If it matches, the corresponding keyword token is emitted; otherwise `Identifier` is emitted. This means keywords are reserved — you cannot use `class` as a variable name.

---

## token.rs

### TokenType

37 variants covering:
- Single-character punctuation: `(`, `)`, `{`, `}`, `,`, `.`, `-`, `+`, `;`, `/`, `*`
- One-or-two character operators: `!`, `!=`, `=`, `==`, `>`, `>=`, `<`, `<=`
- Literals: `Identifier`, `String`, `Number`
- Keywords: `and`, `class`, `else`, `false`, `fun`, `for`, `if`, `nil`, `or`, `print`, `return`, `super`, `this`, `true`, `var`, `while`
- `EOF`

### Literal

`Literal` is a separate enum that carries the actual value of a token when it has one:

```rust
pub enum Literal {
    Number(f64),
    String(String),
    Boolean(bool),
    Nil,
}
```

`Boolean` and `Nil` variants exist so the parser can produce `Literal` values directly from keyword tokens (`true`, `false`, `nil`) without a second conversion step.

### Token

```rust
pub struct Token {
    pub token_type: TokenType,
    pub lexeme: String,       // original source text
    pub literal: Option<Literal>,
    pub line: usize,
}
```

`lexeme` stores the raw source slice. This is used in error messages and as the key for variable lookups throughout the interpreter. `line` is propagated all the way to runtime so error messages always report the correct source location.
