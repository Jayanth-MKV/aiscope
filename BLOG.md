# aiscope: see what your AI tools actually believe about your code

_A 5MB Rust CLI that diffs Cursor / Claude Code / GitHub Copilot context files and tells you exactly where they disagree._

## The problem nobody owns

You have `.cursorrules`. You have `.github/copilot-instructions.md`. You have `CLAUDE.md`. You have `.github/instructions/python.instructions.md`. You have `.github/agents/reviewer.agent.md`. Maybe a `.claude/skills/` folder. Maybe an `apps/web/AGENTS.md` because the web team wanted their own conventions.

Every one of them is a markdown file. Every one of them is silently shoved into the model's context window. None of them know about each other.

So you end up shipping code where:

- One file says **"use snake_case"**, another says **"use camelCase"**.
- The agent allowlist says `tools: [read, search]` but your instructions say _"use the bash tool to run pytest"_.
- Two files repeat the exact same sentence in slightly different words — burning tokens to tell the model the same thing twice.
- A `python.instructions.md` (applyTo: `**/*.py`) and a `typescript.instructions.md` (applyTo: `**/*.ts`) look like a contradiction but actually never apply to the same file. False alarm.

You have no tool that knows the difference. Until now.

## What aiscope does

`aiscope` is a single Rust binary that:

1. **Discovers** every memory file across all 3 major coding-AI tools — Cursor, Claude Code, GitHub Copilot — and across all 5 subsystems each of them supports: instructions, prompts, agents, chat modes, skills.
2. **Parses** the YAML frontmatter (`applyTo`, `globs`, `alwaysApply`, `tools:`, `model:`) so it understands _scope_, not just text.
3. **Extracts** assertions on canonical axes: indentation style, naming convention, quote style, package manager, etc.
4. **Reasons** about pairs: do they overlap in scope? Do they contradict? Is this a real high-severity clash, or just two non-overlapping rules that happen to disagree?
5. **Renders** as a ratatui-style TUI, a `miette` compiler diagnostic, JSON, or a 1280×720 PNG card you can drop into a PR or tweet.

```
$ aiscope .
aiscope · my-repo · 8% wasted
6 sources · 18 rules · 14 clashes · 1 duplicates · 159 tokens

  [HIGH agent]  .github/agents/reviewer.agent.md ⇄ .github/instructions/python.instructions.md
                says "use the bash tool" but agent excludes it
  [HIGH clash]  .github/copilot-instructions.md ⇄ apps/web/AGENTS.md
                4 spaces disagrees with 2 spaces
  [HIGH clash]  .github/copilot-instructions.md ⇄ .github/instructions/python.instructions.md
                camelCase disagrees with snake_case
  +12 more...
```

## Two reasoning modes

Because not everyone agrees on whether prompts and instructions _should_ contradict (a prompt is one-shot; an instruction is permanent), aiscope ships two modes:

- **Uniform** (default) — every cross-source pair is a candidate. Maximum recall.
- **Specific** (`--specific`) — uses a subsystem-clash matrix:
  - Prompts ↔ Instructions: never conflict (prompts are intentional overrides).
  - Agents ↔ everything else: never conflict (agents have their own context).
  - Skills / ChatModes ↔ anything: never conflict.

Pick whichever matches how your team actually uses the files.

## Scope-aware severity

The killer feature: aiscope reads the `applyTo` glob and uses [`globset`](https://docs.rs/globset) to compute whether two scopes overlap.

- `applyTo: **/*.py` and `applyTo: **/*.ts` — **don't overlap** → severity demoted to Low with a `(scopes don't overlap)` note. It's not actually a clash.
- `applyTo: **/*.py` and root `copilot-instructions.md` (no applyTo, applies everywhere) — **overlap** → High severity. This _will_ hit the model at the same time and confuse it.

Same for `apps/web/AGENTS.md` — automatically scoped to `apps/web/**` because that's where it lives.

## The Copilot-only case (one tool, all subsystems)

If your repo only has `.github/`, aiscope still earns its keep. A typical Copilot setup has:

```
.github/
├── copilot-instructions.md        ← applies everywhere
├── instructions/
│   ├── python.instructions.md     ← applyTo: **/*.py
│   └── typescript.instructions.md ← applyTo: **/*.ts
├── prompts/
│   └── unit-test.prompt.md        ← one-shot
└── agents/
    └── reviewer.agent.md          ← tools: [read, search]
apps/web/AGENTS.md                 ← path-scoped to apps/web/**
```

aiscope discovers all 6, parses every frontmatter field, and tells you which conflicts actually matter. It treats `AGENTS.md` as path-scoped automatically — a convention that _should_ be obvious but no other tool does this for you.

## Architecture (for the curious)

A 6-layer deterministic pipeline, each layer pure and testable:

```
scanner → frontmatter → md-parse → canon → extract → reason → render
```

- **scanner** — WalkDir-based discovery per tool, with privacy guard (no reads outside the repo unless `--user` is passed, never CLAUDE.md transcripts)
- **frontmatter** — hand-rolled YAML subset parser (no serde_yaml, no libyaml dep)
- **canon** — NFKC + smart-quote/dash normalization + light stemming
- **extract** — pattern-based assertion extraction onto typed axes (`Naming(Variables)`, `Indentation`, `QuoteStyle`, …)
- **reason** — pair-wise checks with subsystem matrix + scope overlap + severity demotion + duplicate-name detection across agents/skills/chatmodes
- **render** — ratatui TUI, miette diagnostics, JSON, or PNG card with embedded JetBrains Mono

42 tests. Clippy `-D warnings` clean. ~5 MB release binary.

## Install & try

```bash
cargo install --git https://github.com/Jayanth-MKV/aiscope
cd your-repo
aiscope .                # interactive TUI
aiscope --text .         # plain text
aiscope --diag .         # compiler-style diagnostics
aiscope --json .         # for CI / scripts
aiscope --card out.png . # 1280×720 PNG summary
aiscope check .          # exits non-zero if HIGH conflicts found (CI gate)
```

CI integration:

```yaml
- run: cargo install aiscope
- run: aiscope check --specific .
```

That's it. Try it on your repo. You will probably find a conflict. We did.

— [github.com/Jayanth-MKV/aiscope](https://github.com/Jayanth-MKV/aiscope)
