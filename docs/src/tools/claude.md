# Claude Code

aiscope discovers Claude Code memory across **5 subsystems**, including
nested skills.

## Files discovered

| Subsystem      | Glob                                         |
| -------------- | -------------------------------------------- |
| `Instructions` | `**/CLAUDE.md` _(any depth, path-scoped)_    |
| `Instructions` | `~/.claude/CLAUDE.md` _(only with `--user`)_ |
| `Prompts`      | `.claude/commands/*.md`                      |
| `Agents`       | `.claude/agents/*.md`                        |
| `Skills`       | `.claude/skills/*/SKILL.md`                  |

## What aiscope deliberately ignores

- `~/.claude/projects/` — your transcript history. **Never** read, even
  with `--user`.
- Anything outside the repo (unless `--user` is passed and the path is
  exactly `~/.claude/CLAUDE.md`).

See [Privacy guard](../usage/privacy.md).

## Frontmatter

```yaml
---
name: reviewer
description: "Reviews code for Python style"
tools:
  - read
  - search
model: sonnet-4
---
```

| Field         | Used for                                            |
| ------------- | --------------------------------------------------- |
| `name`        | Used for duplicate-name detection across agents     |
| `description` | Optional human label                                |
| `tools`       | Agent tool allowlist (used for `AgentToolMismatch`) |
| `model`       | Optional model hint                                 |

## `CLAUDE.md` path scoping

Just like `AGENTS.md` for Copilot — a `CLAUDE.md` in `apps/api/` is scoped
to `apps/api/**` automatically.

## `--user` opt-in

```bash
aiscope --user .
```

Reads your global Claude memory at `~/.claude/CLAUDE.md` so cross-project
rules show up too. Strict allowlist — no other files outside the repo are
read.
