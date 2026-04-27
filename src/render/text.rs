//! Plain-text renderer. Used when the user passes `--text` or stdout is not a tty.

use crate::model::{ConflictKind, ContextBundle};

pub fn render(bundle: &ContextBundle) -> String {
    let mut out = String::new();
    out.push_str(&format!("aiscope · {}\n", bundle.root.display()));
    out.push_str(&format!(
        "{} rules across {} sources\n",
        bundle.rules.len(),
        bundle.sources.len()
    ));
    out.push_str(&format!(
        "{} conflicts \u{2022} {} tokens \u{2022} {}% wasted\n\n",
        bundle.conflicts.len(),
        bundle.total_tokens,
        bundle.waste_pct()
    ));

    if !bundle.conflicts.is_empty() {
        out.push_str("Conflicts:\n");
        for c in &bundle.conflicts {
            // Resolve left/right to underlying rule indices.
            let (li, ri) = match c.kind {
                ConflictKind::Duplicate | ConflictKind::AgentToolMismatch => (c.left, c.right),
                ConflictKind::Clash | ConflictKind::PolarityConflict => (
                    bundle.assertions[c.left].statement_index,
                    bundle.assertions[c.right].statement_index,
                ),
            };
            let l = bundle.rules.get(li);
            let r = bundle.rules.get(ri);
            let (Some(l), Some(r)) = (l, r) else { continue };
            let tag = match c.kind {
                ConflictKind::Duplicate => "DUP",
                ConflictKind::Clash => "CLASH",
                ConflictKind::PolarityConflict => "POLARITY",
                ConflictKind::AgentToolMismatch => "AGENT-TOOL",
            };
            let sev = match c.severity {
                crate::model::Severity::High => "high",
                crate::model::Severity::Low => "low",
            };
            let lsrc = bundle
                .sources
                .get(l.source_index)
                .map(|s| s.label.as_str())
                .unwrap_or("?");
            let rsrc = bundle
                .sources
                .get(r.source_index)
                .map(|s| s.label.as_str())
                .unwrap_or("?");
            out.push_str(&format!("  [{}] {} ({sev})\n", tag, c.note));
            out.push_str(&format!("       {}: {}\n", lsrc, truncate(&l.text, 70)));
            out.push_str(&format!("       {}: {}\n", rsrc, truncate(&r.text, 70)));
        }
    }

    out
}

fn truncate(s: &str, n: usize) -> String {
    if s.chars().count() <= n {
        s.to_string()
    } else {
        let mut t: String = s.chars().take(n).collect();
        t.push('\u{2026}');
        t
    }
}
