//! GitHub Copilot memory scanner.
//!
//! | Path                                                  | Subsystem    |
//! |-------------------------------------------------------|--------------|
//! | `.github/copilot-instructions.md`                     | Instructions |
//! | `.github/instructions/*.md`                           | Instructions |
//! | `.github/prompts/*.prompt.md`                         | Prompts      |
//! | `.github/chatmodes/*.chatmode.md`                     | ChatModes    |
//! | `.github/agents/*.md`                                 | Agents       |
//! | `AGENTS.md` (any depth, path-scoped)                  | Agents       |

use super::common::{is_skip_dir, push_dir, push_if, read_file_to_source};
use crate::model::{Source, Subsystem, Tool};
use std::path::Path;
use walkdir::WalkDir;

pub fn scan_raw(repo_root: &Path) -> Vec<(Source, String)> {
    let mut out: Vec<(Source, String)> = Vec::new();

    let gh = repo_root.join(".github");
    push_if(
        &mut out,
        repo_root,
        &gh.join("copilot-instructions.md"),
        Tool::Copilot,
        Subsystem::Instructions,
        ".github/copilot-instructions.md",
    );
    push_dir(
        &mut out,
        repo_root,
        &gh.join("instructions"),
        &["md"],
        Tool::Copilot,
        Subsystem::Instructions,
        ".github/instructions/",
    );
    push_dir(
        &mut out,
        repo_root,
        &gh.join("prompts"),
        &["md"],
        Tool::Copilot,
        Subsystem::Prompts,
        ".github/prompts/",
    );
    push_dir(
        &mut out,
        repo_root,
        &gh.join("chatmodes"),
        &["md"],
        Tool::Copilot,
        Subsystem::ChatModes,
        ".github/chatmodes/",
    );
    push_dir(
        &mut out,
        repo_root,
        &gh.join("agents"),
        &["md"],
        Tool::Copilot,
        Subsystem::Agents,
        ".github/agents/",
    );

    // AGENTS.md at any depth — path-scoped to its directory.
    for entry in WalkDir::new(repo_root)
        .max_depth(6)
        .into_iter()
        .filter_entry(|e| !e.file_type().is_dir() || !is_skip_dir(&e.file_name().to_string_lossy()))
        .flatten()
    {
        if entry.file_name() == "AGENTS.md" && entry.file_type().is_file() {
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
                Tool::Copilot,
                Subsystem::Agents,
                &label,
                prefix,
            ) {
                out.push(e);
            }
        }
    }

    out
}
