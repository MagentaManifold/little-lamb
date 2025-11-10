# little-lamb

> Mary had a little lamb,
>
> little lamb, little lamb.
>
> Mary had a little lamb,
>
> its fleece was white as snow...

A *"blazingly fast"* untyped lambda calculus interpreter in Rust and Chumsky (a beautiful parser combinator library), using full-reducing Krivine machine (KN) as the default evaluation strategy.

> [!NOTE]
> This repo is a toy project and is under active development.

## Usage

### Prerequisites

- Rust (recent version should work)
- Cargo

### Installation

```bash
# Install from git
cargo install --git https://github.com/MagentaManifold/little-lamb
```

### Evaluate a program

```bash
little-lamb /path/to/your_program.lil
```

### Try out examples

Examples can be found in the `/examples` directory. Some of them don't make much sense.

```bash
# Run an example
cargo run -- examples/example.lil
# or run with installed binary
little-lamb examples/example.lil
```

## Syntax

Standard-ish lambda calculus syntax with sugars added. Indentation and line breaks are ignored.

Each file represents exactly one expression in untyped lambda calculus.

### Lambda expressions

```little-lamb
\x . x       -- identity
```

```little-lamb
\x y . x     -- K combinator
```

The currying syntax is a syntax sugar for nested lambdas.
E.g., `\x y . x` is equivalent to `\x . \y . x`.

### Function application

```little-lamb
f x          -- apply f to x
```

Function application is left associative and has higher precedence than lambda (and parentheses have the highest precedence, of course).
E.g., `\x . x x x` is equivalent to `\x . ((x x) x)`.

### Let bindings

`let a = <X> in <Y>` is a syntax sugar for `(\a . <Y>) <X>`.

```little-lamb
let I = \x . x in
let K = \x y . x in
K I K
```

Multiple let bindings can also be written using comma syntax:

```little-lamb
let
  I = \x . x,
  K = \x y . x, -- trailing comma is optional
in
K I K
```

```little-lamb
-- Or, you can even do leading comma (inspired by Elm)
let
, I = \x . x
, K = \x y . x
in
K I K
```

All above are equivalent. The comma syntax supports leading and trailing commas for convenience.

### Recursive let bindings

`letrec f = <X> in ...` is a syntax sugar for `let f = Y (\f . <X>) in ...`, where `Y` is the fixed-point combinator.

```little-lamb
import if, iszero, pred, mult in
letrec factorial =
  \n. if (iszero n) 1 (mult n (fact (pred n)))
in factorial 5
```

The comma syntax is also supported for multiple recursive bindings, although it's merely a syntax sugar for nested `letrec`s, so mutual recursion is not supported.

### Built-in types

#### Natural numbers

Natural numbers are a syntax sugar for Church numeral encoding.

```little-lamb
import add in
add 2 3
-- evaluates to \f . \x . f (f (f (f (f x)))),
-- i.e., 5 in Church numeral encoding
```

#### Boolean

Boolean values are a syntax sugar for Church encoding of booleans.

```little-lamb
import and in
and true false
-- evaluates to \x . \y . y, i.e., false in Church encoding
```

### Comments

Anything after `--` until the end of line is ignored. There are no comment blocks.

```little-lamb
\x . x -- this is a comment
```

### Import

Import syntax is still WIP. Currently, it only supports flat imports (no nested directories) and has limited error handling.

You can import definitions from other files or from the built-in standard library:

```little-lamb
import I in        -- import identity combinator from std lib
import K as const in   -- import with alias
import add, succ in  -- import multiple modules
I const
```

`import <mod> as <name> in <body>` is a syntax sugar for `let <name> = <module file content> in <body>`, and comma separated imports is equivalent to nested imports.

The import system works as follows:

1. **User modules**: First search for `.lil` files relative to the current file's directory
2. **Built-in modules**: Fallbacks to searching in the embedded standard library (`lib/` directory)

Available built-in modules include:

- **Combinators**: `I` (identity), `K` (constant), `S` (substitution), `Y` (fixed point)
- **Boolean logic**: `and`, `or`, `not`, `if`
- **Pairs**: `pair`, `car`, `cdr`
- **Arithmetic**: `add`, `mult`, `sub`, `succ` (successor), `pred` (predecessor), `iszero`

**Current limitations:**

- No nested directories (only flat module structure)
- No relative imports (`../` or `./`)
- Circular dependencies will cause an error

## Development

```bash
# Run tests
cargo test

# Run benchmarks (uses Criterion for stats)
cargo bench

# Build release version
cargo build --release
```

## How it works

### Code structure

- **Syntax** (`src/syntax/`): AST/Expression/de Bruijn term types and conversions (`ast.rs`, `expr.rs`, `term.rs`) plus desugaring (`desugar.rs`)
- **Lexer** (`src/lexer.rs`): Tokenizes source code into tokens
- **Parser** (`src/parser.rs`): Parses tokens into AST using Chumsky
- **Import** (`src/import.rs`): Handles module imports and dependency resolution
- **Evaluator** (`src/eval/`): Multiple evaluation strategies
  - `kn.rs`: A [Full-reducing KN machine](https://doi.org/10.1007/s10990-007-9015-z).
  - `substitution.rs`: Naive substitution based evaluation, very slow .
  - `krivine.rs`: A pile of incomprehensible vibe-coded crap that miraculously works. Left here only as a baseline for benchmarking.
- **Main** (`src/main.rs`): CLI entry point

### Pipeline

1. Tokenize source into `Vec<Token>`
2. Parse tokens into `Ast`
3. Desugar AST into `Expr`
4. Convert to de Bruijn indices (`Term`)
5. Eval to normal form (or give up after step limit reached)
6. Print result

## Contributing

I'd be surprised if anyone else would like to work on it, but let me know if you are interested.

## TODO

Roughly in descending order of priority:

- [ ] Syntax support for more structures, like pairs and lists
- [x] Improve performance
- [ ] Improve import system: nested directories, better error messages
- [ ] Support converting results back to primitives and common combinators (not perfect right now, but basically working)
- [ ] Better error messages  
- [ ] Support for binary operators
- [ ] Support step by step evaluation
- [ ] Support REPL
- [ ] Add more examples/library functions
- [ ] Create a web demo (compile to WASM)
- [ ] Clean up tests (most are LLM generated, some aren't really helpful)
- [ ] LSP support (Kinda ambitious, could be another project)
- [ ] Support [Tromp's Diagram](https://tromp.github.io/cl/diagrams.html) styled visualization or other types of graphical representation (a very ambitious goal)

## License

This project is licensed under MIT License.
