# rust-skeleton

`rust-skeleton` makes large Rust files easier for agents and humans to inspect. It preserves the structure of a source file while replacing function and method bodies with compact source-line markers.

```rust
pub fn solve(input: &Input) -> Result<Output, Error> {
    /* implementation omitted: source lines 48-193 */
}
```

The output retains imports, modules, comments outside implementations, attributes, types, fields, variants, trait definitions, signatures, associated items, and macros. Formatting outside replaced bodies is left unchanged.

## Install

```bash
cargo install --path .
```

## Use

```bash
rust-skeleton src/components/core.rs
```

The skeleton is written to stdout. Reduction statistics are written to stderr:

```text
6038 lines, 226384 symbols -> 2457 lines, 75192 symbols
```

This keeps the generated source pipeable:

```bash
rust-skeleton src/components/core.rs > core-skeleton.rs
```

To inspect only the reduction statistics:

```bash
rust-skeleton src/components/core.rs > /dev/null
```

Use an omission marker's source range to open the relevant implementation in the original file when behavioral details are needed.

## Current scope

- Processes one UTF-8 Rust source file at a time.
- Removes free-function, method, and default trait method bodies.
- Leaves empty bodies, function declarations, constants, statics, and macros unchanged.
- Rejects files with syntax errors instead of producing a partial skeleton.
- Produces valid Rust syntax, but the result is not expected to compile.

“Symbols” are counted as Unicode characters rather than bytes.

## AGENTS.md

Add this to your AGENTS.md for your LLM agents to use it:

````md
# Rust skeleton

Use `rust-skeleton <file.rs>` to inspect a Rust file without function implementations. It preserves imports, comments, attributes, types, fields, signatures, and macros, while replacing function and method bodies with source-line markers.

The skeleton is written to stdout; before/after line and symbol counts are written to stderr.

```bash
rust-skeleton src/components/core.rs
rust-skeleton src/components/core.rs > /tmp/core-skeleton.rs
rust-skeleton src/components/core.rs > /dev/null  # inspect reduction statistics only
```

Use the reported source ranges to open the original implementation when behavioral details are needed.
````
