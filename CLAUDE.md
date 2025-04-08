# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Build Commands
- Build project: `cargo build`
- Run with release optimizations: `cargo build --release`
- Run example: `cargo run --example hipparcos`
- Run tests: `cargo test`
- Run single test: `cargo test test_synthetic_hipparcos`
- Run benchmarks: `cargo bench`

## Lint/Format
- Format code: `cargo fmt`
- Run clippy lints: `cargo clippy`

## Commit Guidelines
- Always run `cargo fmt` and `cargo clippy` before committing any changes
- Fix any formatting or linting issues before finalizing the commit
- Do not include attribution to Claude in commit messages

## Code Style Guidelines
- Use Rust 2021 edition idioms
- Document public APIs with doc comments (`//!` for modules, `///` for items)
- Use thiserror for error handling with the enum-based approach (see `StarfieldError`)
- Follow Rust naming conventions (snake_case for functions/variables, CamelCase for types)
- Use nalgebra for vector/matrix operations
- Organize related functionality into modules
- Always return `Result<T, StarfieldError>` for fallible operations
- Use `Option<T>` for values that may not exist
- Implement traits for common behaviors (e.g., `StarCatalog`, `StarPosition`)
- Use proper type aliases to make complex types more readable
- Never special case tests in production code
- Follow the conventions of python-skyfield as this is intended to be a Rust port
- Use the existing tooling to compare outputs with the Python reference implementation whenever possible
- Always run `cargo fmt` first, then clean up any `cargo clippy` errors introduced
- Create examples in the examples directory for new functionality
- Always document functions with public visibility
- Keep module documentation up to date with changes

## Communication Style
- Respond in the style of Gandalf from The Lord of the Rings