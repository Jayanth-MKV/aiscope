# Quickstart

```bash
cd your-repo
aiscope .
```

That's it. aiscope will scan your repo for every supported AI memory file,
extract rules, find conflicts, and drop you into an interactive TUI.

## What happens under the hood

When you run `aiscope .`, it:

1. **Discovers** every memory file across Cursor, Claude Code, and GitHub
   Copilot — see [Tools and subsystems](../concepts/tools-and-subsystems.md).
2. **Parses** YAML frontmatter (`applyTo`, `globs`, `tools:`, …).
3. **Extracts** typed assertions (e.g. _"prefer snake_case for variables"_).
4. **Reasons** about pairs — checks scope overlap, polarity, severity.
5. **Renders** the result via your chosen output mode.

It never makes a network request. It never reads outside the repo unless you
pass `--user` (see [Privacy guard](../usage/privacy.md)).

## Output modes

| Flag             | Use it for                                     |
| ---------------- | ---------------------------------------------- |
| _(default)_      | Interactive ratatui TUI                        |
| `--text`         | Plain text — pipe-friendly                     |
| `--diag`         | Compiler-style diagnostics (miette)            |
| `--json`         | Machine-readable — for scripts and dashboards  |
| `--card out.png` | 1280×720 PNG summary — drop into a tweet or PR |

## Subcommands

| Command         | What it does                                           |
| --------------- | ------------------------------------------------------ |
| `aiscope`       | Scan + render (default `scan` subcommand)              |
| `aiscope check` | Scan + exit non-zero if HIGH conflicts found (CI gate) |
| `aiscope watch` | Re-scan on file change                                 |

## Reasoning modes

| Flag         | Behavior                                                          |
| ------------ | ----------------------------------------------------------------- |
| _(default)_  | **Uniform** — every cross-source pair is candidate. Max recall.   |
| `--specific` | **Specific** — uses the subsystem matrix to silence false alarms. |

See [Reasoning modes](../concepts/reasoning-modes.md) for the matrix.

## Next

- [Your first scan](./first-scan.md) — walk through real output
- [CLI reference](../usage/cli.md) — every flag, every option
