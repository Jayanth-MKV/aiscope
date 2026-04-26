//! GitHub Copilot memory scanner. **Implemented first per product priority.**
//!
//! Reads:
//! - `.github/copilot-instructions.md` (repo-level)
//! - `.github/instructions/*.instructions.md` (per-language)
//!
//! Does NOT touch VS Code workspace state in v0.1 (different storage per OS,
//! complexity vs viral-value ratio is poor for the launch scope).

use crate::model::{Source, Tool};
use std::path::Path;

pub fn scan_raw(repo_root: &Path) -> Vec<(Source, String)> {
    let mut out = Vec::new();

    let main = repo_root.join(".github").join("copilot-instructions.md");
    if let Some(entry) = read(repo_root, &main, "copilot-instructions.md") {
        out.push(entry);
    }

    let inst_dir = repo_root.join(".github").join("instructions");
    if inst_dir.is_dir() {
        if let Ok(read_dir) = std::fs::read_dir(&inst_dir) {
            let mut entries: Vec<_> = read_dir
                .flatten()
                .filter(|e| e.path().extension().map(|x| x == "md").unwrap_or(false))
                .collect();
            entries.sort_by_key(|e| e.path());
            for e in entries {
                let path = e.path();
                let label = format!(
                    ".github/instructions/{}",
                    path.file_name().and_then(|n| n.to_str()).unwrap_or("?")
                );
                if let Some(entry) = read(repo_root, &path, &label) {
                    out.push(entry);
                }
            }
        }
    }

    out
}

fn read(repo_root: &Path, path: &Path, label: &str) -> Option<(Source, String)> {
    let text = std::fs::read_to_string(path).ok()?;
    Some((
        Source {
            tool: Tool::Copilot,
            path: path.strip_prefix(repo_root).unwrap_or(path).to_path_buf(),
            label: label.to_string(),
        },
        text,
    ))
}
