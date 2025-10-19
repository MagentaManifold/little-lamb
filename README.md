# little-lamb

> Mary had a little lamb,
>
> little lamb, little lamb.
>
> Mary had a little lamb,
>
> its fleece was white as snow...

A simple lambda calculus interpreter in Rust and Chumsky (a beautiful parser combinator library), using de Bruijn indices and normal order evaluation.

> [!NOTE]
> This repo is a toy project and is under active development.

## Syntax

Pretty standard lambda calculus syntax. Indentation and line breaks are ignored.

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

### Natural numbers

Natural numbers are a syntax sugar for Church numeral encoding.

```little-lamb
import add in
add 2 3
-- evaluates to \f . \x . f (f (f (f (f x)))),
-- i.e., 5 in Church numeral encoding
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

- **Combinators**: `I` (identity), `K` (constant), `S` (substitution)
- **Boolean logic**: `true`, `false`, `and`, `or`, `not`
- **Arithmetic**: `add`, `mult`, `succ` (successor)

**Current limitations:**

- No nested directories (only flat module structure)
- No relative imports (`../` or `./`)
- Does not support circular dependency

## Examples

Examples can be found in the `/examples` directory. Some of them don't make much sense.

See below for how to run them.

## Usage

### Prerequisites

- Rust (recent version should work)
- Cargo

### Running stuff

```bash
# Clone the repo
git clone https://github.com/MagentaManifold/little-lamb
cd little-lamb

# Try an example
cargo run -- examples/example.lil

# Run tests
cargo test

# Build release version
cargo build --release

# install little-lamb to your system
cargo install --path .
```

### Run your own programs

```bash
cargo run -- /path/to/your_program.lil
```

to run directly, or

```bash
little-lamb /path/to/your_program.lil
```

if you have installed the binary.

## How it works

### Architecture

Pretty straightforward:

- **AST** (`src/ast.rs`): AST/Expression/de Bruijn term types and conversions
- **Lexer** (`src/lexer.rs`): Tokenizes source code into tokens
- **Parser** (`src/parser.rs`): Parses tokens into AST using Chumsky
- **Evaluator** (`src/eval.rs`): desugar + De Bruijn conversion + beta reduction
- **Main** (`src/main.rs`): CLI

### Evaluation

1. Tokenize source into `Vec<Token>`
2. Parse tokens into `Ast`
3. Desugar AST into `Expr`
4. Convert to de Bruijn indices (`Term`)
5. Beta reduce until normal form (or give up after step limit reached)
6. Print result

## Contributing

I'd be surprised if anyone else would like to work on it, but let me know if you are interested.

## TODO

Roughly in descending order of priority:

- [x] Syntax support for common encodings like booleans (done with importing) and Church numerals.
- [x] Improve performance
- [ ] Improve import system: nested directories, better error messages
- [ ] Support converting results back to primitives and common combinators
- [ ] Better error messages  
- [ ] Support step by step evaluation
- [ ] Support REPL
- [ ] Add more examples
- [ ] Clean up tests (most are LLM generated, some isn't really helpful)
- [ ] LSP support
- [ ] Support [Tromp's Diagram](https://tromp.github.io/cl/diagrams.html) styled visualization or other types of graphical representation (a very ambitious goal)

## License

This project is is licensed under MIT License.
