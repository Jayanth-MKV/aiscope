# Changelog

All notable changes to **aiscope** are documented here.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.1.1] — 2026-04-27

### Changed

- `homepage` now points to the mdBook user guide at
  <https://jayanth-mkv.github.io/aiscope/>.
- `documentation` retains <https://docs.rs/aiscope> for API reference.
- README is now embedded as the docs.rs landing page.

## [0.1.0] — 2026-04-27

### Added

- **Multi-tool discovery** across Cursor, Claude Code, and GitHub Copilot,
  spanning all 5 subsystems: instructions, prompts, agents, chat modes, skills.
- **`AGENTS.md` and `CLAUDE.md` any-depth discovery** with automatic
  path-derived scope (e.g. `apps/web/AGENTS.md` → `apps/web/**`).
- **Frontmatter-aware** parser for `applyTo`, `globs`, `alwaysApply`, `tools:`,
  `model:`, `name:`, `description:`.
- **Scope overlap detection** via `globset` — non-overlapping rules are
  demoted to Low severity instead of false-positive HIGH conflicts.
- **Two reasoning modes**: `--specific` (subsystem-aware) and Uniform (default).
- **Agent tool-allowlist mismatch detector** — flags when an instruction says
  _"use the X tool"_ but the agent's `tools:` excludes X.
- **Duplicate-name detection** across agents, skills, and chat modes.
- **5 renderers**: ratatui TUI, `miette` compiler-style diagnostics, plain
  text, JSON, and a 1280×720 PNG summary card with embedded JetBrains Mono.
- **CI gate**: `aiscope check` exits non-zero on HIGH conflicts.
- **Privacy guard**: never reads outside the repo unless `--user` is passed;
  never reads Claude Code transcript history under `~/.claude/projects/`.

### Quality

- 42 tests passing (33 unit + 5 corpus snapshots + 2 copilot-only integration
  - 1 smoke + 1 privacy guard).
- `cargo clippy --all-targets -- -D warnings` clean.
- ~5 MB release binary.

[Unreleased]: https://github.com/Jayanth-MKV/aiscope/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/Jayanth-MKV/aiscope/releases/tag/v0.1.0
