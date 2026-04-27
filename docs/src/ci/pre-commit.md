# Pre-commit hook

Catch conflicts before they ever reach a PR.

## Using [pre-commit](https://pre-commit.com/)

`.pre-commit-config.yaml`:

```yaml
repos:
  - repo: local
    hooks:
      - id: aiscope
        name: aiscope (AI memory consistency)
        entry: aiscope check --specific .
        language: system
        pass_filenames: false
        files: '\.(md|mdx)$'
```

Then:

```bash
pre-commit install
```

The hook runs `aiscope check` whenever you change any markdown file.

## Plain `.git/hooks/pre-commit`

If you don't want pre-commit:

```bash
#!/usr/bin/env bash
set -euo pipefail
exec aiscope check --specific .
```

Save as `.git/hooks/pre-commit` and `chmod +x` it.

## Speed

aiscope is sub-second on typical repos — the hook is essentially free.
