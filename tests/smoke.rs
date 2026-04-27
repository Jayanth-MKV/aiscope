//! End-to-end smoke test: assemble a fake repo from `tests/fixtures/`,
//! run the scanner, assert the camelCase vs snake_case clash is detected.

use std::fs;
use std::path::PathBuf;

fn copy(src: &str, dst: &PathBuf) {
    fs::create_dir_all(dst.parent().unwrap()).unwrap();
    fs::copy(src, dst).unwrap();
}

#[test]
fn detects_camel_vs_snake_across_cursor_and_claude() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path().to_path_buf();

    copy(
        "tests/fixtures/cursor/.cursorrules",
        &root.join(".cursorrules"),
    );
    copy("tests/fixtures/claude/CLAUDE.md", &root.join("CLAUDE.md"));
    copy(
        "tests/fixtures/copilot/copilot-instructions.md",
        &root.join(".github").join("copilot-instructions.md"),
    );

    let bundle = aiscope::build_bundle(&root, aiscope::cmd::PipelineOptions::default());

    assert!(
        !bundle.statements.is_empty(),
        "scanner should extract at least one statement from fixtures"
    );

    let has_clash = bundle.conflicts.iter().any(|c| {
        matches!(
            c.kind,
            aiscope::model::ConflictKind::Clash | aiscope::model::ConflictKind::PolarityConflict
        )
    });
    assert!(
        has_clash,
        "expected at least one cross-source naming/style clash. Got: {:#?}",
        bundle.conflicts
    );
}
