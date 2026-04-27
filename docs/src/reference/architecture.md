# Architecture

aiscope is a 6-layer deterministic pipeline. This page maps the layers to
the actual source modules.

```text
┌─────────┐  ┌─────────────┐  ┌──────────┐  ┌───────┐  ┌─────────┐  ┌────────┐  ┌────────┐
│ scanner │→ │ frontmatter │→ │ md-parse │→ │ canon │→ │ extract │→ │ reason │→ │ render │
└─────────┘  └─────────────┘  └──────────┘  └───────┘  └─────────┘  └────────┘  └────────┘
```

## Module layout

```text
src/
├── main.rs                  # entry, CLI dispatch
├── cli.rs                   # clap arg model
├── lib.rs                   # public API, ContextBundle
├── types.rs                 # Source, Statement, Assertion, Conflict, Scope
├── scanner/
│   ├── mod.rs               # discover() entry point
│   ├── common.rs            # walkdir helpers, gitignore filter
│   ├── copilot.rs           # .github/* + AGENTS.md
│   ├── cursor.rs            # .cursor/* + .cursorrules
│   └── claude.rs            # CLAUDE.md + .claude/* + --user
├── frontmatter/
│   └── mod.rs               # YAML/TOML parser → Scope
├── parse.rs                 # markdown → Statement[]
├── canon.rs                 # NFKC + light stemming
├── extract/
│   ├── mod.rs
│   └── pattern.rs           # regex assertion extractors
├── reason/
│   ├── mod.rs
│   ├── matrix.rs            # Specific-mode subsystem table
│   ├── overlap.rs           # globset-based scope overlap
│   └── tools.rs             # AgentToolMismatch detector
└── render/
    ├── tui.rs               # ratatui interactive
    ├── text.rs              # plain text
    ├── diag.rs              # miette
    ├── json.rs              # serde_json
    └── card.rs              # tiny-skia + cosmic-text PNG
```

## Key types

```rust
pub struct ContextBundle {
    pub root: PathBuf,
    pub sources: Vec<Source>,
    pub statements: Vec<Statement>,
    pub assertions: Vec<Assertion>,
    pub conflicts: Vec<Conflict>,
    pub total_tokens: usize,
    pub stale_tokens: usize,
}
```

Every renderer takes `&ContextBundle`. Adding a new output format means one
new module — nothing else changes.

## Determinism

- `walkdir` results are sorted before consumption.
- `HashMap` is never iterated for output — we use `BTreeMap` or sort first.
- No threads, no async runtime, no clock reads in the hot path.

Same input → byte-identical JSON, byte-identical card PNG, identical
diagnostic line order. This is what makes `aiscope check` safe to wire into
CI.

## Test suite

42 tests across:

- per-module unit tests (frontmatter, canon, extract, overlap, matrix)
- end-to-end fixtures (`tests/fixtures/`) with golden snapshots
- copilot-only fixture proving single-tool repos work
- privacy-guard tests asserting `~/.claude/projects/` is never read

Run `cargo test` to see.
