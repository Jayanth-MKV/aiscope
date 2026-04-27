# Interactive TUI

Run `aiscope` with no output flags to launch the ratatui interface.

```bash
aiscope .
```

## Layout

Three panes:

- **Sources** (left) — every memory file aiscope discovered, color-coded by tool.
- **Unified context** (right) — every rule across every file, with conflicts
  flagged inline. Each rule shows its `applyTo` glob in dim gray on the right.
- **Score** (bottom) — totals: rules, clashes, duplicates, tokens, waste %.

## Keys

| Key             | Action                       |
| --------------- | ---------------------------- |
| `q` / `Esc`     | Quit                         |
| `c`             | Toggle conflicts-only filter |
| `↑` / `↓`       | Scroll one row               |
| `j` / `k`       | Same as ↓ / ↑ (vim-style)    |
| `PgUp` / `PgDn` | Page scroll                  |

## Tool icons

| Icon | Tool        |
| ---- | ----------- |
| `▲`  | Copilot     |
| `○`  | Cursor      |
| `◆`  | Claude Code |

## Falling back to text

If stdout is not a TTY (you piped or redirected), aiscope automatically prints
plain text instead of TUI escape codes. So `aiscope . > scan.txt` works
naturally.
