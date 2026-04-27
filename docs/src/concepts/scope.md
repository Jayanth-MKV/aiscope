# Scope and applyTo

The single most important feature of aiscope is **scope-aware severity**. It's
the difference between a tool that flags every cross-file disagreement and a
tool you actually trust to gate your CI.

## What is a scope?

Every memory file has a scope — _which files does this rule apply to?_
aiscope derives it from three sources:

1. **`applyTo` / `globs` frontmatter** — explicit glob pattern(s).

   ```yaml
   ---
   applyTo: "**/*.py"
   ---
   ```

2. **`alwaysApply: true`** — explicit "applies everywhere".

3. **File location** — `apps/web/AGENTS.md` is implicitly scoped to
   `apps/web/**` because that's where it lives.

If none of these are present, the file applies **everywhere**.

## When do scopes overlap?

Two scopes overlap if **any path** matches both. aiscope computes this with
the [`globset`](https://docs.rs/globset) crate.

| Left scope    | Right scope    | Overlap? |
| ------------- | -------------- | -------- |
| `**/*.py`     | `**/*.ts`      | ❌ no    |
| `**/*.py`     | _(everywhere)_ | ✅ yes   |
| `apps/web/**` | `apps/api/**`  | ❌ no    |
| `apps/web/**` | `**/*.ts`      | ✅ yes   |
| `apps/**`     | `apps/web/**`  | ✅ yes   |

## How it affects severity

If two files contradict each other but their scopes **don't overlap**,
they can never both apply to the same source file — so it's not really a
conflict. aiscope demotes such pairs to **Low** severity with a
`(scopes don't overlap)` note.

This is what makes `aiscope check` safe to run in CI: it won't fail your
build over a `python.instructions.md` vs `typescript.instructions.md`
disagreement that can never actually confuse the model.

## Tool-allowlist scope

For agents specifically, `tools:` defines which tools the agent is allowed
to invoke:

```yaml
---
name: reviewer
tools:
  - read
  - search
---
```

If an instruction file says _"use the bash tool to run pytest"_ but no
agent has `bash` in its allowlist, aiscope flags it as
[`AgentToolMismatch`](./conflict-kinds.md).

## See also

- [Reasoning modes](./reasoning-modes.md) — the orthogonal axis to scope
- [Frontmatter fields](../reference/frontmatter.md) — every supported field
