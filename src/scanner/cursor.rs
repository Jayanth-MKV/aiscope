//! Cursor memory scanner.
//!
//! Reads:
//! - `<repo>/.cursorrules` (legacy, single file)
//! - `<repo>/.cursor/rules/*.mdc` (modern, multi-rule)

use crate::model::{Source, Tool};
use std::path::{Path, PathBuf};

pub fn scan_raw(repo_root: &Path) -> Vec<(Source, String)> {
    let mut out = Vec::new();

    let legacy = repo_root.join(".cursorrules");
    if let Some(entry) = read(&legacy, ".cursorrules", repo_root) {
        out.push(entry);
    }

    let rules_dir = repo_root.join(".cursor").join("rules");
    if rules_dir.is_dir() {
        if let Ok(rd) = std::fs::read_dir(&rules_dir) {
            let mut entries: Vec<_> = rd
                .flatten()
                .filter(|e| e.path().extension().map(|x| x == "mdc").unwrap_or(false))
                .collect();
            entries.sort_by_key(|e| e.path());
            for e in entries {
                let p = e.path();
                let label = format!(
                    ".cursor/rules/{}",
                    p.file_name().and_then(|n| n.to_str()).unwrap_or("?")
                );
                if let Some(entry) = read(&p, &label, repo_root) {
                    out.push(entry);
                }
            }
        }
    }

    out
}

fn read(path: &Path, label: &str, repo_root: &Path) -> Option<(Source, String)> {
    let text = std::fs::read_to_string(path).ok()?;
    let display_path = path
        .strip_prefix(repo_root)
        .map(PathBuf::from)
        .unwrap_or_else(|_| path.to_path_buf());
    Some((
        Source { tool: Tool::Cursor, path: display_path, label: label.to_string() },
        text,
    ))
}
