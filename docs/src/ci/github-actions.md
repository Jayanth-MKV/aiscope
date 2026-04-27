# GitHub Actions

Gate every PR with `aiscope check`. Add this workflow to your repo:

`.github/workflows/aiscope.yml`:

```yaml
name: aiscope

on:
  pull_request:
  push:
    branches: [main]

jobs:
  check:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - uses: dtolnay/rust-toolchain@stable

      - name: Cache cargo
        uses: Swatinem/rust-cache@v2

      - name: Install aiscope
        run: cargo install --locked aiscope

      - name: Check AI memory consistency
        run: aiscope check --specific --diag .
```

## Output in PR logs

Because `--diag` produces compiler-style output, the diagnostics render
beautifully in the GitHub Actions log viewer. Reviewers can read and act on
them without leaving the PR.

## Failing fast

`aiscope check` exits non-zero on any HIGH-severity conflict. The job fails,
the PR turns red, and the merge button is blocked (if you require checks to
pass).

## Generating a PR comment with the card

Combine with [stefanzweifel/git-auto-commit-action](https://github.com/stefanzweifel/git-auto-commit-action)
or a custom comment step:

```yaml
- name: Render summary card
  run: aiscope --card aiscope.png .

- name: Upload card
  uses: actions/upload-artifact@v4
  with:
    name: aiscope-card
    path: aiscope.png
```

## Speed

aiscope is **fast** — sub-second on typical repos, single-digit seconds on
huge monorepos. Adding it to CI adds negligible latency.
