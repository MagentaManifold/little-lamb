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

Pretty standard lambda calculus syntax:

### Lambda expressions

```lambda
\x . x       -- identity
```

```lambda
\x . \y . x  -- K combinator
```

No support for currying, and you can't use parentheses around lambdas for now. E.g., `(\x . x)` is invalid.

### Function application

```little-lamb
(f x)        -- apply f to x
```

No support for multiple arguments or omitting parenthesis for now, so `(f x y)` is invalid, and the correct syntax is `((f x) y)`.

### Let bindings

```little-lamb
let id = \x . x in
let const = \x . \y . x in
((const id) id)
```

`let a = <X> in <Y>` is a syntax sugar for `(\a . <Y> <X>)`

### Comments

Anything after `--` until the end of line is ignored. There are no comment blocks.

```little-lamb
\x . x -- this is a comment
```

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
```

### Write your own programs

Make a `.lil` file (or any plaintext file; file extension doesn't really matter):

```bash
cargo run -- /path/to/your_program.lil
```

## How it works

### Architecture

Pretty straightforward:

- **Parser** (`src/parser.rs`): Turns `.lil` files into AST using Chumsky
- **AST** (`src/ast.rs`): Expression/de Bruijn term types and conversions
- **Evaluator** (`src/eval.rs`): De Bruijn conversion + beta reduction
- **Main** (`src/main.rs`): CLI

### Evaluation

1. Parse source → `Expr` AST
2. Convert to De Bruijn indices → `Term`
3. Beta reduce until normal form (or give up)
4. Print result

## Contributing

I'd be surpriced if anyone else would like to work on it, but let me know if you
are interested.

## TODO

Roughly in descending order of priority:

- [ ] Syntax support for currying and multi-argument application (might want to add a lexer before parser first)
- [x] Syntax support for comments
- [ ] Syntax support for common primitives like booleans and Church numerals.
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
