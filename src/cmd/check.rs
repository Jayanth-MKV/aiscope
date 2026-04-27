//! `aiscope check` — CI mode. Exits 1 on any high-severity conflict.

use super::PipelineOptions;
use anyhow::Result;
use std::path::Path;
use std::process::ExitCode;

pub fn run(path: &Path, pipeline: PipelineOptions) -> Result<ExitCode> {
    let bundle = super::build_bundle(path, pipeline);
    let high: Vec<_> = bundle.high_severity_conflicts().collect();
    if high.is_empty() {
        println!("aiscope: 0 high-severity conflicts");
        Ok(ExitCode::SUCCESS)
    } else {
        eprintln!(
            "aiscope: {} high-severity conflict{}",
            high.len(),
            if high.len() == 1 { "" } else { "s" }
        );
        for c in high {
            eprintln!("  - {:?}: {}", c.kind, c.note);
        }
        Ok(ExitCode::from(1))
    }
}
