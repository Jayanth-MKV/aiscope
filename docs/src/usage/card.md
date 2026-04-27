# Summary card (`--card`)

```bash
aiscope --card scan.png .
```

Renders a 1280×720 PNG summary card. Designed for:

- PR descriptions and review threads
- Tweets / social posts about your repo's hygiene
- Slack / Discord drops in #engineering channels
- Conference slides

## What's on it

- **Title** — repo name (auto-detected from path)
- **Top 3 conflicts** — kind icon, short note, severity tag
- **Stats grid** — sources, rules, tokens, conflicts
- **Waste %** — `stale_tokens / total_tokens`, capped at 100%
- **Footer** — `aiscope · v{VERSION} · {REPO}`

## Font

Bundles **JetBrains Mono Regular** (270 KB, OFL license). No system-font
lookup — same output on every machine.

## Pixel-accurate layout

aiscope measures every glyph with [cosmic-text](https://github.com/pop-os/cosmic-text)
before drawing — no guesswork, no overflow, no clipped text. Long titles are
binary-search-truncated to fit with an ellipsis.

## Reproducibility

Same input → byte-identical PNG. Useful for golden tests in CI.

## Limitations

- Only Latin / Latin-Extended glyphs render — JetBrains Mono Regular doesn't
  ship CJK or arrow glyphs like `⇄`. aiscope uses `vs` instead.
- Fixed 1280×720 — no `--width` / `--height` flags yet.
