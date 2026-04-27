# Conflict kinds

Every diagnostic aiscope emits has a `ConflictKind`. There are four.

## `Clash`

Two assertions on the same axis (e.g. `Naming(Variables)`) disagree.

```text
× camelCase disagrees with snake_case
```

**When it fires**: any two files that assert different values on the same
axis, where their scopes overlap.

**How to fix**: pick one. Update or delete the loser.

## `PolarityConflict`

One file asserts X, another forbids X — explicit polarity disagreement.

```text
× polarity conflict: "use 4 spaces" vs "do not use 4 spaces"
```

**When it fires**: detected via negation patterns in the extractor.

**How to fix**: same as Clash — pick a side.

## `Duplicate`

Two files say the same thing (after canonicalization).

```text
× duplicate rule across .cursorrules and CLAUDE.md
   help: wastes tokens; remove one.
```

**When it fires**: after NFKC normalization + light stemming, two statements
match.

**How to fix**: delete one. Or, if you have multiple tools that intentionally
repeat (e.g. you maintain Cursor _and_ Claude rules in parallel), suppress
this with `aiscope --no-duplicates` _(planned)_.

`Duplicate` also fires on **duplicate `name:`** across agents, skills, or
chat modes — two agents named `reviewer` is undefined behavior.

## `AgentToolMismatch`

An agent's `tools:` allowlist either:

- **Excludes a tool that an instruction says to use**:
  > `python.instructions.md` says _"use the bash tool"_ but agent `reviewer`
  > excludes it
- Or is **empty while the agent body mentions tools** — undefined behavior.

```text
× agent tool mismatch: .github/instructions/python.instructions.md says
  "use the bash tool" but agent reviewer excludes it
   help: add the tool to the agent's `tools:` allowlist, or change the instruction
```

**When it fires**: cross-checked between every `Subsystem::Agents` source's
`scope.tools` and every `Subsystem::Instructions` source's body text.

**How to fix**: either add the tool to the agent's allowlist, or change the
instruction to not require it.

`AgentToolMismatch` is **always reported** — `--specific` mode does not
silence it.

## Severity

Every conflict has a severity:

| Severity | What it means                                                                                   |
| -------- | ----------------------------------------------------------------------------------------------- |
| `High`   | Cross-source AND scopes overlap AND combined-confidence ≥ 0.85. `aiscope check` exits non-zero. |
| `Low`    | Same source, OR scopes don't overlap, OR low confidence. Informational.                         |

See [Scope](./scope.md) for the overlap rules.
