//! Cursor memory scanner.
//!
//! | Path                              | Subsystem    |
//! |-----------------------------------|--------------|
//! | `.cursorrules`                    | Instructions |
//! | `.cursor/rules/*.{md,mdc}`        | Instructions |
//! | `.cursor/commands/*.md`           | Prompts      |
//! | `.cursor/agents/*.md`             | Agents       |
//! | `.cursor/modes/*.md`              | ChatModes    |

use super::common::{push_dir, push_if};
use crate::model::{Source, Subsystem, Tool};
use std::path::Path;

pub fn scan_raw(repo_root: &Path) -> Vec<(Source, String)> {
    let mut out: Vec<(Source, String)> = Vec::new();

    push_if(
        &mut out,
        repo_root,
        &repo_root.join(".cursorrules"),
        Tool::Cursor,
        Subsystem::Instructions,
        ".cursorrules",
    );

    let cur = repo_root.join(".cursor");
    push_dir(
        &mut out,
        repo_root,
        &cur.join("rules"),
        &["md", "mdc"],
        Tool::Cursor,
        Subsystem::Instructions,
        ".cursor/rules/",
    );
    push_dir(
        &mut out,
        repo_root,
        &cur.join("commands"),
        &["md"],
        Tool::Cursor,
        Subsystem::Prompts,
        ".cursor/commands/",
    );
    push_dir(
        &mut out,
        repo_root,
        &cur.join("agents"),
        &["md"],
        Tool::Cursor,
        Subsystem::Agents,
        ".cursor/agents/",
    );
    push_dir(
        &mut out,
        repo_root,
        &cur.join("modes"),
        &["md"],
        Tool::Cursor,
        Subsystem::ChatModes,
        ".cursor/modes/",
    );

    out
}
