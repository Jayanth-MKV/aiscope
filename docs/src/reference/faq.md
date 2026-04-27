# FAQ

### Does aiscope make network requests?

No. Zero. See [Privacy guard](../usage/privacy.md).

### Does aiscope write to my files?

No, except the PNG you ask for via `--card path.png`.

### Will it work on a repo with only Copilot files?

Yes. There's even a dedicated test fixture for it. Single-tool repos are
fully supported.

### Why does aiscope flag two files that _can't_ both apply (different `applyTo` globs)?

It doesn't — by default, non-overlapping scopes are demoted to **Low**
severity. `aiscope check` only fails on **High**. See
[Scope](../concepts/scope.md).

### My `--specific` mode hides a conflict I want to see. How do I get it back?

Drop the flag — default mode (Uniform) shows everything. Or use `--diag` /
`--text` to see Low-severity items inline.

### Does aiscope support [tool I just thought of]?

If the tool stores rules as markdown with optional YAML frontmatter, opening
an issue with paths and conventions is the easiest path. Adding a new
scanner module is ~50 lines.

### Why JetBrains Mono and not [other font]?

Free, open source (OFL), great Latin coverage, looks professional. The font
is bundled (270 KB) so the card renders identically on every machine.

### My card shows `vs` instead of `⇄`. Why?

JetBrains Mono Regular doesn't ship the `⇄` glyph (U+21C4). Rather than
fall back to a system font (which would break reproducibility), aiscope uses
`vs`. See [Summary card](../usage/card.md).

### Is the JSON schema stable?

Within a `0.x` minor: fields may be added but existing fields are stable.
Starting at `1.0`: fully stable.

### Can I run aiscope on a non-git directory?

Yes. aiscope honors `.gitignore` if present but doesn't require git.

### Where does the `4% context wasted` number come from?

`stale_tokens / total_tokens`, where `stale_tokens` is the size of any
statement involved in a `Duplicate` or losing side of a clash.
