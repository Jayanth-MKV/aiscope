# GitHub Copilot

aiscope discovers Copilot memory across **4 subsystems** plus the
ecosystem-standard `AGENTS.md`.

## Files discovered

| Subsystem      | Glob                                      |
| -------------- | ----------------------------------------- |
| `Instructions` | `.github/copilot-instructions.md`         |
| `Instructions` | `.github/instructions/*.instructions.md`  |
| `Prompts`      | `.github/prompts/*.prompt.md`             |
| `Agents`       | `.github/agents/*.agent.md`               |
| `Agents`       | `**/AGENTS.md` _(any depth, path-scoped)_ |
| `ChatModes`    | `.github/chatmodes/*.chatmode.md`         |

## Frontmatter

```yaml
---
applyTo: "**/*.py"
description: "Python conventions"
---
```

| Field         | Used for                                            |
| ------------- | --------------------------------------------------- |
| `applyTo`     | Scope glob(s) — single string or array              |
| `description` | Optional human label                                |
| `name`        | Agent name (used for duplicate detection)           |
| `tools`       | Agent tool allowlist (used for `AgentToolMismatch`) |

## `AGENTS.md` path scoping

`apps/web/AGENTS.md` is automatically scoped to `apps/web/**`. So a
top-level `AGENTS.md` saying _"use camelCase"_ and an
`apps/api/AGENTS.md` saying _"use snake_case"_ won't conflict — their
scopes don't overlap.

## Tips

- Use **one** root `copilot-instructions.md` for cross-cutting rules.
- Use `.github/instructions/<lang>.instructions.md` with `applyTo` for
  language-specific rules.
- Move team-specific rules into `apps/<team>/AGENTS.md` to scope them.
