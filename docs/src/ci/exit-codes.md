# Exit codes

| Code | Meaning                                                          |
| ---- | ---------------------------------------------------------------- |
| `0`  | Success — no HIGH-severity conflicts                             |
| `1`  | One or more HIGH-severity conflicts found (`aiscope check` only) |
| `2`  | Argument or I/O error                                            |

## Behavior by command

| Command         | Exits non-zero on HIGH conflicts? |
| --------------- | --------------------------------- |
| `aiscope`       | ❌ no — informational only        |
| `aiscope scan`  | ❌ no — same as above             |
| `aiscope check` | ✅ yes — exits `1`                |
| `aiscope watch` | ❌ no — runs forever              |

## Why `scan` doesn't fail

`scan` is for inspection (TUI, JSON, card). Failing it would break dashboards
and watch loops. Use `check` whenever you want a hard gate.

## CI snippet

```yaml
- run: aiscope check --specific .
  # exit 1 fails the job; no extra config needed
```
