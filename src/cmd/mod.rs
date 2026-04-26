//! Subcommand dispatch.

pub mod check;
pub mod scan;
pub mod watch;

use crate::detect::tokens;
use crate::model::{ContextBundle, Rule};
use crate::{parse, reason, scanner};
use std::path::Path;

/// Build a fully-populated `ContextBundle` for `repo_root`.
/// Shared by every subcommand.
///
/// Pipeline (deterministic core):
///   scanners → raw text  → parse::parse → Statement
///                       → canon::canonicalize  → CanonicalText
///                       → extract::pattern → Assertion
///                       → reason::detect_clashes / detect_duplicates → Conflict
pub fn build_bundle(repo_root: &Path) -> ContextBundle {
    let pairs = scanner::scan_all(repo_root);

    let mut sources = Vec::with_capacity(pairs.len());
    let mut statements = Vec::new();
    for (source, text) in pairs {
        let source_index = sources.len();
        sources.push(source);
        statements.extend(parse::parse(source_index, &text));
    }

    // Layer 3: extract assertions from each statement.
    let mut assertions = Vec::new();
    for (i, stmt) in statements.iter().enumerate() {
        let canon = crate::canon::canonicalize(&stmt.text);
        assertions.extend(crate::extract::pattern::extract(i, stmt, &canon));
    }

    // Layer 4: detect duplicates + clashes + polarity conflicts.
    let mut conflicts = reason::detect_duplicates(&statements, &sources);
    conflicts.extend(reason::detect_clashes(&assertions, &statements));

    // Build legacy `rules` view consumed by text/JSON/TUI renderers.
    let mut rules: Vec<Rule> = statements
        .iter()
        .map(|s| Rule {
            source_index: s.source_index,
            text: s.text.clone(),
            tokens: 0,
            fingerprint: reason::fingerprint(&s.text),
        })
        .collect();
    tokens::rescore(&mut rules);

    let total_tokens: usize = rules.iter().map(|r| r.tokens).sum();
    // "Stale" tokens = tokens in duplicate statements (other side of the dup).
    let stale_tokens: usize = conflicts
        .iter()
        .filter(|c| matches!(c.kind, crate::model::ConflictKind::Duplicate))
        .map(|c| rules.get(c.right).map(|r| r.tokens).unwrap_or(0))
        .sum();

    ContextBundle {
        root: repo_root.to_path_buf(),
        sources,
        statements,
        assertions,
        rules,
        conflicts,
        total_tokens,
        stale_tokens,
    }
}
