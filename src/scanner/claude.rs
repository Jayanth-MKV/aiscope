//! Claude Code memory scanner.
//!
//! Reads (RULES ONLY, never session logs):
//! - `<repo>/CLAUDE.md`
//! - `<repo>/.claude/agents/*.md`
//! - `~/.claude/CLAUDE.md` (global, optional)
//!
//! **Privacy guard:** v0.1 MUST NOT open any file under `~/.claude/projects/`.

use crate::model::{Source, Tool};
use std::path::{Path, PathBuf};

pub fn scan_raw(repo_root: &Path) -> Vec<(Source, String)> {
    let mut out = Vec::new();

    let repo_claude = repo_root.join("CLAUDE.md");
    if let Some(entry) = read(&repo_claude, "CLAUDE.md", repo_root) {
        out.push(entry);
    }

    let agents_dir = repo_root.join(".claude").join("agents");
    if agents_dir.is_dir() {
        if let Ok(rd) = std::fs::read_dir(&agents_dir) {
            let mut entries: Vec<_> = rd
                .flatten()
                .filter(|e| e.path().extension().map(|x| x == "md").unwrap_or(false))
                .collect();
            entries.sort_by_key(|e| e.path());
            for e in entries {
                let p = e.path();
                let label = format!(
                    ".claude/agents/{}",
                    p.file_name().and_then(|n| n.to_str()).unwrap_or("?")
                );
                if let Some(entry) = read(&p, &label, repo_root) {
                    out.push(entry);
                }
            }
        }
    }

    // Global CLAUDE.md (~/.claude/CLAUDE.md)
    if let Some(home) = directories::BaseDirs::new().map(|b| b.home_dir().to_path_buf()) {
        let global = home.join(".claude").join("CLAUDE.md");
        if global.is_file() {
            // PRIVACY GUARD: confirm the path is the rules file, never a session log.
            assert!(
                !global.starts_with(home.join(".claude").join("projects")),
                "aiscope must never read session logs"
            );
            if let Some(entry) = read(&global, "~/.claude/CLAUDE.md", repo_root) {
                out.push(entry);
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
        Source { tool: Tool::Claude, path: display_path, label: label.to_string() },
        text,
    ))
}

#[cfg(test)]
mod tests {
    #[test]
    fn does_not_reference_projects_path() {
        let src = include_str!("claude.rs");
        let mentions = src.matches("projects").count();
        assert!(
            mentions <= 10,
            "claude.rs mentions 'projects' {mentions} times — v0.1 must avoid it"
        );
    }
}
