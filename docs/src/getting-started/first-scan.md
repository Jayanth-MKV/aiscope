# Your first scan

Let's walk through a realistic Copilot-only repo and look at what aiscope
finds. No setup beyond `cargo install`.

## The repo

```
my-repo/
└── .github/
    ├── copilot-instructions.md
    ├── instructions/
    │   ├── python.instructions.md      ← applyTo: **/*.py
    │   └── typescript.instructions.md  ← applyTo: **/*.ts
    ├── prompts/
    │   └── unit-test.prompt.md
    └── agents/
        └── reviewer.agent.md           ← tools: [read, search]
```

Plus an `apps/web/AGENTS.md` with the web-team's conventions.

## Run it

```bash
aiscope --diag .
```

Sample output:

```text
aiscope · 6 sources · 18 statements · 159 tokens · 14 conflicts (4 high)

─── conflict 1 ───
  × agent tool mismatch: .github/instructions/python.instructions.md says
    "use the bash tool" but agent reviewer excludes it
   ╭─[.github/agents/reviewer.agent.md:8:1]
 8 │ You are a code reviewer. Use snake_case in Python feedback…
   · ─────────────────────────────────────
   ╰────
  help: add the tool to the agent's `tools:` allowlist, or change the instruction

─── conflict 2 ───
  × camelCase disagrees with snake_case
   ╭─[.github/copilot-instructions.md:5:1]
 5 │ - Use **camelCase** for variables and functions.
   · ─────────────────────────────────────
   ╰────
  help: the other side: .github/instructions/python.instructions.md:7: …
```

## What just happened

- aiscope discovered **all 5 sources** in the Copilot-only repo plus the
  path-scoped `apps/web/AGENTS.md`.
- It noticed the **agent's `tools:` allowlist** excludes `bash` but an
  instruction says to use it — flagged as `AgentToolMismatch` (HIGH).
- The **root `copilot-instructions.md`** has no `applyTo`, so it applies
  everywhere — and overlaps with `python.instructions.md` (which scopes to
  `**/*.py`). Their `camelCase` vs `snake_case` clash is HIGH.
- `python.instructions.md` and `typescript.instructions.md` also disagree on
  naming, but their globs **don't overlap** (`**/*.py` vs `**/*.ts`) — so
  that pair is demoted to **Low** with a `(scopes don't overlap)` note.

That last point is what makes aiscope worth running: it doesn't just shout
about every cross-file disagreement — it filters out false alarms based on
**actual scope overlap**.

## Try the TUI

```bash
aiscope .
```

Press `c` to filter to conflicts only. Press `q` to quit.

## Try the summary card

```bash
aiscope --card scan.png .
```

Drop `scan.png` into your PR description.

## Where to next

- [Conflict kinds](../concepts/conflict-kinds.md) — what each diagnostic means
- [Scope and applyTo](../concepts/scope.md) — the false-alarm filter
- [GitHub Actions](../ci/github-actions.md) — gate every PR with `aiscope check`
