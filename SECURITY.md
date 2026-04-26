# Security policy

## Reporting a vulnerability

Please email security@aiscope.dev (placeholder) with details. Do **not** open a public issue for security problems.

## Scope

`aiscope` v0.1 is a read-only local CLI. The threat model is limited to:

- A maliciously crafted rule file causing the parser to crash, OOM, or hang.
- A path-traversal trick causing the scanner to read files outside the project root or `~/.claude/`, `~/.cursor/`.

## Out of scope

- Reading anything under `~/.claude/projects/` is **already disallowed** by `tests/privacy_guard.rs`. If you find a way around that guard, please report it.
- The PNG card renderer uses no untrusted fonts or images; rendered text comes only from your own rule files.

## Acknowledgements

Reporters credited in release notes (opt-in).
