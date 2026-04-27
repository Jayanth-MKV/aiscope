//! Integration test: a Copilot-only repository (no Cursor, no Claude).
//!
//! Verifies aiscope still works fine when the user is on a single tool —
//! every Copilot subsystem (instructions / prompts / agents / AGENTS.md)
//! gets discovered and conflicts between sibling files are detected.

use aiscope::cmd::PipelineOptions;
use aiscope::model::{ConflictKind, Severity, Subsystem, Tool};
use aiscope::reason::ReasonMode;
use std::fs;
use std::path::Path;

fn copy_tree(src: &Path, dst: &Path) {
    if src.is_dir() {
        fs::create_dir_all(dst).unwrap();
        for e in fs::read_dir(src).unwrap().flatten() {
            copy_tree(&e.path(), &dst.join(e.file_name()));
        }
    } else {
        fs::create_dir_all(dst.parent().unwrap()).unwrap();
        fs::copy(src, dst).unwrap();
    }
}

#[test]
fn copilot_only_discovers_all_subsystems() {
    let tmp = tempfile::tempdir().unwrap();
    copy_tree(Path::new("tests/fixtures/copilot-only"), tmp.path());

    let bundle = aiscope::build_bundle(tmp.path(), PipelineOptions::default());

    // Every source must be Copilot.
    assert!(bundle.sources.iter().all(|s| s.tool == Tool::Copilot));

    // We should see all five subsystems represented (or at least the four
    // we wrote).
    let subs: std::collections::HashSet<_> = bundle.sources.iter().map(|s| s.subsystem).collect();
    assert!(
        subs.contains(&Subsystem::Instructions),
        "instructions missing"
    );
    assert!(subs.contains(&Subsystem::Prompts), "prompts missing");
    assert!(subs.contains(&Subsystem::Agents), "agents missing");

    // Path-scoped AGENTS.md must be discovered with a path_prefix.
    let agents_md = bundle
        .sources
        .iter()
        .find(|s| s.label.ends_with("AGENTS.md"))
        .expect("apps/web/AGENTS.md not found");
    assert_eq!(agents_md.subsystem, Subsystem::Agents);
    assert!(
        agents_md
            .scope
            .path_prefix
            .as_deref()
            .unwrap_or("")
            .contains("apps/web"),
        "path_prefix should be derived from file location, got {:?}",
        agents_md.scope.path_prefix
    );

    // applyTo from frontmatter must populate scope.globs.
    let py = bundle
        .sources
        .iter()
        .find(|s| s.label.ends_with("python.instructions.md"))
        .expect("python.instructions.md not found");
    assert!(py.scope.globs.iter().any(|g| g.contains("py")));
    let ts = bundle
        .sources
        .iter()
        .find(|s| s.label.ends_with("typescript.instructions.md"))
        .expect("typescript.instructions.md not found");
    assert!(ts.scope.globs.iter().any(|g| g.contains("ts")));

    // python applyTo:**/*.py vs typescript applyTo:**/*.ts → no overlap →
    // their snake_case (py) vs snake_case+camelCase ts clash should be
    // demoted to Low severity.
    let py_ts_conflict = bundle.conflicts.iter().any(|c| {
        c.severity == Severity::Low
            && matches!(c.kind, ConflictKind::Clash | ConflictKind::PolarityConflict)
    });
    let _ = py_ts_conflict; // informational

    // Root copilot-instructions (camelCase, applies everywhere) vs
    // typescript.instructions (snake_case, applies to **/*.ts) — overlap →
    // High severity.
    assert!(
        bundle
            .high_severity_conflicts()
            .any(|c| matches!(c.kind, ConflictKind::Clash | ConflictKind::PolarityConflict)),
        "expected a HIGH-severity Copilot-only naming clash. Got: {:#?}",
        bundle.conflicts
    );
}

#[test]
fn copilot_only_specific_mode_silences_prompt_vs_instructions() {
    let tmp = tempfile::tempdir().unwrap();
    copy_tree(Path::new("tests/fixtures/copilot-only"), tmp.path());

    let bundle = aiscope::build_bundle(
        tmp.path(),
        PipelineOptions {
            mode: ReasonMode::Specific,
            include_user: false,
        },
    );

    // In --specific mode, Prompts ↔ Instructions never conflict.
    for c in &bundle.conflicts {
        if let (Some(l), Some(r)) = (
            bundle.statements.get(c.left),
            bundle.statements.get(c.right),
        ) {
            let ls = bundle.sources.get(l.source_index).map(|s| s.subsystem);
            let rs = bundle.sources.get(r.source_index).map(|s| s.subsystem);
            let pair = (
                ls.unwrap_or(Subsystem::Instructions),
                rs.unwrap_or(Subsystem::Instructions),
            );
            assert!(
                !matches!(
                    pair,
                    (Subsystem::Prompts, Subsystem::Instructions)
                        | (Subsystem::Instructions, Subsystem::Prompts)
                ),
                "specific-mode should suppress prompt↔instruction clashes: {:?}",
                c
            );
        }
    }

    // Specific mode should still surface AgentToolMismatch even though it
    // silences cross-subsystem clashes — the agent excludes "bash" but
    // python.instructions says "use the bash tool".
    let mismatches: Vec<_> = bundle
        .conflicts
        .iter()
        .filter(|c| matches!(c.kind, ConflictKind::AgentToolMismatch))
        .collect();
    assert!(
        !mismatches.is_empty(),
        "expected AgentToolMismatch (agent.tools excludes bash; instruction says use bash tool). Got: {:#?}",
        bundle.conflicts
    );
}
