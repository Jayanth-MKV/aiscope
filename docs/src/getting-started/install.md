# Install

aiscope is a single Rust binary with **zero runtime dependencies**.

## From source (recommended for now)

```bash
cargo install --git https://github.com/Jayanth-MKV/aiscope
```

Cargo drops the `aiscope` binary into `~/.cargo/bin/`, which is on your PATH
if you installed Rust via [rustup](https://rustup.rs).

Verify:

```bash
aiscope --version
```

## Pre-built binaries

Each [GitHub Release](https://github.com/Jayanth-MKV/aiscope/releases) ships
pre-built binaries for:

| OS      | Architecture            |
| ------- | ----------------------- |
| Linux   | x86_64                  |
| Linux   | aarch64                 |
| macOS   | x86_64                  |
| macOS   | aarch64 (Apple Silicon) |
| Windows | x86_64                  |

Download the archive for your platform, extract, and put `aiscope` somewhere
on your PATH.

Each archive ships with a `.sha256` file — verify before running:

```bash
sha256sum -c aiscope-v0.1.0-x86_64-unknown-linux-gnu.tar.gz.sha256
```

## crates.io

Once published, you'll be able to:

```bash
cargo install aiscope
```

## Build requirements

Only needed if you're building from source:

- Rust **1.85+** (edition 2024)
- A C linker (cc on macOS / Linux, MSVC on Windows)

That's it. No Node, no Python, no native libs.

## Uninstall

```bash
cargo uninstall aiscope
```
