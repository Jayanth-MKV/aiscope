# Tools and subsystems

aiscope discovers files across **3 tools × 5 subsystems** = up to 15 distinct
source kinds.

| Subsystem      | What it is                                               |
| -------------- | -------------------------------------------------------- |
| `Instructions` | Always-on rules the model uses on every turn             |
| `Prompts`      | One-shot snippets the user invokes explicitly            |
| `Agents`       | Named subagents with their own tool allowlist            |
| `ChatModes`    | UI modes that switch the available tools / system prompt |
| `Skills`       | Reusable capability bundles (Claude-specific convention) |

## GitHub Copilot

| Subsystem      | Discovered from                                                             |
| -------------- | --------------------------------------------------------------------------- |
| `Instructions` | `.github/copilot-instructions.md`, `.github/instructions/*.instructions.md` |
| `Prompts`      | `.github/prompts/*.prompt.md`                                               |
| `Agents`       | `.github/agents/*.agent.md`, plus `AGENTS.md` at any depth                  |
| `ChatModes`    | `.github/chatmodes/*.chatmode.md`                                           |

`AGENTS.md` is automatically **path-scoped** — `apps/web/AGENTS.md` gets
`Scope.path_prefix = "apps/web/**"`.

## Cursor

| Subsystem      | Discovered from                               |
| -------------- | --------------------------------------------- |
| `Instructions` | `.cursorrules` (legacy), `.cursor/rules/*.md` |
| `Prompts`      | `.cursor/commands/*.md`                       |
| `Agents`       | `.cursor/agents/*.md`                         |
| `ChatModes`    | `.cursor/modes/*.md`                          |

## Claude Code

| Subsystem      | Discovered from                        |
| -------------- | -------------------------------------- |
| `Instructions` | `CLAUDE.md` at any depth (path-scoped) |
| `Prompts`      | `.claude/commands/*.md`                |
| `Agents`       | `.claude/agents/*.md`                  |
| `Skills`       | `.claude/skills/*/SKILL.md`            |

The opt-in `--user` flag also reads `~/.claude/CLAUDE.md` (your global
instructions). aiscope **never** reads `~/.claude/projects/` (transcript
history) — see [Privacy guard](../usage/privacy.md).

## Why subsystems matter

Different subsystems have different roles. A `Prompt` is a one-shot
override — it's _meant_ to contradict the always-on `Instructions`. An
`Agent` runs in its own context window — what it says shouldn't conflict
with what the main `Instructions` say.

That's why aiscope ships two reasoning modes: **Uniform** (default, max
recall) and **Specific** (`--specific`, subsystem-aware). See
[Reasoning modes](./reasoning-modes.md).
