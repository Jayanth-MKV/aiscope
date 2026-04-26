//! Layer 3a — Deterministic pattern-based axis extraction.
//!
//! For each statement we try every registered extractor; each may emit zero,
//! one, or many assertions. Confidence is hard-coded per extractor and
//! reflects how much wiggle room the pattern allows.
//!
//! Polarity detection is **structural**: we scan the surrounding tokens for
//! negation/preference words BEFORE matching axis values. This eliminates
//! the "Don't use camelCase, prefer snake_case" false-clash class in one shot.
//!
//! Conditional clauses ("in legacy code", "for tests only") are extracted
//! into `Assertion::condition` so the reasoner can compare like-for-like.

use crate::canon::CanonicalText;
use crate::model::{
    Assertion, Axis, AxisValue, Condition, ExtractionOrigin, NamingScope, Polarity, Statement,
};
use regex::Regex;
use std::sync::OnceLock;

/// Run all pattern extractors over one statement. Returns every assertion
/// produced; callers concatenate across statements.
pub fn extract(stmt_index: usize, stmt: &Statement, _canon: &CanonicalText) -> Vec<Assertion> {
    let mut out = Vec::new();

    // For clause splitting + pattern matching we use the ORIGINAL statement
    // text (lowercased) — `canon` strips punctuation, which would erase the
    // comma in "Don't use camelCase, prefer snake_case" and merge the two
    // clauses into one. Canon is reserved for fingerprinting in the reasoner.
    let text = stmt.text.to_lowercase();

    let clauses = split_clauses(&text);
    for clause in clauses {
        let polarity = detect_polarity(&clause.text);
        let condition = clause.condition.map(|raw| Condition { raw });

        for ext in EXTRACTORS {
            ext(&clause.text, polarity, condition.as_ref(), stmt_index, &mut out);
        }
    }

    out
}

// ---------------------------------------------------------------------------
// Polarity detection
// ---------------------------------------------------------------------------

const FORBID_WORDS: &[&str] = &[
    "don't", "do not", "dont", "never", "avoid", "forbid", "ban", "no ",
    "not ", "without",
];
const PREFER_WORDS: &[&str] = &[
    "use ", "prefer", "always", "must", "should", "require", "stick to",
    "go with", "default to", "favour", "favor", "choose",
];
const ALLOW_WORDS: &[&str] = &[
    "may", "can ", "is allowed", "is fine", "is acceptable", "is ok",
    "is okay", "permitted",
];

fn detect_polarity(text: &str) -> Polarity {
    // Order matters: forbid > allow > prefer (most negation-heavy first).
    if FORBID_WORDS.iter().any(|w| text.contains(w)) {
        return Polarity::Forbid;
    }
    if ALLOW_WORDS.iter().any(|w| text.contains(w)) {
        return Polarity::Allow;
    }
    if PREFER_WORDS.iter().any(|w| text.contains(w)) {
        return Polarity::Prefer;
    }
    Polarity::Prefer // default — bare assertions are preferences
}

// ---------------------------------------------------------------------------
// Clause splitter (handles "X, but Y" / "X; Y" / "X. Y" / conditionals)
// ---------------------------------------------------------------------------

struct Clause {
    text: String,
    condition: Option<String>,
}

fn split_clauses(text: &str) -> Vec<Clause> {
    static SPLITTER: OnceLock<Regex> = OnceLock::new();
    let re = SPLITTER.get_or_init(|| {
        // Split on sentence terminators, semicolons, commas, and contrastive conjunctions.
        Regex::new(r#"\s*(?:[.;,]+|\bbut\b|\bhowever\b|\bwhereas\b|\band yet\b)\s*"#).unwrap()
    });

    let mut out = Vec::new();
    for piece in re.split(text) {
        let piece = piece.trim();
        if piece.is_empty() {
            continue;
        }
        // Detect a trailing conditional clause ("in legacy code", "for tests").
        let (head, condition) = strip_condition(piece);
        out.push(Clause {
            text: head.to_string(),
            condition: condition.map(str::to_string),
        });
    }
    if out.is_empty() {
        out.push(Clause { text: text.to_string(), condition: None });
    }
    out
}

fn strip_condition(text: &str) -> (&str, Option<&str>) {
    static COND: OnceLock<Regex> = OnceLock::new();
    let re = COND.get_or_init(|| {
        Regex::new(
            r#"(?P<head>.+?)\s+(?P<cond>(?:in (?:legacy|new|test|tests|production|dev|development)|for (?:tests?|production|dev|development|legacy)|when (?:writing|interfacing|migrating|in)|only (?:when|in|for)|except (?:in|when|for))\b.*)$"#,
        )
        .unwrap()
    });
    if let Some(c) = re.captures(text) {
        let head = c.name("head").map(|m| m.as_str()).unwrap_or(text);
        let cond = c.name("cond").map(|m| m.as_str());
        (head, cond)
    } else {
        (text, None)
    }
}

// ---------------------------------------------------------------------------
// Extractor table — one per axis value group
// ---------------------------------------------------------------------------

type Extractor = fn(&str, Polarity, Option<&Condition>, usize, &mut Vec<Assertion>);

const EXTRACTORS: &[Extractor] = &[
    extract_naming,
    extract_indentation,
    extract_quote_style,
    extract_package_manager,
    extract_async_style,
    extract_test_colocation,
    extract_type_strictness,
    extract_error_handling,
    extract_import_style,
    extract_comment_density,
];

fn push(
    out: &mut Vec<Assertion>,
    stmt_index: usize,
    axis: Axis,
    value: AxisValue,
    polarity: Polarity,
    condition: Option<&Condition>,
    confidence: f32,
) {
    out.push(Assertion {
        statement_index: stmt_index,
        axis,
        value,
        polarity,
        condition: condition.cloned(),
        confidence,
        origin: ExtractionOrigin::Pattern,
    });
}

// ---------- Naming ---------------------------------------------------------

fn extract_naming(
    text: &str,
    pol: Polarity,
    cond: Option<&Condition>,
    si: usize,
    out: &mut Vec<Assertion>,
) {
    static RX: OnceLock<Vec<(Regex, AxisValue, NamingScope)>> = OnceLock::new();
    let table = RX.get_or_init(|| {
        let mk = |re: &str, v: AxisValue, s: NamingScope| (Regex::new(re).unwrap(), v, s);
        vec![
            mk(r"\bcamel[\s_-]?cas(?:e|ing|ed)\b", AxisValue::CamelCase, NamingScope::Any),
            mk(r"\bsnake[\s_-]?cas(?:e|ing|ed)\b", AxisValue::SnakeCase, NamingScope::Any),
            mk(r"\bpascal[\s_-]?cas(?:e|ing|ed)\b", AxisValue::PascalCase, NamingScope::Any),
            mk(r"\bkebab[\s_-]?cas(?:e|ing|ed)\b", AxisValue::KebabCase, NamingScope::Any),
            mk(r"\bscreaming[\s_-]?snake[\s_-]?case\b", AxisValue::ScreamingSnakeCase, NamingScope::Any),
            mk(r"\ball[\s_-]?caps\b", AxisValue::ScreamingSnakeCase, NamingScope::Constants),
        ]
    });

    let scope_hint = detect_naming_scope(text);

    for (rx, value, default_scope) in table {
        if rx.is_match(text) {
            let scope = scope_hint.unwrap_or(*default_scope);
            push(out, si, Axis::Naming(scope), *value, pol, cond, 0.97);
        }
    }
}

fn detect_naming_scope(text: &str) -> Option<NamingScope> {
    if text.contains("variable") || text.contains("identifier") {
        Some(NamingScope::Variables)
    } else if text.contains("function") || text.contains("method") {
        Some(NamingScope::Functions)
    } else if text.contains("type") || text.contains("class") || text.contains("interface")
        || text.contains("struct") || text.contains("enum")
    {
        Some(NamingScope::Types)
    } else if text.contains("constant") {
        Some(NamingScope::Constants)
    } else if text.contains("file") || text.contains("module") {
        Some(NamingScope::Files)
    } else {
        None
    }
}

// ---------- Indentation ----------------------------------------------------

fn extract_indentation(
    text: &str,
    pol: Polarity,
    cond: Option<&Condition>,
    si: usize,
    out: &mut Vec<Assertion>,
) {
    static RX: OnceLock<Vec<(Regex, AxisValue)>> = OnceLock::new();
    let table = RX.get_or_init(|| {
        vec![
            (Regex::new(r"\b(?:use\s+)?tabs?\b(?:\s+for\s+indent\w*)?").unwrap(), AxisValue::Tabs),
            (Regex::new(r"\bindent\w*\s+with\s+tabs?\b").unwrap(), AxisValue::Tabs),
            (Regex::new(r"\b2[\s-]?spaces?\b").unwrap(), AxisValue::Spaces2),
            (Regex::new(r"\btwo\s+spaces?\b").unwrap(), AxisValue::Spaces2),
            (Regex::new(r"\b4[\s-]?spaces?\b").unwrap(), AxisValue::Spaces4),
            (Regex::new(r"\bfour\s+spaces?\b").unwrap(), AxisValue::Spaces4),
            (Regex::new(r"\b8[\s-]?spaces?\b").unwrap(), AxisValue::Spaces8),
        ]
    });
    // Only fire if the statement is actually about indentation/style.
    let is_indent_topic = text.contains("indent") || text.contains("tab") || text.contains("space");
    if !is_indent_topic {
        return;
    }
    for (rx, value) in table {
        if rx.is_match(text) {
            push(out, si, Axis::Indentation, *value, pol, cond, 0.95);
        }
    }
}

// ---------- Quote style ----------------------------------------------------

fn extract_quote_style(
    text: &str,
    pol: Polarity,
    cond: Option<&Condition>,
    si: usize,
    out: &mut Vec<Assertion>,
) {
    if !text.contains("quote") && !text.contains("string") && !text.contains("'") && !text.contains('"') {
        return;
    }
    if text.contains("single quote") || text.contains("single-quote") {
        push(out, si, Axis::QuoteStyle, AxisValue::SingleQuote, pol, cond, 0.96);
    }
    if text.contains("double quote") || text.contains("double-quote") {
        push(out, si, Axis::QuoteStyle, AxisValue::DoubleQuote, pol, cond, 0.96);
    }
    if text.contains("backtick") || text.contains("template literal") {
        push(out, si, Axis::QuoteStyle, AxisValue::Backtick, pol, cond, 0.95);
    }
}

// ---------- Package manager ------------------------------------------------

fn extract_package_manager(
    text: &str,
    pol: Polarity,
    cond: Option<&Condition>,
    si: usize,
    out: &mut Vec<Assertion>,
) {
    static RX: OnceLock<Vec<(Regex, AxisValue)>> = OnceLock::new();
    let table = RX.get_or_init(|| {
        vec![
            (Regex::new(r"\bpnpm\b").unwrap(), AxisValue::Pnpm),
            (Regex::new(r"\byarn\b").unwrap(), AxisValue::Yarn),
            (Regex::new(r"\bnpm\b").unwrap(), AxisValue::Npm),
            (Regex::new(r"\bbun\b").unwrap(), AxisValue::Bun),
        ]
    });

    // Require either an explicit PM topic OR an opinionated polarity
    // (Prefer/Forbid) — this way "the npm registry is reliable" (default
    // Prefer-as-fallback gives a hit) is excluded, while "Use pnpm" passes.
    let explicit_topic = text.contains("package")
        || text.contains("install")
        || text.contains("dependenc")
        || text.contains("lockfile")
        || text.contains(" pm ")
        || text.starts_with("pm ")
        || text.contains("manager");

    let opinionated = matches!(pol, Polarity::Prefer | Polarity::Forbid)
        && (FORBID_WORDS.iter().any(|w| text.contains(w))
            || PREFER_WORDS.iter().any(|w| text.contains(w)));

    if !explicit_topic && !opinionated {
        return;
    }

    for (rx, value) in table {
        if rx.is_match(text) {
            push(out, si, Axis::PackageManager, *value, pol, cond, 0.93);
        }
    }
}

// ---------- Async style ----------------------------------------------------

fn extract_async_style(
    text: &str,
    pol: Polarity,
    cond: Option<&Condition>,
    si: usize,
    out: &mut Vec<Assertion>,
) {
    if (text.contains("async") || text.contains("await"))
        && (text.contains("async/await") || text.contains("async-await") || text.contains("await"))
    {
        push(out, si, Axis::AsyncStyle, AxisValue::AsyncAwait, pol, cond, 0.94);
    }
    if text.contains("promise chain") || text.contains(".then(") || text.contains("then chains") {
        push(out, si, Axis::AsyncStyle, AxisValue::PromiseChain, pol, cond, 0.93);
    }
    if text.contains("callback") {
        push(out, si, Axis::AsyncStyle, AxisValue::Callbacks, pol, cond, 0.90);
    }
}

// ---------- Test colocation ------------------------------------------------

fn extract_test_colocation(
    text: &str,
    pol: Polarity,
    cond: Option<&Condition>,
    si: usize,
    out: &mut Vec<Assertion>,
) {
    let test_topic = text.contains("test");
    if !test_topic {
        return;
    }
    if text.contains("co-locate") || text.contains("colocate") || text.contains("beside") || text.contains("next to") || text.contains("alongside") {
        push(out, si, Axis::TestColocation, AxisValue::BesideSource, pol, cond, 0.95);
    }
    if text.contains("__tests__") || text.contains("/tests/") || text.contains("dedicated") || text.contains("separate test") || text.contains("test directory") || text.contains("tests folder") {
        push(out, si, Axis::TestColocation, AxisValue::DedicatedDir, pol, cond, 0.93);
    }
}

// ---------- Type strictness ------------------------------------------------

fn extract_type_strictness(
    text: &str,
    pol: Polarity,
    cond: Option<&Condition>,
    si: usize,
    out: &mut Vec<Assertion>,
) {
    if text.contains("strict") && (text.contains("type") || text.contains("typescript")) {
        push(out, si, Axis::TypeStrictness, AxisValue::Strict, pol, cond, 0.92);
    }
    if text.contains("any") && (text.contains("type") || text.contains("avoid") || text.contains("allow")) {
        // "avoid any" → polarity Forbid against Loose; "any is fine" → Allow Loose.
        push(out, si, Axis::TypeStrictness, AxisValue::Loose, pol, cond, 0.85);
    }
    if text.contains("noimplicitany") {
        push(out, si, Axis::TypeStrictness, AxisValue::Strict, pol, cond, 0.95);
    }
}

// ---------- Error handling -------------------------------------------------

fn extract_error_handling(
    text: &str,
    pol: Polarity,
    cond: Option<&Condition>,
    si: usize,
    out: &mut Vec<Assertion>,
) {
    if text.contains("throw") || text.contains("exception") {
        push(out, si, Axis::ErrorHandling, AxisValue::Throw, pol, cond, 0.90);
    }
    if text.contains("result type") || text.contains("result<") || text.contains("either type") || text.contains("neverthrow") || text.contains("ok/err") {
        push(out, si, Axis::ErrorHandling, AxisValue::ResultType, pol, cond, 0.93);
    }
}

// ---------- Import style ---------------------------------------------------

fn extract_import_style(
    text: &str,
    pol: Polarity,
    cond: Option<&Condition>,
    si: usize,
    out: &mut Vec<Assertion>,
) {
    if !text.contains("import") {
        return;
    }
    if text.contains("named import") || text.contains("named exports") {
        push(out, si, Axis::ImportStyle, AxisValue::NamedImport, pol, cond, 0.93);
    }
    if text.contains("default import") || text.contains("default export") {
        push(out, si, Axis::ImportStyle, AxisValue::DefaultImport, pol, cond, 0.92);
    }
    if text.contains("namespace import") || text.contains("import * as") {
        push(out, si, Axis::ImportStyle, AxisValue::NamespaceImport, pol, cond, 0.93);
    }
}

// ---------- Comment density ------------------------------------------------

fn extract_comment_density(
    text: &str,
    pol: Polarity,
    cond: Option<&Condition>,
    si: usize,
    out: &mut Vec<Assertion>,
) {
    if !text.contains("comment") && !text.contains("docstring") && !text.contains("doc string") && !text.contains("documentation") {
        return;
    }
    if text.contains("heavy") || text.contains("verbose") || text.contains("extensive") || text.contains("thorough") {
        push(out, si, Axis::CommentDensity, AxisValue::Heavy, pol, cond, 0.88);
    }
    if text.contains("minimal") || text.contains("sparse") || text.contains("only when") || text.contains("self-document") {
        push(out, si, Axis::CommentDensity, AxisValue::Minimal, pol, cond, 0.88);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::canon::canonicalize;
    use crate::model::Statement;

    fn ext(text: &str) -> Vec<Assertion> {
        let stmt = Statement {
            source_index: 0,
            text: text.to_string(),
            byte_start: 0,
            byte_end: text.len(),
            line: 1,
        };
        let canon = canonicalize(text);
        extract(0, &stmt, &canon)
    }

    #[test]
    fn camel_case_prefer() {
        let a = ext("Use camelCase for variables.");
        assert!(a.iter().any(|x| matches!(x.value, AxisValue::CamelCase) && x.polarity == Polarity::Prefer));
    }

    #[test]
    fn snake_case_via_underscore_token_survives() {
        let a = ext("Always use snake_case for variables.");
        assert!(a.iter().any(|x| matches!(x.value, AxisValue::SnakeCase)));
    }

    #[test]
    fn negation_flips_polarity() {
        let a = ext("Don't use camelCase.");
        let camel = a.iter().find(|x| matches!(x.value, AxisValue::CamelCase)).expect("camel found");
        assert_eq!(camel.polarity, Polarity::Forbid);
    }

    #[test]
    fn dont_x_prefer_y_yields_two_clauses_no_self_clash() {
        let a = ext("Don't use camelCase, prefer snake_case.");
        let camel = a.iter().find(|x| matches!(x.value, AxisValue::CamelCase)).unwrap();
        let snake = a.iter().find(|x| matches!(x.value, AxisValue::SnakeCase)).unwrap();
        assert_eq!(camel.polarity, Polarity::Forbid);
        assert_eq!(snake.polarity, Polarity::Prefer);
    }

    #[test]
    fn historical_narrative_filtered_via_condition() {
        // "We migrated from npm to pnpm last year." — currently this still
        // emits assertions for both pnpm and npm. The reasoner uses polarity
        // and condition to avoid clash. Test that conditions are extracted
        // when present.
        let a = ext("Use pnpm for tests only.");
        let pnpm = a.iter().find(|x| matches!(x.value, AxisValue::Pnpm));
        assert!(pnpm.is_some(), "pnpm assertion should be extracted");
        // condition should be detected
        assert!(pnpm.unwrap().condition.is_some(), "condition should be captured");
    }

    #[test]
    fn package_manager_topic_required() {
        // "the npm registry is reliable" is NOT about choosing a package manager.
        let a = ext("The npm registry is reliable.");
        assert!(a.iter().all(|x| !matches!(x.axis, Axis::PackageManager)));
    }

    #[test]
    fn quote_style_single_vs_double() {
        let a = ext("Prefer single quotes for strings.");
        let b = ext("Always use double quotes for strings.");
        assert!(a.iter().any(|x| matches!(x.value, AxisValue::SingleQuote)));
        assert!(b.iter().any(|x| matches!(x.value, AxisValue::DoubleQuote)));
    }

    #[test]
    fn indentation_tabs() {
        let a = ext("Indent with tabs.");
        assert!(a.iter().any(|x| matches!(x.value, AxisValue::Tabs)));
    }
}
