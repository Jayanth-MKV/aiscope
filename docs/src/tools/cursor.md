# Cursor

aiscope discovers Cursor memory across **4 subsystems**, including the legacy
single-file format.

## Files discovered

| Subsystem      | Glob                                   |
| -------------- | -------------------------------------- |
| `Instructions` | `.cursorrules` _(legacy, single file)_ |
| `Instructions` | `.cursor/rules/*.md`                   |
| `Prompts`      | `.cursor/commands/*.md`                |
| `Agents`       | `.cursor/agents/*.md`                  |
| `ChatModes`    | `.cursor/modes/*.md`                   |

## Frontmatter

```yaml
---
globs:
  - "**/*.ts"
  - "**/*.tsx"
alwaysApply: false
description: "TypeScript style"
---
```

| Field         | Used for                       |
| ------------- | ------------------------------ |
| `globs`       | Scope glob(s) — array          |
| `alwaysApply` | If true, scope is "everywhere" |
| `description` | Optional human label           |

## Migration tip

If you have a legacy `.cursorrules` plus modular `.cursor/rules/*.md`,
aiscope will surface both — and likely flag duplicates. Migrate fully to
`.cursor/rules/` and delete `.cursorrules` to clean up.
