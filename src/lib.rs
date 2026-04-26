//! Public library API — used by integration tests and (later) embedders.
//! The binary entry point is `src/main.rs`.

pub mod canon;
pub mod cmd;
pub mod detect;
pub mod diag;
pub mod extract;
pub mod model;
pub mod parse;
pub mod reason;
pub mod render;
pub mod scanner;

pub use cmd::build_bundle;
