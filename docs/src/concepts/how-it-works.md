# How aiscope thinks

aiscope is a **6-layer deterministic pipeline**. Each layer is a pure function
of the previous one's output — same input always produces the same diagnostics
in the same order. No network. No randomness. No background jobs.

```text
scanner → frontmatter → md-parse → canon → extract → reason → render
```

## Layer 1 — `scanner`

Walks the repo finding every supported memory file. One scanner per tool:

- `scanner::copilot` — `.github/copilot-instructions.md`,
  `.github/{instructions,prompts,agents,chatmodes}/`, plus `AGENTS.md`
  at any depth (path-scoped).
- `scanner::cursor` — `.cursorrules`, `.cursor/{rules,commands,agents,modes}/`.
- `scanner::claude` — `CLAUDE.md` at any depth (path-scoped),
  `.claude/{agents,commands,skills/*/SKILL.md}`, plus opt-in
  `~/.claude/CLAUDE.md` via `--user`.

A privacy guard ensures the scanner **never** reads outside the repo unless
`--user` is passed, and **never** reads transcript history under
`~/.claude/projects/`.

## Layer 2 — `frontmatter`

Parses the YAML subset used by every memory-file ecosystem (`applyTo`,
`globs`, `alwaysApply`, `tools:`, `model:`, `name:`, `description:`).

The output is a typed `Scope` containing globs, path prefix, model, and
tool allowlist — used later by the reasoner.

## Layer 3 — `md-parse`

Converts each markdown body into a stream of typed `Statement`s (bullets,
headings, paragraphs), preserving line numbers and byte offsets so we can
later render compiler-style diagnostics with source spans.

## Layer 4 — `canon`

Normalizes each statement: NFKC, smart-quote/dash collapse, light stemming.
This is what makes paraphrase detection deterministic without using ML.

## Layer 5 — `extract`

Pattern-based assertion extraction onto typed axes:

- `Naming(Variables | Functions | Types | Files)`
- `Indentation(Tabs | Spaces2 | Spaces4 | Spaces8)`
- `QuoteStyle(Single | Double)`
- `PackageManager(Npm | Yarn | Pnpm | Bun)`
- … and more

## Layer 6 — `reason`

Pair-wise checks. Two assertions become a conflict only when:

1. The subsystem matrix permits it (in `--specific` mode).
2. The scopes overlap (otherwise demoted to Low).
3. The polarity disagrees on the same axis.

Plus duplicate-name detection across agents/skills/chat modes, plus the
agent-tool-allowlist mismatch detector.

## Render

Five renderers, all consuming the same typed `ContextBundle`:

- ratatui TUI
- miette compiler-style diagnostics
- plain text
- JSON
- 1280×720 PNG card with embedded JetBrains Mono

## Why this shape

Each layer is **independently testable**. The 42-test suite covers every
layer in isolation plus end-to-end snapshots. Determinism + clean layering
means you can rely on `aiscope check` in CI as a stable gate — it'll never
flake on you.
