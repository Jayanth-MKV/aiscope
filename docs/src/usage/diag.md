# Compiler-style diagnostics (`--diag`)

```bash
aiscope --diag .
```

Renders every conflict as a [miette](https://github.com/zkat/miette)
diagnostic with source spans — same look as `cargo check`.

## Sample

```text
× camelCase disagrees with snake_case
   ╭─[.github/copilot-instructions.md:5:1]
 4 │
 5 │ - Use **camelCase** for variables and functions.
   · ─────────────────────────────────────
 6 │
   ╰────
  help: the other side: .github/instructions/python.instructions.md:7:
        "Use snake_case for variables."
```

## Why use it

- It's the format **engineers already read** — every Rust dev recognizes it.
- Source spans take you straight to the offending line.
- Works in PR comments — looks great pasted into GitHub markdown.

## Combine with `aiscope check`

```bash
aiscope check --specific --diag .
```

Get diagnostic-style output **and** the non-zero exit code in one command —
ideal for `pre-commit` and CI logs.
