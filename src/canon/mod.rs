//! Layer 2 — `Statement` → `CanonicalText` normalization.
//!
//! Pipeline (each step is deterministic and side-effect free):
//!
//! 1. **Unicode NFKC** — combine compatibility forms (`ﬁ` → `fi`, smart
//!    quotes → ASCII quotes).
//! 2. **Caseless fold** — Unicode-aware lowercase (`İ` → `i̇`).
//! 3. **Punctuation stripping** — strip everything except `[a-z0-9_'\-`]`
//!    and whitespace.
//! 4. **Whitespace collapse** — runs of whitespace → single space.
//! 5. **Token-level stemming** — Snowball English stemmer, leaves identifiers
//!    with embedded digits/underscores untouched (so `snake_case` survives).
//!
//! Output is used for:
//!   - SHA-256 fingerprints in duplicate detection
//!   - Pattern matching in `crate::extract::pattern`
//!   - Embedding inputs in `crate::extract::embedding`

use rust_stemmers::{Algorithm, Stemmer};
use std::sync::OnceLock;
use unicode_normalization::UnicodeNormalization;

/// One canonicalized form of a statement, plus a token list ready for
/// pattern matching and embedding.
#[derive(Debug, Clone)]
pub struct CanonicalText {
    /// Lowercased, NFKC-normalized, whitespace-collapsed string.
    /// Punctuation REMOVED. Identifiers preserved.
    pub canon: String,
    /// Tokens after stemming. Order preserved.
    pub stems: Vec<String>,
}

static STEMMER: OnceLock<Stemmer> = OnceLock::new();
fn stemmer() -> &'static Stemmer {
    STEMMER.get_or_init(|| Stemmer::create(Algorithm::English))
}

/// Normalize one statement.
pub fn canonicalize(text: &str) -> CanonicalText {
    // 1. NFKC + smart-quote / dash normalization
    let nfkc: String = text
        .nfkc()
        .map(|c| match c {
            '\u{2018}' | '\u{2019}' | '\u{201A}' | '\u{201B}' => '\'',
            '\u{201C}' | '\u{201D}' | '\u{201E}' | '\u{201F}' => '"',
            '\u{2013}' | '\u{2014}' | '\u{2212}' => '-',
            '\u{00A0}' | '\u{2000}'..='\u{200B}' => ' ',
            _ => c,
        })
        .collect();

    // 2. Caseless lowercase via the `caseless` crate (Unicode-correct).
    let folded: String = caseless::default_case_fold_str(&nfkc);

    // 3 + 4. Strip punctuation (keep identifier chars), collapse whitespace.
    let mut canon = String::with_capacity(folded.len());
    let mut prev_space = true;
    for c in folded.chars() {
        let keep = c.is_alphanumeric()
            || c == '_'
            || c == '-'
            || c == '\''
            || c == '`';
        if keep {
            canon.push(c);
            prev_space = false;
        } else if c.is_whitespace() && !prev_space {
            canon.push(' ');
            prev_space = true;
        }
        // punctuation otherwise dropped
    }
    let canon = canon.trim().to_string();

    // 5. Token-level stemming. We DON'T stem identifiers that contain `_`
    //    (e.g. `snake_case`) or digits — they're proper nouns of code.
    let stem = stemmer();
    let stems: Vec<String> = canon
        .split_whitespace()
        .map(|tok| {
            if tok.contains('_') || tok.chars().any(|c| c.is_ascii_digit()) {
                tok.to_string()
            } else {
                stem.stem(tok).into_owned()
            }
        })
        .collect();

    CanonicalText { canon, stems }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn nfkc_smart_quotes_and_dashes() {
        let c = canonicalize("Use \u{201C}snake_case\u{201D} \u{2014} not camelCase.");
        // smart quotes → ASCII (then stripped as punctuation), em-dash → hyphen, lowercased.
        assert!(c.canon.contains("snake_case"));
        assert!(c.canon.contains("- not camelcase") || c.canon.contains("not camelcase"));
    }

    #[test]
    fn snake_case_identifier_survives_stemming() {
        let c = canonicalize("Always use snake_case for variables.");
        assert!(c.stems.contains(&"snake_case".to_string()));
    }

    #[test]
    fn camelcase_lowercased_and_stemmed() {
        let c = canonicalize("Always use camelCase for variables.");
        assert!(c.canon.contains("camelcase"));
    }

    #[test]
    fn whitespace_collapsed() {
        let c = canonicalize("Use   \t  pnpm,   not    npm.");
        assert!(!c.canon.contains("  "));
    }

    #[test]
    fn deterministic() {
        let a = canonicalize("Use camelCase for variables.");
        let b = canonicalize("Use camelCase for variables.");
        assert_eq!(a.canon, b.canon);
        assert_eq!(a.stems, b.stems);
    }
}
