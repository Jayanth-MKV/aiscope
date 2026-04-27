//! Per-tool memory file scanners.

pub mod claude;
pub mod common;
pub mod copilot;
pub mod cursor;

use crate::model::Source;
use std::path::Path;

#[derive(Debug, Default, Clone, Copy)]
pub struct ScanOptions {
    /// When true, also scan user-scope (cross-repo) memory files
    /// like `~/.claude/CLAUDE.md`.
    pub include_user: bool,
}

/// Scan all known tools rooted at `repo_root`.
pub fn scan_all(repo_root: &Path, opts: ScanOptions) -> Vec<(Source, String)> {
    let mut out: Vec<(Source, String)> = Vec::new();
    out.extend(copilot::scan_raw(repo_root));
    out.extend(claude::scan_raw(repo_root, opts.include_user));
    out.extend(cursor::scan_raw(repo_root));
    out
}
