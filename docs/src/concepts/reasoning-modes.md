# Reasoning modes

aiscope ships **two reasoning modes**. They control which subsystem pairs are
even considered for clash detection.

## Uniform (default)

Every cross-source pair is a candidate. Maximum recall.

```bash
aiscope .
aiscope --diag .
```

Use Uniform when:

- You want to catch **everything** that might confuse the model.
- You have a small, simple repo with one tool and few files.
- You're doing the first audit of a new repo.

## Specific (`--specific`)

Subsystem-aware filtering via this matrix:

| Left subsystem | Right subsystem | Can clash? |
| -------------- | --------------- | ---------- |
| `Instructions` | `Instructions`  | ✅ yes     |
| `Instructions` | `Prompts`       | ❌ no      |
| `Instructions` | `Agents`        | ❌ no      |
| `Instructions` | `ChatModes`     | ❌ no      |
| `Instructions` | `Skills`        | ❌ no      |
| `Prompts`      | `Prompts`       | ✅ yes     |
| `Agents`       | `Agents`        | ✅ yes     |
| `ChatModes`    | `ChatModes`     | ✅ yes     |
| `Skills`       | `Skills`        | ✅ yes     |

```bash
aiscope --specific .
aiscope check --specific .
```

### Why these rules?

- **Prompts ↔ Instructions**: prompts are intentional one-shot overrides.
  They _should_ contradict the always-on instructions when the user invokes
  them.
- **Agents ↔ everything**: agents run with their own context window. What an
  agent says doesn't reach the main session.
- **Skills / ChatModes ↔ anything else**: these are also opt-in contexts.

## Which should I use in CI?

Use **`--specific`** in CI:

```yaml
- run: aiscope check --specific .
```

It's the right default for most teams — fewer false positives, still catches
the conflicts that actually break model behavior.

Use **default Uniform** locally when you want to see _everything_.

## Note: AgentToolMismatch is unaffected

The `--specific` flag only filters cross-subsystem clashes. It does **not**
silence `AgentToolMismatch` diagnostics — those are always reported, because
they're always real bugs.
