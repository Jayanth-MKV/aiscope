//! Token counting via tiktoken-rs (cl100k_base — used by GPT-4 / Claude approximation).

use crate::model::Rule;
use std::sync::OnceLock;
use tiktoken_rs::{CoreBPE, cl100k_base};

fn bpe() -> &'static CoreBPE {
    static B: OnceLock<CoreBPE> = OnceLock::new();
    B.get_or_init(|| cl100k_base().expect("cl100k_base bundled with tiktoken-rs"))
}

/// Count tokens for a string. Cheap and synchronous.
pub fn count(text: &str) -> usize {
    bpe().encode_with_special_tokens(text).len()
}

/// Re-tokenize all rules with the real BPE (replaces the bytes/4 placeholder
/// from `scanner::approx_tokens`).
pub fn rescore(rules: &mut [Rule]) {
    for r in rules.iter_mut() {
        r.tokens = count(&r.text);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn counts_nonzero_for_real_text() {
        let n = count("Always use snake_case for variables.");
        assert!((5..=20).contains(&n), "unexpected token count: {n}");
    }
}
