//! Tiny YAML frontmatter extractor.
//!
//! We only need a handful of well-known keys (`applyTo`, `globs`,
//! `alwaysApply`, `model`, `tools`, `name`, `description`, `agent`).
//! Pulling in `serde_yaml` for that would add ~200KB and a brittle
//! dependency on libyaml. Instead this hand-rolled parser handles the
//! exact subset Copilot/Cursor/Claude actually emit.
//!
//! Recognised value shapes:
//!   key: value
//!   key: "quoted value"
//!   key: '**/*.ts'
//!   key: [a, b, c]
//!   key:
//!     - a
//!     - b

use crate::model::Scope;
use std::collections::HashMap;

#[derive(Debug, Default, Clone)]
pub struct Frontmatter {
    /// Raw key → first value (or comma-joined for arrays).
    pub map: HashMap<String, FmValue>,
}

#[derive(Debug, Clone)]
pub enum FmValue {
    Scalar(String),
    List(Vec<String>),
}

impl FmValue {
    pub fn as_scalar(&self) -> Option<&str> {
        match self {
            FmValue::Scalar(s) => Some(s),
            FmValue::List(_) => None,
        }
    }
    pub fn as_list(&self) -> Vec<String> {
        match self {
            FmValue::Scalar(s) => vec![s.clone()],
            FmValue::List(v) => v.clone(),
        }
    }
}

/// Parse a `---\n...\n---\n` block from the start of `text`. Returns
/// `(frontmatter, body_offset)` where `body_offset` is the byte index
/// in `text` where the body begins.
pub fn parse(text: &str) -> (Frontmatter, usize) {
    if !text.starts_with("---") {
        return (Frontmatter::default(), 0);
    }
    let after_first = &text[3..];
    let nl = match after_first.find('\n') {
        Some(n) => n + 1,
        None => return (Frontmatter::default(), 0),
    };
    let body_rel = &after_first[nl..];
    let close = match body_rel.find("\n---") {
        Some(n) => n,
        None => return (Frontmatter::default(), 0),
    };
    let yaml = &body_rel[..close];
    let after_close = &body_rel[close + 4..];
    let consume = match after_close.find('\n') {
        Some(n) => n + 1,
        None => after_close.len(),
    };
    let body_offset = 3 + nl + close + 4 + consume;

    let fm = parse_yaml_subset(yaml);
    (fm, body_offset)
}

fn parse_yaml_subset(yaml: &str) -> Frontmatter {
    let mut map = HashMap::new();
    let mut lines = yaml.lines().peekable();
    while let Some(raw) = lines.next() {
        let line = raw.trim_end();
        if line.trim().is_empty() || line.trim_start().starts_with('#') {
            continue;
        }
        // Skip continuation/list lines that didn't have a key on this line.
        let Some(colon) = line.find(':') else {
            continue;
        };
        let key = line[..colon].trim().to_string();
        if key.is_empty() || key.contains(' ') {
            continue;
        }
        let rest = line[colon + 1..].trim();

        if rest.is_empty() {
            // Block list / mapping follows on indented lines.
            let mut items = Vec::new();
            while let Some(peek) = lines.peek() {
                let p = peek.trim_start();
                if let Some(rest) = p.strip_prefix("- ") {
                    items.push(strip_quotes(rest.trim()));
                    lines.next();
                } else if p.is_empty() {
                    lines.next();
                } else {
                    break;
                }
            }
            if !items.is_empty() {
                map.insert(key, FmValue::List(items));
            }
            continue;
        }

        if let Some(stripped) = rest.strip_prefix('[').and_then(|s| s.strip_suffix(']')) {
            let items: Vec<String> = stripped
                .split(',')
                .map(|s| strip_quotes(s.trim()))
                .filter(|s| !s.is_empty())
                .collect();
            map.insert(key, FmValue::List(items));
        } else {
            map.insert(key, FmValue::Scalar(strip_quotes(rest)));
        }
    }
    Frontmatter { map }
}

fn strip_quotes(s: &str) -> String {
    let s = s.trim();
    if (s.starts_with('"') && s.ends_with('"') && s.len() >= 2)
        || (s.starts_with('\'') && s.ends_with('\'') && s.len() >= 2)
    {
        s[1..s.len() - 1].to_string()
    } else {
        s.to_string()
    }
}

/// Project the frontmatter into a `Scope`. Recognises the keys actually
/// used by Copilot, Cursor, and Claude:
///   - `applyTo` (Copilot .instructions.md): comma-separated globs
///   - `globs` (Cursor .mdc): list of globs
///   - `alwaysApply` (Cursor): bool
///   - `model` (prompts/agents): which model the rule targets
///   - `tools` (agents): allowlist
pub fn to_scope(fm: &Frontmatter, path_prefix: Option<String>) -> Scope {
    let mut globs: Vec<String> = Vec::new();

    if let Some(v) = fm.map.get("applyTo") {
        if let Some(s) = v.as_scalar() {
            globs.extend(
                s.split(',')
                    .map(|g| g.trim().to_string())
                    .filter(|g| !g.is_empty()),
            );
        } else {
            globs.extend(v.as_list());
        }
    }
    if let Some(v) = fm.map.get("globs") {
        globs.extend(v.as_list());
    }

    let always_apply = fm
        .map
        .get("alwaysApply")
        .and_then(|v| v.as_scalar())
        .map(|s| s.eq_ignore_ascii_case("true"))
        .unwrap_or(false);

    let model = fm
        .map
        .get("model")
        .and_then(|v| v.as_scalar())
        .map(|s| s.to_string());

    let tools = fm.map.get("tools").map(|v| v.as_list()).unwrap_or_default();

    Scope {
        globs,
        always_apply,
        path_prefix,
        model,
        tools,
    }
}

/// Convenience: pull the frontmatter `name`.
pub fn name(fm: &Frontmatter) -> Option<String> {
    fm.map
        .get("name")
        .and_then(|v| v.as_scalar())
        .map(|s| s.to_string())
}

/// Convenience: pull the frontmatter `description`.
pub fn description(fm: &Frontmatter) -> Option<String> {
    fm.map
        .get("description")
        .and_then(|v| v.as_scalar())
        .map(|s| s.to_string())
}

/// True if globs `a` and `b` could match any common path.
/// Empty side = matches everywhere → always overlaps.
pub fn globs_overlap(a: &[String], b: &[String]) -> bool {
    if a.is_empty() || b.is_empty() {
        return true;
    }
    use globset::Glob;
    // Build matchers from each side.
    let a_globs: Vec<_> = a
        .iter()
        .filter_map(|g| Glob::new(g).ok().map(|x| x.compile_matcher()))
        .collect();
    let b_globs: Vec<_> = b
        .iter()
        .filter_map(|g| Glob::new(g).ok().map(|x| x.compile_matcher()))
        .collect();
    if a_globs.is_empty() || b_globs.is_empty() {
        return true;
    }
    // Probe each glob's literal patterns against the other side.
    for g in a {
        let probe = literal_probe(g);
        if b_globs.iter().any(|m| m.is_match(&probe)) {
            return true;
        }
    }
    for g in b {
        let probe = literal_probe(g);
        if a_globs.iter().any(|m| m.is_match(&probe)) {
            return true;
        }
    }
    false
}

/// Convert a glob into a representative path that the glob itself matches.
/// Replaces `**` → `x`, `*` → `x`, `?` → `x`, drops `[...]`.
fn literal_probe(glob: &str) -> String {
    let mut out = String::with_capacity(glob.len());
    let bytes = glob.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        let c = bytes[i] as char;
        match c {
            '*' => {
                out.push('x');
                while i + 1 < bytes.len() && bytes[i + 1] == b'*' {
                    i += 1;
                }
            }
            '?' => out.push('x'),
            '[' => {
                while i < bytes.len() && bytes[i] != b']' {
                    i += 1;
                }
            }
            '{' => {
                while i < bytes.len() && bytes[i] != b'}' {
                    i += 1;
                }
            }
            _ => out.push(c),
        }
        i += 1;
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_apply_to() {
        let (fm, off) = parse("---\napplyTo: \"**/*.ts\"\n---\nbody\n");
        assert_eq!(
            fm.map.get("applyTo").and_then(|v| v.as_scalar()),
            Some("**/*.ts")
        );
        assert!(off > 0);
    }

    #[test]
    fn parses_block_list() {
        let (fm, _) = parse("---\ntools:\n  - read\n  - write\n---\n");
        let tools = fm.map.get("tools").unwrap().as_list();
        assert_eq!(tools, vec!["read", "write"]);
    }

    #[test]
    fn parses_inline_list() {
        let (fm, _) = parse("---\nglobs: [\"**/*.rs\", \"**/*.toml\"]\n---\n");
        let g = fm.map.get("globs").unwrap().as_list();
        assert_eq!(g, vec!["**/*.rs", "**/*.toml"]);
    }

    #[test]
    fn glob_overlap_basics() {
        assert!(globs_overlap(&["**/*.ts".into()], &["**/*.ts".into()]));
        assert!(globs_overlap(&["**/*".into()], &["src/foo.rs".into()]));
        assert!(!globs_overlap(&["**/*.ts".into()], &["**/*.py".into()]));
        assert!(globs_overlap(&[], &["**/*.ts".into()]));
    }

    #[test]
    fn no_frontmatter_returns_empty() {
        let (fm, off) = parse("just markdown body");
        assert!(fm.map.is_empty());
        assert_eq!(off, 0);
    }
}
