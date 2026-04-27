//! aiscope — DevTools for your AI coding tools' memory.

use aiscope::cmd;
use aiscope::reason::ReasonMode;
use anyhow::Result;
use clap::{Parser, Subcommand};
use std::path::PathBuf;
use std::process::ExitCode;

#[derive(Debug, Parser)]
#[command(name = "aiscope", version, about, long_about = None)]
struct Cli {
    /// Path to the repository to scan.
    #[arg(default_value = ".")]
    path: PathBuf,

    /// Plain text output.
    #[arg(long, conflicts_with_all = ["json", "card", "diag"])]
    text: bool,

    /// Machine-readable JSON.
    #[arg(long, conflicts_with_all = ["text", "card", "diag"])]
    json: bool,

    /// Render a shareable PNG card.
    #[arg(long, value_name = "PATH", conflicts_with_all = ["text", "json", "diag"])]
    card: Option<PathBuf>,

    /// Compiler-grade diagnostics (miette).
    #[arg(long, conflicts_with_all = ["text", "json", "card"])]
    diag: bool,

    /// Filter rules whose text matches this substring.
    #[arg(long, value_name = "PATTERN")]
    grep: Option<String>,

    /// Subsystem-aware reasoning. Disables conflicts between Prompts and
    /// Instructions, between Agents and non-Agents, etc. Default is
    /// uniform mode (every cross-file pair is a candidate conflict).
    #[arg(long)]
    specific: bool,

    /// Also scan user-scope memory files (`~/.claude/CLAUDE.md`, etc.).
    #[arg(long)]
    user: bool,

    #[command(subcommand)]
    command: Option<Command>,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// CI mode: scan and exit 1 if any high-severity conflicts are detected.
    Check,
    /// Watch the repo and re-scan on file changes.
    Watch,
}

fn main() -> Result<ExitCode> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "aiscope=info".into()),
        )
        .init();

    let cli = Cli::parse();
    let pipeline = cmd::PipelineOptions {
        mode: if cli.specific { ReasonMode::Specific } else { ReasonMode::Uniform },
        include_user: cli.user,
    };

    match cli.command {
        Some(Command::Check) => cmd::check::run(&cli.path, pipeline),
        Some(Command::Watch) => cmd::watch::run(&cli.path, pipeline).map(|_| ExitCode::SUCCESS),
        None => cmd::scan::run(
            &cli.path,
            &cmd::scan::ScanOptions {
                text: cli.text,
                json: cli.json,
                card: cli.card,
                grep: cli.grep,
                diag: cli.diag,
                pipeline,
            },
        )
        .map(|_| ExitCode::SUCCESS),
    }
}
