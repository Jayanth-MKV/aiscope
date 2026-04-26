//! `aiscope watch` — live re-scan on file change. Stub for v0.1.

use anyhow::Result;
use std::path::Path;

pub fn run(repo_root: &Path) -> Result<()> {
    // TODO(v0.1 Sunday): notify-rs watcher on .cursor/, .claude/, .github/
    // and re-render TUI on debounced change events.
    let bundle = super::build_bundle(repo_root);
    print!("{}", crate::render::text::render(&bundle));
    eprintln!("\nwatch mode: not yet implemented (v0.1 stub) \u{2014} exiting after one scan.");
    Ok(())
}
