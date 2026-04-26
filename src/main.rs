//! aiscope — DevTools for your AI coding tools' memory.
//!
//! Read-only, local, no telemetry. Does NOT read session logs in v0.1.

use aiscope::cmd;
use anyhow::Result;
use clap::{Parser, Subcommand};
use std::path::PathBuf;

/// aiscope — see what your AI coding tools actually remember about your project.
#[derive(Debug, Parser)]
#[command(name = "aiscope", version, about, long_about = None)]
struct Cli {
    /// Path to the repository to scan (defaults to current directory).
    #[arg(default_value = ".")]
    path: PathBuf,

    /// Output mode.
    #[arg(long, conflicts_with_all = ["json", "card", "diag"])]
    text: bool,

    /// Emit machine-readable JSON to stdout.
    #[arg(long, conflicts_with_all = ["text", "card", "diag"])]
    json: bool,

    /// Render a shareable PNG card to the given path.
    #[arg(long, value_name = "PATH", conflicts_with_all = ["text", "json", "diag"])]
    card: Option<PathBuf>,

    /// Render conflicts as compiler-grade diagnostics (miette).
    #[arg(long, conflicts_with_all = ["text", "json", "card"])]
    diag: bool,

    /// Filter rules whose text matches this substring.
    #[arg(long, value_name = "PATTERN")]
    grep: Option<String>,

    #[command(subcommand)]
    command: Option<Command>,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// CI mode: scan and exit 1 if any conflicts are detected.
    Check,
    /// Watch the repo and re-scan on file changes.
    Watch,
}

fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "aiscope=info".into()),
        )
        .init();

    let cli = Cli::parse();

    match cli.command {
        Some(Command::Check) => cmd::check::run(&cli.path),
        Some(Command::Watch) => cmd::watch::run(&cli.path),
        None => cmd::scan::run(
            &cli.path,
            &cmd::scan::ScanOptions {
                text: cli.text,
                json: cli.json,
                card: cli.card,
                grep: cli.grep,
                diag: cli.diag,
            },
        ),
    }
}
