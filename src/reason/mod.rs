//! Layer 4 — `Assertion`s → `Conflict`s.
//!
//! Three kinds of conflict are produced:
//!
//! - **Duplicate** — two `Statement`s canonicalize to the same SHA-256
//!   fingerprint (cross-source paraphrase-free copy).
//! - **Clash** — two `Assertion`s on the same `(axis, condition)` carry
//!   different `AxisValue`s with the same `Polarity::Prefer`.
//! - **PolarityConflict** — one assertion `Prefer`s a value while another
//!   `Forbid`s the same value (with same condition).
//!
//! All conflicts carry a `severity`:
//!   - `High` if both sides came from cross-source files AND combined
//!     confidence ≥ 0.85.
//!   - `Low` otherwise (intra-source contradictions, weak confidence).

use crate::canon::canonicalize;
use crate::model::{
    Assertion, Conflict, ConflictKind, Severity, Source, Statement,
};
use sha2::{Digest, Sha256};
use std::collections::HashMap;

const HIGH_CONFIDENCE_BAR: f32 = 0.85;

/// Returns a stable string identifying just the axis *kind* (independent of
/// any embedded scope, e.g. `NamingScope`). Used as a clash-group key so
/// scoped + unscoped statements can still collide on the same axis.
fn axis_kind(a: crate::model::Axis) -> &'static str {
    use crate::model::Axis::*;
    match a {
        Naming(_) => "naming",
        Indentation => "indentation",
        QuoteStyle => "quote",
        PackageManager => "package_manager",
        AsyncStyle => "async",
        TestColocation => "test_colocation",
        TypeStrictness => "type_strictness",
        CommentDensity => "comment_density",
        ErrorHandling => "error_handling",
        ImportStyle => "import_style",
    }
}

/// Compute a 64-bit fingerprint of a statement's canonicalized form.
/// Two statements with the same fingerprint are treated as duplicates.
pub fn fingerprint(text: &str) -> u64 {
    let canon = canonicalize(text);
    let mut hasher = Sha256::new();
    hasher.update(canon.canon.as_bytes());
    let bytes = hasher.finalize();
    u64::from_be_bytes(bytes[..8].try_into().expect("sha256 has 32 bytes"))
}

/// Detect duplicate statements across sources. A duplicate is two statements
/// from DIFFERENT sources sharing a fingerprint. Same-source duplicates are
/// usually formatting artefacts, not real waste.
pub fn detect_duplicates(
    statements: &[Statement],
    sources: &[Source],
) -> Vec<Conflict> {
    let mut by_fp: HashMap<u64, Vec<usize>> = HashMap::new();
    for (i, s) in statements.iter().enumerate() {
        by_fp.entry(fingerprint(&s.text)).or_default().push(i);
    }

    let mut out = Vec::new();
    for (_, idxs) in by_fp {
        if idxs.len() < 2 {
            continue;
        }
        // Emit one conflict per pair where the two sides have DIFFERENT sources.
        for i in 0..idxs.len() {
            for j in (i + 1)..idxs.len() {
                let li = idxs[i];
                let ri = idxs[j];
                let l_src = statements[li].source_index;
                let r_src = statements[ri].source_index;
                if l_src == r_src {
                    continue;
                }
                let l_label = sources.get(l_src).map(|s| s.label.as_str()).unwrap_or("?");
                let r_label = sources.get(r_src).map(|s| s.label.as_str()).unwrap_or("?");
                out.push(Conflict {
                    kind: ConflictKind::Duplicate,
                    left: li,
                    right: ri,
                    axis: None,
                    note: format!("same statement appears in {l_label} and {r_label}"),
                    severity: Severity::High,
                    confidence: 1.0,
                });
            }
        }
    }
    out
}

/// Detect axis clashes and polarity conflicts.
pub fn detect_clashes(assertions: &[Assertion], statements: &[Statement]) -> Vec<Conflict> {
    use crate::model::Polarity;

    // Group by (axis-kind, condition-as-string). The axis-kind drops
    // `NamingScope` so "camelCase for variables" and "Don't use camelCase"
    // (unscoped) can clash on the underlying value.
    let mut groups: HashMap<(&'static str, String), Vec<usize>> = HashMap::new();
    for (i, a) in assertions.iter().enumerate() {
        let cond_key = a
            .condition
            .as_ref()
            .map(|c| c.raw.clone())
            .unwrap_or_default();
        groups.entry((axis_kind(a.axis), cond_key)).or_default().push(i);
    }

    let mut out = Vec::new();

    for (_key, idxs) in groups {
        // Build sub-buckets by AxisValue, separating polarities.
        for i in 0..idxs.len() {
            for j in (i + 1)..idxs.len() {
                let a = &assertions[idxs[i]];
                let b = &assertions[idxs[j]];

                // Skip if the two assertions came from the same statement
                // (clauses of "X but not Y" are NOT a real clash).
                if a.statement_index == b.statement_index {
                    continue;
                }

                // Skip if they came from the same source (intra-file rules
                // are author's known nuance, not cross-tool drift).
                let a_src = statements[a.statement_index].source_index;
                let b_src = statements[b.statement_index].source_index;
                let cross_source = a_src != b_src;

                let combined_conf = a.confidence.min(b.confidence);
                let severity = if cross_source && combined_conf >= HIGH_CONFIDENCE_BAR {
                    Severity::High
                } else {
                    Severity::Low
                };

                // Case 1 — same axis, both Prefer, different values → Clash
                if a.value != b.value
                    && a.polarity == Polarity::Prefer
                    && b.polarity == Polarity::Prefer
                {
                    out.push(Conflict {
                        kind: ConflictKind::Clash,
                        left: idxs[i],
                        right: idxs[j],
                        axis: Some(a.axis),
                        note: format!(
                            "{} disagrees with {}",
                            value_label(a.value),
                            value_label(b.value)
                        ),
                        severity,
                        confidence: combined_conf,
                    });
                    continue;
                }

                // Case 2 — same value, opposite polarities → PolarityConflict
                if a.value == b.value
                    && ((a.polarity == Polarity::Prefer && b.polarity == Polarity::Forbid)
                        || (a.polarity == Polarity::Forbid && b.polarity == Polarity::Prefer))
                {
                    out.push(Conflict {
                        kind: ConflictKind::PolarityConflict,
                        left: idxs[i],
                        right: idxs[j],
                        axis: Some(a.axis),
                        note: format!(
                            "one prefers {}, the other forbids it",
                            value_label(a.value)
                        ),
                        severity,
                        confidence: combined_conf,
                    });
                }
            }
        }
    }

    out
}

fn value_label(v: crate::model::AxisValue) -> &'static str {
    use crate::model::AxisValue::*;
    match v {
        CamelCase => "camelCase",
        SnakeCase => "snake_case",
        PascalCase => "PascalCase",
        KebabCase => "kebab-case",
        ScreamingSnakeCase => "SCREAMING_SNAKE_CASE",
        Tabs => "tabs",
        Spaces2 => "2 spaces",
        Spaces4 => "4 spaces",
        Spaces8 => "8 spaces",
        SingleQuote => "single quotes",
        DoubleQuote => "double quotes",
        Backtick => "backticks",
        Npm => "npm",
        Pnpm => "pnpm",
        Yarn => "yarn",
        Bun => "bun",
        AsyncAwait => "async/await",
        PromiseChain => "promise chains",
        Callbacks => "callbacks",
        BesideSource => "tests beside source",
        DedicatedDir => "tests in dedicated directory",
        Strict => "strict typing",
        Loose => "loose typing",
        Heavy => "heavy commenting",
        Minimal => "minimal commenting",
        Throw => "throw exceptions",
        ResultType => "Result-type errors",
        NamedImport => "named imports",
        DefaultImport => "default imports",
        NamespaceImport => "namespace imports",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::extract::pattern;
    use crate::model::{Source, Tool};
    use std::path::PathBuf;

    fn mk_stmt(src: usize, text: &str) -> Statement {
        Statement {
            source_index: src,
            text: text.to_string(),
            byte_start: 0,
            byte_end: text.len(),
            line: 1,
        }
    }

    fn mk_src(label: &str) -> Source {
        Source { tool: Tool::Cursor, path: PathBuf::from(label), label: label.to_string() }
    }

    #[test]
    fn cross_source_camel_vs_snake_clashes() {
        let stmts = vec![
            mk_stmt(0, "Use camelCase for variables."),
            mk_stmt(1, "Always use snake_case for variables."),
        ];
        let mut assertions = Vec::new();
        for (i, s) in stmts.iter().enumerate() {
            let canon = canonicalize(&s.text);
            assertions.extend(pattern::extract(i, s, &canon));
        }
        let clashes = detect_clashes(&assertions, &stmts);
        assert!(clashes.iter().any(|c| matches!(c.kind, ConflictKind::Clash)));
        assert!(clashes.iter().any(|c| c.severity == Severity::High));
    }

    #[test]
    fn dont_x_prefer_y_in_same_statement_no_clash() {
        let stmts = vec![mk_stmt(0, "Don't use camelCase, prefer snake_case.")];
        let canon = canonicalize(&stmts[0].text);
        let assertions = pattern::extract(0, &stmts[0], &canon);
        let clashes = detect_clashes(&assertions, &stmts);
        // No Clash — same-statement clauses must be ignored.
        assert!(clashes.iter().all(|c| !matches!(c.kind, ConflictKind::Clash)));
    }

    #[test]
    fn intra_source_clash_is_low_severity() {
        let stmts = vec![
            mk_stmt(0, "Use camelCase for variables."),
            mk_stmt(0, "Use snake_case for variables."),
        ];
        let mut assertions = Vec::new();
        for (i, s) in stmts.iter().enumerate() {
            let canon = canonicalize(&s.text);
            assertions.extend(pattern::extract(i, s, &canon));
        }
        let clashes = detect_clashes(&assertions, &stmts);
        assert!(clashes.iter().any(|c| c.severity == Severity::Low));
    }

    #[test]
    fn duplicate_paraphrased_via_canonical_punctuation() {
        let stmts = vec![
            mk_stmt(0, "Use camelCase."),
            mk_stmt(1, "use   camelCase"),
        ];
        let srcs = vec![mk_src("a"), mk_src("b")];
        let dups = detect_duplicates(&stmts, &srcs);
        assert_eq!(dups.len(), 1, "case+whitespace should canonicalize identically");
    }

    #[test]
    fn polarity_conflict_detected() {
        let stmts = vec![
            mk_stmt(0, "Use camelCase for variables."),
            mk_stmt(1, "Don't use camelCase."),
        ];
        let mut assertions = Vec::new();
        for (i, s) in stmts.iter().enumerate() {
            let canon = canonicalize(&s.text);
            assertions.extend(pattern::extract(i, s, &canon));
        }
        let clashes = detect_clashes(&assertions, &stmts);
        assert!(
            clashes.iter().any(|c| matches!(c.kind, ConflictKind::PolarityConflict)),
            "Prefer + Forbid on same value should be a polarity conflict"
        );
    }
}
