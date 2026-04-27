# CLI reference

```text
aiscope [OPTIONS] [PATH]
aiscope <SUBCOMMAND> [OPTIONS] [PATH]
```

`PATH` defaults to the current directory.

## Global options

| Flag            | Description                                                       |
| --------------- | ----------------------------------------------------------------- |
| `--text`        | Plain text output                                                 |
| `--diag`        | Compiler-style diagnostics (miette)                               |
| `--json`        | Machine-readable JSON                                             |
| `--card <FILE>` | Render a 1280×720 PNG summary card                                |
| `--specific`    | Use the [Specific reasoning mode](../concepts/reasoning-modes.md) |
| `--user`        | Also read user-scope memory (e.g. `~/.claude/CLAUDE.md`)          |
| `--version`     | Print version and exit                                            |
| `--help`        | Print help                                                        |

If none of `--text` / `--diag` / `--json` / `--card` is passed, aiscope
launches the [interactive TUI](./tui.md).

## Subcommands

### `aiscope scan` _(default)_

Scan and render. Same as `aiscope` with no subcommand.

```bash
aiscope scan --diag .
```

### `aiscope check`

Scan and exit non-zero if any HIGH-severity conflicts are found. Designed
for CI gates.

```bash
aiscope check --specific .
echo $?     # 0 = clean, 1 = HIGH conflicts, 2 = error
```

See [Exit codes](../ci/exit-codes.md).

### `aiscope watch`

Re-scan whenever a memory file changes. Great for live editing.

```bash
aiscope watch .
```

## Examples

```bash
# Interactive TUI on the current dir
aiscope

# CI-friendly: exit non-zero on HIGH conflicts, suppress false alarms
aiscope check --specific .

# Generate a PR-ready summary card
aiscope --card scan.png .

# Pipe JSON to jq
aiscope --json . | jq '.conflicts[] | select(.severity == "high")'

# Read user-scope memory too (opt-in)
aiscope --user .
```

## Output to stdout vs file

All renderers except `--card` write to **stdout** and respect piping. The
TUI auto-falls-back to plain text when stdout is not a TTY:

```bash
aiscope . > scan.txt    # writes plain text, not TUI escape codes
```
