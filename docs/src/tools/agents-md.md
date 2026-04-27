# `AGENTS.md`

The `AGENTS.md` file is a tool-agnostic convention emerging across the
ecosystem — Copilot, Cursor, Claude, and Codex all read it. aiscope treats
it as a first-class source.

## Discovery

aiscope finds `AGENTS.md` at **any depth** in the repo and assigns it to the
**Copilot Agents subsystem** by convention.

## Path scoping

The file's directory becomes its scope's `path_prefix`:

| File location            | Scope             |
| ------------------------ | ----------------- |
| `AGENTS.md`              | everywhere        |
| `apps/web/AGENTS.md`     | `apps/web/**`     |
| `services/api/AGENTS.md` | `services/api/**` |

This means a top-level `AGENTS.md` and a `apps/web/AGENTS.md` will not
conflict on naming rules unless both apply to overlapping paths — perfect
for monorepos.

## Why use it

If you want **one source of truth** that all your AI tools read, put it in
`AGENTS.md`. Most modern AI tools will pick it up automatically, and aiscope
will validate its consistency with everything else.

## Example

```markdown
# AGENTS.md

## Code style

- Use 2-space indentation
- Prefer single quotes in TypeScript
- Use snake_case for Python

## Testing

- Every PR must include tests for new public APIs
```

No frontmatter needed — the file applies to its directory and below.
