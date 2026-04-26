//! Default scan command (no subcommand).

use crate::{diag, render};
use anyhow::Result;
use std::path::{Path, PathBuf};

#[derive(Debug, Default, Clone)]
pub struct ScanOptions {
    pub text: bool,
    pub json: bool,
    pub card: Option<PathBuf>,
    pub grep: Option<String>,
    /// Render conflicts in compiler-grade `miette` style.
    pub diag: bool,
}

pub fn run(path: &Path, opts: &ScanOptions) -> Result<()> {
    let mut bundle = super::build_bundle(path);

    if let Some(pat) = &opts.grep {
        let needle = pat.to_lowercase();
        bundle.rules.retain(|r| r.text.to_lowercase().contains(&needle));
    }

    if opts.json {
        println!("{}", render::json::render(&bundle)?);
        return Ok(());
    }
    if let Some(card_path) = &opts.card {
        return render::card::render(&bundle, card_path);
    }
    if opts.diag {
        print!("{}", diag::render(&bundle));
        return Ok(());
    }
    if opts.text {
        print!("{}", render::text::render(&bundle));
        return Ok(());
    }

    render::tui::render(&bundle)
}
