# DESIGN

## Context
Mixed Pickles validates git commit messages from CLI and Python APIs.

## Decision
Core validation logic lives in Rust, exposed through a CLI and PyO3 bindings. Git command output is parsed with robust delimiters, and user-controlled commit metadata is sanitized before terminal output.

## Trade-offs
Rust-first design improves performance and correctness, but increases FFI complexity for Python consumers.

## Consequences
CLI and Python interfaces remain consistent while reducing parsing and output-injection risk.
