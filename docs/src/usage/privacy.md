# Privacy guard

aiscope is a **read-only, offline tool** by design. Three guarantees:

## 1. Zero network

aiscope makes **no network requests**. Ever. No telemetry, no update checks,
no crash reporting. You can verify with `strace`, Wireshark, or by reading
the source — there are no `reqwest`, `ureq`, or `hyper` dependencies in the
`[dependencies]` block of `Cargo.toml`.

## 2. Zero writes (unless you pass `--card`)

aiscope opens every file as **read-only**. The only file it ever writes is
the PNG you ask for via `--card path.png`.

## 3. Scope respect

By default, aiscope only reads files **inside your repo**.

- It does **not** read `~/.claude/CLAUDE.md` unless you pass `--user`.
- It **never** reads `~/.claude/projects/` (your transcript history) — even
  with `--user`. That directory is on the explicit deny-list.
- It does **not** follow symlinks out of the repo.
- It respects `.gitignore` — files ignored by git are skipped.

## What about secrets?

Memory files are markdown — they're meant to be committed and shared. aiscope
prints the **content of those files** as part of diagnostics. If you put
secrets in `copilot-instructions.md`, they'll appear in `aiscope --diag`
output. Don't do that.

## Want even tighter sandboxing?

```bash
firejail --net=none --read-only=/ --read-only=/home/$USER aiscope check .
```

Or run inside a Docker container with `--network=none`.

## See it for yourself

```bash
strace -e trace=network aiscope check .
# (no output — no network syscalls)
```
