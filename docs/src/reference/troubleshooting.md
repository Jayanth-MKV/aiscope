# Troubleshooting

## "command not found: aiscope"

Cargo's bin directory isn't on your `PATH`. Add it:

- **Linux / macOS**: add `export PATH="$HOME/.cargo/bin:$PATH"` to your
  shell rc file.
- **Windows (PowerShell)**: `$env:Path += ";$env:USERPROFILE\.cargo\bin"`,
  or update your user `Path` env var permanently via System Settings.

## "no AI memory files found"

Run from your repo root, not a subdirectory. Or pass the path explicitly:

```bash
aiscope /path/to/repo
```

If your repo legitimately has no memory files, that's the expected output.

## TUI looks broken / shows escape codes

Your terminal doesn't fully support the cursor protocol. Use `--text`:

```bash
aiscope --text .
```

## `aiscope check` always passes even with conflicts

`check` only fails on **High**-severity conflicts. If everything is **Low**
(typically because scopes don't overlap), exit code is `0`. Use `--text` or
`--diag` to see Low items.

## Card overflows / text is clipped

Please [open an issue](https://github.com/Jayanth-MKV/aiscope/issues/new/choose)
with the input that triggered it. As of v0.1.0 the card uses pixel-accurate
text measurement and binary-search truncation — overflow should be
impossible.

## Build fails: "cosmic-text requires Rust ≥ 1.85"

Upgrade your toolchain:

```bash
rustup update stable
```

## "permission denied" reading some file

aiscope only reads what your user can read. Check file permissions:

```bash
ls -la .github/
```

## Everything else

[Open an issue](https://github.com/Jayanth-MKV/aiscope/issues/new/choose) —
include OS, Rust version, `aiscope --version`, and the output of
`aiscope --diag .` if relevant.
