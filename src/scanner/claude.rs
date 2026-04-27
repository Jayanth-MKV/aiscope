//! Claude Code memory scanner.
//!
//! | Path                                         | Subsystem    |
//! |----------------------------------------------|--------------|
//! | `CLAUDE.md` (any depth, path-scoped)         | Instructions |
//! | `.claude/agents/*.md`                        | Agents       |
//! | `.claude/commands/*.md`                      | Prompts      |
//! | `.claude/skills/*/SKILL.md`                  | Skills       |
//! | `~/.claude/CLAUDE.md` (when `--user`)        | Instructions |
//!
//! **Privacy guard:** v0.1 MUST NOT open any file under `~/.claude/projects/`.

use super::common::{is_skip_dir, push_dir, read_file_to_source};
use crate::model::{Source, Subsystem, Tool};
use std::path::Path;
use walkdir::WalkDir;

pub fn scan_raw(repo_root: &Path, include_user: bool) -> Vec<(Source, String)> {
    let mut out: Vec<(Source, String)> = Vec::new();

    // CLAUDE.md at any depth — root one is unscoped, deeper ones path-scoped.
    for entry in WalkDir::new(repo_root)
        .max_depth(6)
        .into_iter()
        .filter_entry(|e| !e.file_type().is_dir() || !is_skip_dir(&e.file_name().to_string_lossy()))
        .flatten()
    {
        if entry.file_name() == "CLAUDE.md" && entry.file_type().is_file() {
            let p = entry.path();
            let rel = p.strip_prefix(repo_root).unwrap_or(p);
            let label = rel.to_string_lossy().replace('\\', "/");
            let prefix = rel
                .parent()
                .map(|x| x.to_string_lossy().replace('\\', "/"))
                .filter(|s| !s.is_empty())
                .map(|s| format!("{s}/**"));
            if let Some(e) = read_file_to_source(
                repo_root,
                p,
                Tool::Claude,
                Subsystem::Instructions,
                &label,
                prefix,
            ) {
                out.push(e);
            }
        }
    }

    let cl = repo_root.join(".claude");
    push_dir(
        &mut out,
        repo_root,
        &cl.join("agents"),
        &["md"],
        Tool::Claude,
        Subsystem::Agents,
        ".claude/agents/",
    );
    push_dir(
        &mut out,
        repo_root,
        &cl.join("commands"),
        &["md"],
        Tool::Claude,
        Subsystem::Prompts,
        ".claude/commands/",
    );

    // .claude/skills/*/SKILL.md
    let skills_dir = cl.join("skills");
    if skills_dir.is_dir() {
        if let Ok(rd) = std::fs::read_dir(&skills_dir) {
            let mut subdirs: Vec<_> = rd.flatten().filter(|e| e.path().is_dir()).collect();
            subdirs.sort_by_key(|e| e.path());
            for d in subdirs {
                let skill_md = d.path().join("SKILL.md");
                if skill_md.is_file() {
                    let rel = skill_md
                        .strip_prefix(repo_root)
                        .unwrap_or(&skill_md)
                        .to_path_buf();
                    let label = rel.to_string_lossy().replace('\\', "/");
                    if let Some(e) = read_file_to_source(
                        repo_root,
                        &skill_md,
                        Tool::Claude,
                        Subsystem::Skills,
                        &label,
                        None,
                    ) {
                        out.push(e);
                    }
                }
            }
        }
    }

    // Optional ~/.claude/CLAUDE.md (rules only — never session logs).
    if include_user {
        if let Some(home) = directories::BaseDirs::new().map(|b| b.home_dir().to_path_buf()) {
            let global = home.join(".claude").join("CLAUDE.md");
            if global.is_file() {
                assert!(
                    !global.starts_with(home.join(".claude").join("projects")),
                    "aiscope must never read session logs"
                );
                if let Some(e) = read_file_to_source(
                    repo_root,
                    &global,
                    Tool::Claude,
                    Subsystem::Instructions,
                    "~/.claude/CLAUDE.md",
                    None,
                ) {
                    out.push(e);
                }
            }
        }
    }

    out
}

#[cfg(test)]
mod tests {
    #[test]
    fn does_not_reference_projects_path() {
        let src = include_str!("claude.rs");
        // 'projects' may appear in this assertion + comments; cap is generous.
        let mentions = src.matches("projects").count();
        assert!(
            mentions <= 10,
            "claude.rs mentions 'projects' {mentions} times — v0.1 must avoid it"
        );
    }
}
