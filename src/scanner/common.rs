//! Shared helpers for tool-specific scanners.

use crate::frontmatter;
use crate::model::{Source, Subsystem, Tool};
use std::path::Path;

pub fn read_file_to_source(
    repo_root: &Path,
    path: &Path,
    tool: Tool,
    subsystem: Subsystem,
    label: &str,
    path_prefix: Option<String>,
) -> Option<(Source, String)> {
    let text = std::fs::read_to_string(path).ok()?;
    let (fm, _body) = frontmatter::parse(&text);
    let rel = path.strip_prefix(repo_root).unwrap_or(path).to_path_buf();
    let scope = frontmatter::to_scope(&fm, path_prefix);
    Some((
        Source {
            tool,
            subsystem,
            path: rel,
            label: label.to_string(),
            name: frontmatter::name(&fm),
            description: frontmatter::description(&fm),
            scope,
        },
        text,
    ))
}

pub fn push_dir(
    out: &mut Vec<(Source, String)>,
    repo_root: &Path,
    dir: &Path,
    exts: &[&str],
    tool: Tool,
    subsystem: Subsystem,
    label_prefix: &str,
) {
    if !dir.is_dir() {
        return;
    }
    let Ok(rd) = std::fs::read_dir(dir) else {
        return;
    };
    let mut entries: Vec<_> = rd
        .flatten()
        .filter(|e| {
            e.path()
                .extension()
                .and_then(|x| x.to_str())
                .map(|x| exts.iter().any(|w| w.eq_ignore_ascii_case(x)))
                .unwrap_or(false)
        })
        .collect();
    entries.sort_by_key(|e| e.path());
    for e in entries {
        let p = e.path();
        let label = format!(
            "{label_prefix}{}",
            p.file_name().and_then(|n| n.to_str()).unwrap_or("?")
        );
        if let Some(entry) = read_file_to_source(repo_root, &p, tool, subsystem, &label, None) {
            out.push(entry);
        }
    }
}

pub fn push_if(
    out: &mut Vec<(Source, String)>,
    repo_root: &Path,
    path: &Path,
    tool: Tool,
    subsystem: Subsystem,
    label: &str,
) {
    if !path.is_file() {
        return;
    }
    if let Some(entry) = read_file_to_source(repo_root, path, tool, subsystem, label, None) {
        out.push(entry);
    }
}

pub fn is_skip_dir(name: &str) -> bool {
    matches!(
        name,
        "node_modules" | "target" | ".git" | "dist" | "build" | ".next" | ".venv"
    )
}
