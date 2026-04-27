# Security policy

## Reporting a vulnerability

Please report security issues **privately** via GitHub's built-in
**Private Vulnerability Reporting**:

> [github.com/Jayanth-MKV/aiscope/security/advisories/new](https://github.com/Jayanth-MKV/aiscope/security/advisories/new)

This opens an encrypted channel visible only to you and the maintainers — no
email needed, no public disclosure until a fix is ready.

If for some reason that link doesn't work for you, open a minimal public
issue titled `security: please contact me` and a maintainer will reach out
through GitHub. **Do not** include vulnerability details in a public issue.

We aim to acknowledge new reports within **72 hours** and ship a fix or
mitigation for confirmed issues within **30 days**.

## Supported versions

Only the latest `0.x` release receives security fixes. Once `1.0` ships,
the latest `1.x` will be the supported line.

## Scope

`aiscope` v0.1 is a read-only local CLI. The threat model is limited to:

- A maliciously crafted rule file causing the parser to crash, OOM, or hang.
- A path-traversal trick causing the scanner to read files outside the project root or `~/.claude/`, `~/.cursor/`.

## Out of scope

- Reading anything under `~/.claude/projects/` is **already disallowed** by `tests/privacy_guard.rs`. If you find a way around that guard, please report it.
- The PNG card renderer uses no untrusted fonts or images; rendered text comes only from your own rule files.

## Acknowledgements

Reporters credited in release notes (opt-in).
