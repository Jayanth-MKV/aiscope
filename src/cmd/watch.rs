//! `aiscope watch` — re-scan on file changes.

use super::PipelineOptions;
use anyhow::Result;
use notify::{Event, RecursiveMode, Watcher};
use std::path::Path;
use std::sync::mpsc;
use std::time::{Duration, Instant};

pub fn run(path: &Path, pipeline: PipelineOptions) -> Result<()> {
    let (tx, rx) = mpsc::channel::<notify::Result<Event>>();
    let mut watcher = notify::recommended_watcher(move |res| {
        let _ = tx.send(res);
    })?;
    watcher.watch(path, RecursiveMode::Recursive)?;

    print_bundle(path, pipeline);

    let mut last = Instant::now();
    while let Ok(_evt) = rx.recv() {
        if last.elapsed() < Duration::from_millis(150) {
            continue;
        }
        print_bundle(path, pipeline);
        last = Instant::now();
    }
    Ok(())
}

fn print_bundle(path: &Path, pipeline: PipelineOptions) {
    let bundle = super::build_bundle(path, pipeline);
    print!("{}", crate::render::text::render(&bundle));
}
