//! Layer 5 — Compiler-grade diagnostic rendering via `miette`.
//!
//! Renders each conflict as a Rust-compiler-style diagnostic with source
//! spans, labels, severity, and a `help:` suggestion. Pure plain-text output
//! suitable for CI logs; the TUI uses its own renderer.

use crate::model::{Conflict, ConflictKind, ContextBundle, Severity};
use miette::{Diagnostic, GraphicalReportHandler, LabeledSpan, NamedSource, Severity as MS};
use std::fmt;
use std::sync::Arc;

#[derive(Debug, Clone)]
struct AiscopeDiagnostic {
    code: &'static str,
    message: String,
    severity: MS,
    help: Option<String>,
    src: Arc<NamedSource<String>>,
    spans: Vec<LabeledSpan>,
}

impl fmt::Display for AiscopeDiagnostic {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.message)
    }
}

impl std::error::Error for AiscopeDiagnostic {}

impl Diagnostic for AiscopeDiagnostic {
    fn code<'a>(&'a self) -> Option<Box<dyn fmt::Display + 'a>> {
        Some(Box::new(self.code))
    }
    fn severity(&self) -> Option<MS> {
        Some(self.severity)
    }
    fn help<'a>(&'a self) -> Option<Box<dyn fmt::Display + 'a>> {
        self.help
            .as_ref()
            .map(|h| Box::new(h.clone()) as Box<dyn fmt::Display>)
    }
    fn source_code(&self) -> Option<&dyn miette::SourceCode> {
        Some(&*self.src)
    }
    fn labels<'a>(&'a self) -> Option<Box<dyn Iterator<Item = LabeledSpan> + 'a>> {
        Some(Box::new(self.spans.iter().cloned()))
    }
}

/// Render every conflict in the bundle as a miette diagnostic. Returns one
/// big string suitable for stdout / CI logs.
pub fn render(bundle: &ContextBundle) -> String {
    if bundle.conflicts.is_empty() {
        return format!(
            "aiscope · {} sources · {} statements · {} tokens · 0 conflicts\n",
            bundle.sources.len(),
            bundle.statements.len(),
            bundle.total_tokens
        );
    }

    // Build a NamedSource per touched source file (load file contents).
    let mut named: Vec<Option<Arc<NamedSource<String>>>> = vec![None; bundle.sources.len()];
    for (i, src) in bundle.sources.iter().enumerate() {
        let abs = bundle.root.join(&src.path);
        if let Ok(text) = std::fs::read_to_string(&abs) {
            named[i] = Some(Arc::new(NamedSource::new(src.label.clone(), text)));
        } else {
            named[i] = Some(Arc::new(NamedSource::new(src.label.clone(), String::new())));
        }
    }

    let mut out = String::new();
    out.push_str(&format!(
        "aiscope · {} sources · {} statements · {} tokens · {} conflicts ({} high)\n\n",
        bundle.sources.len(),
        bundle.statements.len(),
        bundle.total_tokens,
        bundle.conflicts.len(),
        bundle.high_severity_conflicts().count()
    ));

    let handler = GraphicalReportHandler::new();

    for (n, conf) in bundle.conflicts.iter().enumerate() {
        let diag = build_diagnostic(conf, bundle, &named);
        out.push_str(&format!("─── conflict {} ───\n", n + 1));
        let _ = handler.render_report(&mut out, &diag);
        out.push('\n');
    }

    out
}

fn build_diagnostic(
    conf: &Conflict,
    bundle: &ContextBundle,
    named: &[Option<Arc<NamedSource<String>>>],
) -> AiscopeDiagnostic {
    let (left_idx, right_idx) = match conf.kind {
        ConflictKind::Duplicate | ConflictKind::AgentToolMismatch => (conf.left, conf.right),
        ConflictKind::Clash | ConflictKind::PolarityConflict => {
            // For Clash/PolarityConflict, conf.left/right index assertions.
            // Resolve to underlying statements.
            (
                bundle.assertions[conf.left].statement_index,
                bundle.assertions[conf.right].statement_index,
            )
        }
    };

    let l_stmt = &bundle.statements[left_idx];
    let r_stmt = &bundle.statements[right_idx];

    let l_src_idx = l_stmt.source_index;
    let r_src_idx = r_stmt.source_index;

    // We use the LEFT source as the primary `source_code`; for the right side
    // we fall through to a textual note (miette only attaches one source per
    // diagnostic). Cross-file diagnostics are still very readable.
    let primary = named
        .get(l_src_idx)
        .and_then(|x| x.clone())
        .unwrap_or_else(|| Arc::new(NamedSource::new("?", String::new())));

    let l_label = bundle
        .sources
        .get(l_src_idx)
        .map(|s| s.label.as_str())
        .unwrap_or("?");
    let r_label = bundle
        .sources
        .get(r_src_idx)
        .map(|s| s.label.as_str())
        .unwrap_or("?");

    let l_span_start = l_stmt.byte_start;
    let l_span_len = l_stmt.byte_end.saturating_sub(l_stmt.byte_start).max(1);

    let (code, message, help) = match conf.kind {
        ConflictKind::Duplicate => (
            "aiscope::duplicate",
            format!("duplicate rule across {l_label} and {r_label}"),
            Some(format!(
                "wastes tokens; remove one. (also at {r_label}:{}: {})",
                r_stmt.line,
                truncate(&r_stmt.text, 80)
            )),
        ),
        ConflictKind::Clash => (
            "aiscope::clash",
            format!("contradictory rules: {}", conf.note),
            Some(format!(
                "the other side: {r_label}:{}: {}",
                r_stmt.line,
                truncate(&r_stmt.text, 80)
            )),
        ),
        ConflictKind::PolarityConflict => (
            "aiscope::polarity",
            format!("polarity conflict: {}", conf.note),
            Some(format!(
                "the other side: {r_label}:{}: {}",
                r_stmt.line,
                truncate(&r_stmt.text, 80)
            )),
        ),
        ConflictKind::AgentToolMismatch => (
            "aiscope::agent_tool",
            format!("agent tool mismatch: {}", conf.note),
            Some(
                "add the tool to the agent's `tools:` allowlist, or change the instruction"
                    .to_string(),
            ),
        ),
    };

    let label = LabeledSpan::at(l_span_start..l_span_start + l_span_len, "this rule");

    AiscopeDiagnostic {
        code,
        message,
        severity: match conf.severity {
            Severity::High => MS::Error,
            Severity::Low => MS::Warning,
        },
        help,
        src: primary,
        spans: vec![label],
    }
}

fn truncate(s: &str, n: usize) -> String {
    if s.chars().count() <= n {
        s.to_string()
    } else {
        let mut out: String = s.chars().take(n - 1).collect();
        out.push('…');
        out
    }
}
