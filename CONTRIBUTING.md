# Contributing to aiscope

Thanks for considering a contribution!

## Quickstart

```bash
git clone https://github.com/Jayanth-MKV/aiscope
cd aiscope
cargo test
cargo run -- --text
```

## Where to start

- **Add a new tool scanner** (`src/scanner/<tool>.rs`): Cline, Aider, Continue, Windsurf are all wanted. Open an issue first so we can agree on the file paths to read.
- **Add conflict pairs** (`src/detect/conflicts.rs::PAIRS`): high-signal, low-false-positive pairs only.
- **Improve the TUI** (`src/render/tui.rs`): currently a text fallback.
- **Improve the PNG card** (`src/render/card.rs`): tiny-skia drawing.

## Privacy ground rules

`aiscope` v0.1 is **read-only and local**. It must never:

1. Read anything under `~/.claude/projects/` (session logs).
2. Send data over the network.
3. Write to your repo without an explicit flag.

The `tests/privacy_guard.rs` test enforces (1). Don't disable it.

## Style

- `cargo fmt` + `cargo clippy -- -D warnings` (CI enforces both).
- Keep PRs small. One feature per PR.
