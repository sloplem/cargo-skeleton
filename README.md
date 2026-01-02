# cargo-skeleton

A tool for generating concise summaries of Rust projects, designed to provide context to AI agents while minimizing token usage.

## Overview

`cargo-skeleton` creates information-dense project summaries by:
- Discovering all source files used in a project (via cargo's dependency info)
- Optionally stripping function bodies while preserving signatures
- Filtering out test code
- Counting tokens to measure context usage

This allows you to fit more project context into an AI agent's context window.

## Installation

```bash
cargo install --path .
```

## Usage

### Basic usage (full source code)
```bash
cargo skeleton > project-summary.rs
```

### Stripped mode (signatures only, no tests)
```bash
cargo skeleton --strip > project-skeleton.rs
```

## Features

### Source File Discovery
Uses cargo's `.d` (dependency info) files to discover exactly which source files rustc compiled. This is more accurate than searching for `*.rs` files, as it:
- Only includes files actually used by the build
- Respects `mod` declarations and `#[path]` attributes
- Handles conditional compilation correctly

### Function Body Stripping
The `--strip` flag removes function and method implementations while preserving:
- Function signatures and return types
- Struct and enum definitions
- Trait definitions
- Type aliases and constants
- `use` statements and `mod` declarations
- Doc comments

### Test Filtering
When using `--strip`, items with `#[test]` or `#[cfg(test)]` attributes are automatically removed, as test code is typically not relevant for understanding the core project structure.

### File Separation
Files are separated using C-style `#line` directives:
```rust
#line 1 "src/main.rs"
// ... contents of main.rs ...

#line 1 "src/parser/mod.rs"
// ... contents of parser/mod.rs ...
```

This format is familiar to LLMs from C/C++ training data and clearly indicates file boundaries.

### Token Counting
The tool uses `tiktoken-rs` with the `cl100k_base` encoding (used by GPT-4 and similar to Claude) to estimate token usage. Statistics are printed to stderr:

```
--- Token Statistics ---
Total tokens: 32337
Character count: 123101
Tokens per character: 0.263
```

## Results

Tested on the [`just`](https://github.com/casey/just) command runner (~141K tokens full source):

| Mode | Tokens | % of Full | Reduction |
|------|--------|-----------|-----------|
| Full source | 141,103 | 100% | - |
| `--strip` only | 60,261 | 43% | 57% |
| `--strip` (no tests) | 32,337 | 23% | **77%** |

The stripped skeleton uses only 23% of the original tokens while preserving all structural information.

### Context Window Impact

For Claude's 200K token context window:
- Full `just` source: 141K tokens = 71% of context
- Stripped skeleton: 32K tokens = **16% of context**

You could fit **6+ projects the size of `just`** in a single context window.

## How It Works

1. **Run `cargo check`** to generate dependency info files
2. **Parse `.d` files** in `target/debug/deps/` to discover source files
3. **Filter** to only include `src/` files (excludes dependencies)
4. **For each file:**
   - Output `#line` directive
   - If `--strip`: parse with `syn`, remove function bodies and tests, format with `prettyplease`
   - Otherwise: output as-is
5. **Count tokens** and display statistics

## Future Ideas

### File Ordering
Currently files are output in alphabetical order. Alternative orderings to explore:

- **Depth-first (module tree)**: Follow `mod` declarations, output parent before children
  - Pros: Natural reading order, maintains narrative flow, keeps related code together
  - Cons: Requires parsing to build dependency graph
  - Example: `main.rs` → `parser/mod.rs` → `parser/lexer.rs` → `parser/token.rs`

- **Breadth-first**: Show high-level structure first, then details
  - Pros: Overview before diving deep
  - Cons: May be confusing to see declarations without immediate context

**Research needed**: To determine which ordering works best, we'd need to:
- Test with actual AI agents on real tasks
- Measure task completion, accuracy, or other relevant metrics
- Compare different orderings empirically rather than guessing

This is an example of a feature where agent performance testing would be valuable.

### Other Ideas

- Configurable stripping (e.g., strip only large functions, keep small ones)
- Include/exclude specific modules or files
- Generate project structure diagram
- Support for workspaces (multiple crates)
- Output format options (JSON, LSP-like structure)
- Integration with IDEs or AI coding assistants

## Contributing

Contributions welcome! This tool was designed to help AI agents work with Rust codebases more efficiently.

## License

MIT
