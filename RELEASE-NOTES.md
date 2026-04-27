# aiscope v0.1.0

**The first release.** A single Rust binary that finds conflicts, duplicates, and tool-allowlist mismatches across Cursor, Claude Code, and GitHub Copilot memory files.

## What's inside

- **3 tools × 5 subsystems** discovered automatically:
  - Cursor — `.cursorrules`, `.cursor/{rules,commands,agents,modes}/`
  - Claude — `CLAUDE.md` (any depth, path-scoped), `.claude/{agents,commands,skills/*/SKILL.md}`
  - Copilot — `.github/copilot-instructions.md`, `.github/{instructions,prompts,agents,chatmodes}/`, plus `AGENTS.md` (any depth, path-scoped)
- **Frontmatter-aware**: parses `applyTo`, `globs`, `alwaysApply`, `tools:`, `model:`, `name:`, `description:`
- **Scope overlap detection** via [`globset`](https://docs.rs/globset) — non-overlapping rules can't actually clash, so they're demoted to Low severity
- **Two reasoning modes**: `--specific` for subsystem-aware filtering (Prompts ↔ Instructions never conflict, etc.), default Uniform for max recall
- **Agent tool-allowlist mismatch detector** — flags when an instruction says _"use the bash tool"_ but the agent's `tools:` excludes it
- **Duplicate-name detection** across agents, skills, chat modes
- **Privacy guard** — never reads outside the repo unless `--user` is passed
- **5 renderers**: ratatui TUI, `miette` compiler-style diagnostics, plain text, JSON, 1280×720 PNG card
- **CI gate**: `aiscope check` exits non-zero on HIGH conflicts

## Quality

- **42 tests** passing (33 unit + 5 corpus snapshots + 2 copilot-only integration + 1 smoke + 1 privacy guard)
- **Clippy** `-D warnings` clean
- **~5 MB** release binary
- **Rust 1.95**, edition 2024
- **Deterministic** — same input always produces the same diagnostics, in the same order

## Install

```bash
cargo install --git https://github.com/Jayanth-MKV/aiscope
```

## Quickstart

```bash
aiscope .                  # interactive TUI
aiscope --text .           # plain text
aiscope --diag .           # compiler-grade diagnostics
aiscope --json .           # machine-readable
aiscope --card out.png .   # 1280×720 PNG summary
aiscope check --specific . # CI gate
```

## CI

```yaml
- uses: actions/checkout@v4
- run: cargo install --git https://github.com/Jayanth-MKV/aiscope
- run: aiscope check --specific .
```

## Acknowledgments

- [JetBrains Mono](https://www.jetbrains.com/lp/mono/) (Apache-2.0) — embedded for the PNG card
- `tiny-skia`, `cosmic-text`, `ratatui`, `miette`, `globset`, `walkdir`, `clap` — incredible Rust ecosystem

See [BLOG.md](BLOG.md) for the longer story.
