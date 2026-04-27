# JSON output

```bash
aiscope --json . > scan.json
```

The full bundle as one JSON object — suitable for scripts, dashboards, and
custom integrations.

## Schema (top-level)

```json
{
  "root": "/path/to/repo",
  "sources": [
    {
      "tool": "copilot",
      "subsystem": "instructions",
      "path": ".github/copilot-instructions.md",
      "label": ".github/copilot-instructions.md",
      "name": null,
      "description": null,
      "scope": {
        "globs": [],
        "always_apply": false,
        "path_prefix": null,
        "model": null,
        "tools": []
      }
    }
  ],
  "statements": [
    {
      "source_index": 0,
      "text": "Use camelCase for variables and functions.",
      "line": 5,
      "byte_start": 87,
      "byte_end": 130
    }
  ],
  "rules": [
    /* same as statements (legacy alias) */
  ],
  "assertions": [
    {
      "statement_index": 0,
      "axis": { "kind": "naming", "scope": "variables" },
      "value": "camel_case",
      "polarity": "positive",
      "confidence": 0.95
    }
  ],
  "conflicts": [
    {
      "kind": "clash",
      "left": 0,
      "right": 3,
      "axis": { "kind": "naming", "scope": "variables" },
      "note": "camelCase disagrees with snake_case",
      "severity": "high",
      "confidence": 0.93
    }
  ],
  "total_tokens": 159,
  "stale_tokens": 12
}
```

## Recipes

### List only HIGH conflicts

```bash
aiscope --json . | jq '.conflicts[] | select(.severity == "high")'
```

### Count duplicates

```bash
aiscope --json . | jq '[.conflicts[] | select(.kind == "duplicate")] | length'
```

### Find sources with no `applyTo`

```bash
aiscope --json . | jq '.sources[] | select(.scope.globs == []) | .label'
```

### Waste percentage

```bash
aiscope --json . | jq '(.stale_tokens / .total_tokens * 100 | floor)'
```

## Stability

The schema follows semver:

- **0.x**: fields may be added; existing fields are stable within a minor.
- **1.0+**: schema is fully stable.

Field names use `snake_case` to match Rust conventions on the producing side.
