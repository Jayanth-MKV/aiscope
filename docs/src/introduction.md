# aiscope

> **DevTools for your AI coding tools' memory.**
> See what Cursor, Claude Code, and GitHub Copilot actually remember about
> your project — and where they disagree.

![aiscope summary card](https://raw.githubusercontent.com/Jayanth-MKV/aiscope/main/aiscope-demo.png)

## What it does

You have `.cursorrules`. You have `.github/copilot-instructions.md`. You have
`CLAUDE.md`. You have `.github/instructions/python.instructions.md`. You have
`.github/agents/reviewer.agent.md`. Maybe a `.claude/skills/` folder. Maybe an
`apps/web/AGENTS.md`.

Every one of them is a markdown file silently shoved into the model's context
window. None of them know about each other. So you ship code where:

- One file says **"use snake_case"**, another says **"use camelCase"**.
- The agent allowlist says `tools: [read, search]` but your instructions say
  _"use the bash tool to run pytest"_.
- Two files repeat the same sentence in slightly different words — burning
  tokens to tell the model the same thing twice.
- A `python.instructions.md` (`applyTo: **/*.py`) and a
  `typescript.instructions.md` (`applyTo: **/*.ts`) look like a contradiction
  but never apply to the same file. False alarm.

aiscope is a **single, deterministic Rust binary** that finds these — and
knows the difference between a real conflict and a false alarm.

## Three minutes to value

```bash
cargo install --git https://github.com/Jayanth-MKV/aiscope
cd your-repo
aiscope .                # interactive TUI
aiscope --diag .         # compiler-grade diagnostics
aiscope check .          # exits non-zero on HIGH conflicts (CI gate)
```

## Where to next

- [Install](./getting-started/install.md) — every supported way to get aiscope
- [Quickstart](./getting-started/quickstart.md) — first scan in 60 seconds
- [How aiscope thinks](./concepts/how-it-works.md) — the 6-layer pipeline
- [Per-tool guides](./tools/copilot.md) — what aiscope expects to find
- [GitHub Actions](./ci/github-actions.md) — wire it into CI today

## Status

- **Version**: 0.1.0
- **Tests**: 42 passing across unit, corpus snapshot, and integration suites
- **License**: MIT
- **Source**: [github.com/Jayanth-MKV/aiscope](https://github.com/Jayanth-MKV/aiscope)
