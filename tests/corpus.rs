//! Adversarial corpus snapshot tests.
//!
//! Each fixture in `tests/corpus/*.md` is a hand-written tricky markdown
//! document. We feed it through the full deterministic pipeline (parse →
//! canon → extract) and snapshot the resulting `Assertion`s as JSON.
//!
//! These tests double as a **determinism gate**: if the same fixture ever
//! produces a different assertion set on Linux/macOS/Windows, the snapshot
//! diff will fail and CI blocks the merge.
//!
//! To accept intentional changes:
//!   cargo insta review

use aiscope::canon::canonicalize;
use aiscope::extract::pattern;
use aiscope::model::{Assertion, Statement};
use aiscope::parse;
use serde::Serialize;
use std::fs;
use std::path::Path;

#[derive(Serialize)]
struct CorpusOutput {
    fixture: String,
    statements: Vec<Statement>,
    assertions: Vec<Assertion>,
}

fn run_corpus(name: &str) -> CorpusOutput {
    let path = Path::new("tests/corpus").join(format!("{name}.md"));
    let text = fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("failed to read corpus fixture {}: {e}", path.display()));

    let statements = parse::parse(0, &text);

    let mut assertions = Vec::new();
    for (i, stmt) in statements.iter().enumerate() {
        let canon = canonicalize(&stmt.text);
        assertions.extend(pattern::extract(i, stmt, &canon));
    }

    CorpusOutput {
        fixture: name.to_string(),
        statements,
        assertions,
    }
}

macro_rules! corpus_snapshot {
    ($name:ident) => {
        #[test]
        fn $name() {
            let out = run_corpus(stringify!($name));
            insta::assert_json_snapshot!(out);
        }
    };
}

corpus_snapshot!(negation_clause);
corpus_snapshot!(historical_narrative);
corpus_snapshot!(conditional_clauses);
corpus_snapshot!(frontmatter_and_code);
corpus_snapshot!(multi_axis);
