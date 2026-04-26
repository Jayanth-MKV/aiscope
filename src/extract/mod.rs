//! Layer 3 — extraction subsystem.
//!
//! Turns canonicalized statements into typed `Assertion`s on known `Axis`es.
//! Three stages, each runnable independently, each producing assertions
//! tagged with a confidence and an `ExtractionOrigin`:
//!
//! 1. [`pattern`]   — deterministic regex/keyword rules. Confidence ≥ 0.95.
//! 2. [`embedding`] — semantic similarity to canonical exemplars (optional).
//! 3. [`cross_encoder`] — pairwise verification (optional).
//!
//! In v0.1 only [`pattern`] is wired in. The other two activate behind the
//! `semantic` feature.

pub mod pattern;
