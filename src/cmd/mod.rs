//! Subcommand dispatch.

pub mod check;
pub mod scan;
pub mod watch;

use crate::detect::tokens;
use crate::model::{ContextBundle, Rule};
use crate::reason::ReasonMode;
use crate::scanner::ScanOptions;
use crate::{parse, reason, scanner};
use std::path::Path;

#[derive(Debug, Default, Clone, Copy)]
pub struct PipelineOptions {
    pub mode: ReasonMode,
    pub include_user: bool,
}

pub fn build_bundle(repo_root: &Path, opts: PipelineOptions) -> ContextBundle {
    let pairs = scanner::scan_all(
        repo_root,
        ScanOptions {
            include_user: opts.include_user,
        },
    );

    let mut sources = Vec::with_capacity(pairs.len());
    let mut statements = Vec::new();
    for (source, text) in pairs {
        let source_index = sources.len();
        sources.push(source);
        statements.extend(parse::parse(source_index, &text));
    }

    let mut assertions = Vec::new();
    for (i, stmt) in statements.iter().enumerate() {
        let canon = crate::canon::canonicalize(&stmt.text);
        assertions.extend(crate::extract::pattern::extract(i, stmt, &canon));
    }

    let mut conflicts = reason::detect_duplicates(&statements, &sources);
    conflicts.extend(reason::detect_clashes(
        &assertions,
        &statements,
        &sources,
        opts.mode,
    ));
    conflicts.extend(reason::detect_duplicate_names(&sources));
    conflicts.extend(reason::detect_agent_tool_mismatches(&sources, &statements));

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
