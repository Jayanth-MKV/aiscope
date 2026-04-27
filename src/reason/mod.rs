//! Layer 4 — `Assertion`s → `Conflict`s.
//!
//! Two reasoning modes:
//!
//! - **`ReasonMode::Uniform`** *(default)* — every cross-file pair is a
//!   candidate conflict. Subsystem boundaries are ignored. Scope (applyTo /
//!   globs / path-prefix) gates *severity* but never silences a conflict.
//!
//! - **`ReasonMode::Specific`** *(`--specific` flag)* — subsystem-aware:
//!     * `Prompts` ↔ `Instructions`  → never conflict
//!     * `Agents`  ↔ non-`Agents`    → never conflict
//!     * `Skills` / `ChatModes`      → only duplicate-name and tool-allowlist
//!     * `Instructions` ↔ `Instructions` → full clash detection
//!
//! Severity rules (both modes):
//!
//! - `Severity::High` iff cross-file AND combined confidence ≥ 0.85
//!   AND scopes overlap.
//! - `Severity::Low`  if scopes do not overlap, OR confidence < 0.85,
//!   OR same source file.

use crate::canon::canonicalize;
use crate::frontmatter::globs_overlap;
use crate::model::{Assertion, Conflict, ConflictKind, Severity, Source, Statement, Subsystem};
use sha2::{Digest, Sha256};
use std::collections::HashMap;

const HIGH_CONFIDENCE_BAR: f32 = 0.85;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ReasonMode {
    #[default]
    Uniform,
    Specific,
}

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

pub fn fingerprint(text: &str) -> u64 {
    let canon = canonicalize(text);
    let mut hasher = Sha256::new();
    hasher.update(canon.canon.as_bytes());
    let bytes = hasher.finalize();
    u64::from_be_bytes(bytes[..8].try_into().expect("sha256 has 32 bytes"))
}

/// Detect duplicate statements across different source files.
pub fn detect_duplicates(statements: &[Statement], sources: &[Source]) -> Vec<Conflict> {
    let mut by_fp: HashMap<u64, Vec<usize>> = HashMap::new();
    for (i, s) in statements.iter().enumerate() {
        by_fp.entry(fingerprint(&s.text)).or_default().push(i);
    }
    let mut out = Vec::new();
    for (_, idxs) in by_fp {
        if idxs.len() < 2 {
            continue;
        }
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
                let scope_ok = match (sources.get(l_src), sources.get(r_src)) {
                    (Some(a), Some(b)) => scopes_overlap(a, b),
                    _ => true,
                };
                let severity = if scope_ok {
                    Severity::High
                } else {
                    Severity::Low
                };
                out.push(Conflict {
                    kind: ConflictKind::Duplicate,
                    left: li,
                    right: ri,
                    axis: None,
                    note: format!("same statement appears in {l_label} and {r_label}"),
                    severity,
                    confidence: 1.0,
                });
            }
        }
    }
    out
}

/// Detect axis clashes and polarity conflicts.
pub fn detect_clashes(
    assertions: &[Assertion],
    statements: &[Statement],
    sources: &[Source],
    mode: ReasonMode,
) -> Vec<Conflict> {
    use crate::model::Polarity;

    let mut groups: HashMap<(&'static str, String), Vec<usize>> = HashMap::new();
    for (i, a) in assertions.iter().enumerate() {
        let cond_key = a
            .condition
            .as_ref()
            .map(|c| c.raw.clone())
            .unwrap_or_default();
        groups
            .entry((axis_kind(a.axis), cond_key))
            .or_default()
            .push(i);
    }

    let mut out = Vec::new();

    for (_key, idxs) in groups {
        for i in 0..idxs.len() {
            for j in (i + 1)..idxs.len() {
                let a = &assertions[idxs[i]];
                let b = &assertions[idxs[j]];

                if a.statement_index == b.statement_index {
                    continue;
                }

                let a_src = statements[a.statement_index].source_index;
                let b_src = statements[b.statement_index].source_index;
                let cross_source = a_src != b_src;

                let a_source = sources.get(a_src);
                let b_source = sources.get(b_src);

                // Specific-mode subsystem gating.
                if mode == ReasonMode::Specific {
                    if let (Some(la), Some(lb)) = (a_source, b_source) {
                        if !subsystems_can_clash(la.subsystem, lb.subsystem) {
                            continue;
                        }
                    }
                }

                // Scope gating: do these rules ever apply to the same path?
                let scope_ok = match (a_source, b_source) {
                    (Some(la), Some(lb)) => scopes_overlap(la, lb),
                    _ => true,
                };

                let combined_conf = a.confidence.min(b.confidence);
                let severity = if cross_source && combined_conf >= HIGH_CONFIDENCE_BAR && scope_ok {
                    Severity::High
                } else {
                    Severity::Low
                };

                let scope_note = if !scope_ok {
                    " (scopes don't overlap)"
                } else {
                    ""
                };

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
                            "{} disagrees with {}{}",
                            value_label(a.value),
                            value_label(b.value),
                            scope_note
                        ),
                        severity,
                        confidence: combined_conf,
                    });
                    continue;
                }

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
                            "one prefers {}, the other forbids it{}",
                            value_label(a.value),
                            scope_note
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

/// In `--specific` mode: which subsystem pairs are ever real conflicts?
fn subsystems_can_clash(a: Subsystem, b: Subsystem) -> bool {
    use Subsystem::*;
    match (a, b) {
        // Prompts and slash-commands are user-invoked actions; their phrasing
        // is intentional and not in tension with always-on instructions.
        (Prompts, Instructions) | (Instructions, Prompts) => false,
        // Agents are isolated runners; only conflict with other agents.
        (Agents, x) | (x, Agents) if !matches!(x, Agents) => false,
        // Skills and ChatModes get name/tool checks elsewhere, no rule clashes.
        (Skills, _) | (_, Skills) | (ChatModes, _) | (_, ChatModes) => false,
        _ => true,
    }
}

/// Two sources have overlapping scopes if their applyTo / globs / path-prefix
/// can both match at least one common path.
fn scopes_overlap(a: &Source, b: &Source) -> bool {
    let a_globs = collect_globs(a);
    let b_globs = collect_globs(b);
    globs_overlap(&a_globs, &b_globs)
}

fn collect_globs(s: &Source) -> Vec<String> {
    let mut g = s.scope.globs.clone();
    if let Some(p) = &s.scope.path_prefix {
        g.push(p.clone());
    }
    g
}

/// Detect duplicate `name` frontmatter across agents / skills / chatmodes.
/// Each duplicate name becomes a `Conflict::Duplicate` with severity High.
pub fn detect_duplicate_names(sources: &[Source]) -> Vec<Conflict> {
    let mut by_name: HashMap<(Subsystem, String), Vec<usize>> = HashMap::new();
    for (i, s) in sources.iter().enumerate() {
        if !matches!(
            s.subsystem,
            Subsystem::Agents | Subsystem::Skills | Subsystem::ChatModes
        ) {
            continue;
        }
        if let Some(n) = &s.name {
            by_name.entry((s.subsystem, n.clone())).or_default().push(i);
        }
    }
    let mut out = Vec::new();
    for ((sub, name), idxs) in by_name {
        if idxs.len() < 2 {
            continue;
        }
        for i in 0..idxs.len() {
            for j in (i + 1)..idxs.len() {
                let l = idxs[i];
                let r = idxs[j];
                out.push(Conflict {
                    kind: ConflictKind::Duplicate,
                    left: l,
                    right: r,
                    axis: None,
                    note: format!(
                        "{} '{}' is defined twice ({} vs {})",
                        sub.label(),
                        name,
                        sources[l].label,
                        sources[r].label
                    ),
                    severity: Severity::High,
                    confidence: 1.0,
                });
            }
        }
    }
    out
}

/// Detect agents whose tool-allowlist looks broken:
///
/// - **Empty allowlist** while the agent's body mentions any tool word
///   ("read", "write", "search", "bash", "edit", "browse") — undefined behavior.
/// - **Mismatch**: an instructions-source body says "use the X tool" but
///   the agent's `tools:` allowlist does not contain X.
///
/// Both surface as `ConflictKind::AgentToolMismatch` with severity High.
pub fn detect_agent_tool_mismatches(sources: &[Source], statements: &[Statement]) -> Vec<Conflict> {
    use crate::model::Subsystem;
    let known_tools = [
        "read", "write", "edit", "search", "grep", "bash", "shell", "browse", "fetch", "run",
    ];
    let mut out = Vec::new();

    for (i, s) in sources.iter().enumerate() {
        if s.subsystem != Subsystem::Agents {
            continue;
        }
        // Find any statement that came from this agent file.
        let agent_stmt = statements
            .iter()
            .position(|st| st.source_index == i);

        // (1) Empty allowlist.
        if s.scope.tools.is_empty() {
            if let Some(stmt_idx) = agent_stmt {
                let body_lc: String = statements
                    .iter()
                    .filter(|st| st.source_index == i)
                    .map(|st| st.text.to_lowercase())
                    .collect::<Vec<_>>()
                    .join(" ");
                if known_tools.iter().any(|t| body_lc.contains(t)) {
                    out.push(Conflict {
                        kind: ConflictKind::AgentToolMismatch,
                        left: stmt_idx,
                        right: stmt_idx,
                        axis: None,
                        note: format!(
                            "agent {} mentions tools but has no `tools:` allowlist (undefined behavior)",
                            s.label
                        ),
                        severity: Severity::High,
                        confidence: 0.9,
                    });
                }
            }
            continue;
        }

        // (2) Mismatch: any instructions-source mentions a tool word that the
        // agent's allowlist excludes.
        let allowed: std::collections::HashSet<String> =
            s.scope.tools.iter().map(|t| t.to_lowercase()).collect();
        for (j, other) in sources.iter().enumerate() {
            if i == j || other.subsystem != Subsystem::Instructions {
                continue;
            }
            for (sti, st) in statements.iter().enumerate() {
                if st.source_index != j {
                    continue;
                }
                let body_lc = st.text.to_lowercase();
                for tool in known_tools.iter() {
                    let phrase = format!("{tool} tool");
                    if body_lc.contains(&phrase) && !allowed.contains(*tool) {
                        let agent_idx = agent_stmt.unwrap_or(sti);
                        out.push(Conflict {
                            kind: ConflictKind::AgentToolMismatch,
                            left: agent_idx,
                            right: sti,
                            axis: None,
                            note: format!(
                                "{} says \"use the {tool} tool\" but agent {} excludes it",
                                other.label, s.label
                            ),
                            severity: Severity::High,
                            confidence: 0.9,
                        });
                    }
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
    use crate::model::{Scope, Source, Subsystem, Tool};
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
        Source {
            tool: Tool::Cursor,
            subsystem: Subsystem::Instructions,
            path: PathBuf::from(label),
            label: label.to_string(),
            name: None,
            description: None,
            scope: Scope::default(),
        }
    }

    fn mk_src_with(label: &str, sub: Subsystem, globs: Vec<String>) -> Source {
        let mut s = mk_src(label);
        s.subsystem = sub;
        s.scope.globs = globs;
        s
    }

    #[test]
    fn cross_source_camel_vs_snake_clashes() {
        let stmts = vec![
            mk_stmt(0, "Use camelCase for variables."),
            mk_stmt(1, "Always use snake_case for variables."),
        ];
        let srcs = vec![mk_src("a"), mk_src("b")];
        let mut assertions = Vec::new();
        for (i, s) in stmts.iter().enumerate() {
            let canon = canonicalize(&s.text);
            assertions.extend(pattern::extract(i, s, &canon));
        }
        let clashes = detect_clashes(&assertions, &stmts, &srcs, ReasonMode::Uniform);
        assert!(clashes
            .iter()
            .any(|c| matches!(c.kind, ConflictKind::Clash)));
        assert!(clashes.iter().any(|c| c.severity == Severity::High));
    }

    #[test]
    fn dont_x_prefer_y_in_same_statement_no_clash() {
        let stmts = vec![mk_stmt(0, "Don't use camelCase, prefer snake_case.")];
        let srcs = vec![mk_src("a")];
        let canon = canonicalize(&stmts[0].text);
        let assertions = pattern::extract(0, &stmts[0], &canon);
        let clashes = detect_clashes(&assertions, &stmts, &srcs, ReasonMode::Uniform);
        assert!(clashes
            .iter()
            .all(|c| !matches!(c.kind, ConflictKind::Clash)));
    }

    #[test]
    fn intra_source_clash_is_low_severity() {
        let stmts = vec![
            mk_stmt(0, "Use camelCase for variables."),
            mk_stmt(0, "Use snake_case for variables."),
        ];
        let srcs = vec![mk_src("a")];
        let mut assertions = Vec::new();
        for (i, s) in stmts.iter().enumerate() {
            let canon = canonicalize(&s.text);
            assertions.extend(pattern::extract(i, s, &canon));
        }
        let clashes = detect_clashes(&assertions, &stmts, &srcs, ReasonMode::Uniform);
        assert!(clashes.iter().any(|c| c.severity == Severity::Low));
    }

    #[test]
    fn duplicate_paraphrased_via_canonical_punctuation() {
        let stmts = vec![mk_stmt(0, "Use camelCase."), mk_stmt(1, "use   camelCase")];
        let srcs = vec![mk_src("a"), mk_src("b")];
        let dups = detect_duplicates(&stmts, &srcs);
        assert_eq!(dups.len(), 1);
    }

    #[test]
    fn polarity_conflict_detected() {
        let stmts = vec![
            mk_stmt(0, "Use camelCase for variables."),
            mk_stmt(1, "Don't use camelCase."),
        ];
        let srcs = vec![mk_src("a"), mk_src("b")];
        let mut assertions = Vec::new();
        for (i, s) in stmts.iter().enumerate() {
            let canon = canonicalize(&s.text);
            assertions.extend(pattern::extract(i, s, &canon));
        }
        let clashes = detect_clashes(&assertions, &stmts, &srcs, ReasonMode::Uniform);
        assert!(clashes
            .iter()
            .any(|c| matches!(c.kind, ConflictKind::PolarityConflict)));
    }

    #[test]
    fn specific_mode_gates_prompts_vs_instructions() {
        let stmts = vec![
            mk_stmt(0, "Use camelCase for variables."),
            mk_stmt(1, "Use snake_case for variables."),
        ];
        let srcs = vec![
            mk_src_with("inst", Subsystem::Instructions, vec![]),
            mk_src_with("prompt", Subsystem::Prompts, vec![]),
        ];
        let mut assertions = Vec::new();
        for (i, s) in stmts.iter().enumerate() {
            let canon = canonicalize(&s.text);
            assertions.extend(pattern::extract(i, s, &canon));
        }
        let clashes = detect_clashes(&assertions, &stmts, &srcs, ReasonMode::Specific);
        assert!(
            !clashes
                .iter()
                .any(|c| matches!(c.kind, ConflictKind::Clash)),
            "in --specific mode, prompt vs instruction must not clash"
        );
    }

    #[test]
    fn non_overlapping_scope_demotes_to_low() {
        let stmts = vec![
            mk_stmt(0, "Use camelCase for variables."),
            mk_stmt(1, "Use snake_case for variables."),
        ];
        let srcs = vec![
            mk_src_with("ts", Subsystem::Instructions, vec!["**/*.ts".into()]),
            mk_src_with("py", Subsystem::Instructions, vec!["**/*.py".into()]),
        ];
        let mut assertions = Vec::new();
        for (i, s) in stmts.iter().enumerate() {
            let canon = canonicalize(&s.text);
            assertions.extend(pattern::extract(i, s, &canon));
        }
        let clashes = detect_clashes(&assertions, &stmts, &srcs, ReasonMode::Uniform);
        assert!(clashes
            .iter()
            .any(|c| matches!(c.kind, ConflictKind::Clash)));
        assert!(clashes.iter().all(|c| c.severity == Severity::Low));
    }

    #[test]
    fn duplicate_agent_name_detected() {
        let mut a = mk_src("a.md");
        a.subsystem = Subsystem::Agents;
        a.name = Some("reviewer".into());
        let mut b = mk_src("b.md");
        b.subsystem = Subsystem::Agents;
        b.name = Some("reviewer".into());
        let dups = detect_duplicate_names(&[a, b]);
        assert_eq!(dups.len(), 1);
        assert_eq!(dups[0].severity, Severity::High);
    }
}
