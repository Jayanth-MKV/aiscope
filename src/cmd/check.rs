//! `aiscope check` — CI mode. Exits 1 if any HIGH-severity conflicts are found.

use crate::diag;
use anyhow::Result;
use std::path::Path;
use std::process::ExitCode;

pub fn run(repo_root: &Path) -> Result<()> {
    let bundle = super::build_bundle(repo_root);
    print!("{}", diag::render(&bundle));

    if bundle.high_severity_conflicts().next().is_some() {
        std::process::exit(1);
    }
    Ok(())
}

#[allow(dead_code)]
fn _unused_exit_marker() -> ExitCode {
    ExitCode::SUCCESS
}
