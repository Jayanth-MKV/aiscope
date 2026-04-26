//! Per-tool memory file scanners.
//!
//! Each scanner reads a tool's known memory file locations and returns
//! `(Source, String)` pairs (raw file contents). Statement extraction
//! happens in [`crate::parse`], NOT here. Scanners are read-only and must
//! NEVER touch `~/.claude/projects/` or any session log directory.

pub mod claude;
pub mod copilot;
pub mod cursor;

use crate::model::Source;
use std::path::Path;

/// Scan all known tools rooted at `repo_root`. Returns one `(Source, raw_text)`
/// pair per discovered file. Never reads session logs.
pub fn scan_all(repo_root: &Path) -> Vec<(Source, String)> {
    let mut out: Vec<(Source, String)> = Vec::new();
    out.extend(copilot::scan_raw(repo_root));
    out.extend(claude::scan_raw(repo_root));
    out.extend(cursor::scan_raw(repo_root));
    out
}
