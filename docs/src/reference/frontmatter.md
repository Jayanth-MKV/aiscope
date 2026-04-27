# Frontmatter fields

Every supported field across every tool, in one table.

| Field         | Tools                       | Type            | Used for                            |
| ------------- | --------------------------- | --------------- | ----------------------------------- |
| `applyTo`     | Copilot                     | string \| array | Scope glob                          |
| `globs`       | Cursor                      | array           | Scope glob                          |
| `alwaysApply` | Cursor                      | bool            | Force "everywhere" scope            |
| `name`        | All (agents, skills, modes) | string          | Duplicate-name detection            |
| `description` | All                         | string          | Human label (shown in TUI/card)     |
| `tools`       | Copilot, Claude (agents)    | array           | Agent tool allowlist                |
| `model`       | Copilot, Claude (agents)    | string          | Optional model hint (informational) |

## Parsing rules

- aiscope uses [`serde_yaml`](https://docs.rs/serde_yaml) under the hood.
- Unknown fields are **ignored**, not errored — forward-compatible with
  whatever Copilot/Cursor/Claude add next.
- Missing fields fall back to defaults (no scope = everywhere).
- Both `---` and `+++` (TOML) delimiters are recognized.

## Glob syntax

aiscope uses [`globset`](https://docs.rs/globset) — same syntax as `git`:

| Pattern         | Matches                          |
| --------------- | -------------------------------- |
| `*.py`          | any `.py` in the current dir     |
| `**/*.py`       | any `.py` at any depth           |
| `apps/web/**`   | everything under `apps/web/`     |
| `!**/test_*.py` | exclusions are not yet supported |

## Examples

```yaml
---
applyTo: "**/*.py" # Copilot
---
```

```yaml
---
globs: # Cursor
  - "**/*.ts"
  - "**/*.tsx"
alwaysApply: false
---
```

```yaml
---
name: reviewer # Claude / Copilot agent
description: "Reviews PRs"
tools: [read, search]
model: sonnet-4
---
```
