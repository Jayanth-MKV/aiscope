//! JSON renderer. Pure passthrough — the model derives Serialize.

use crate::model::ContextBundle;
use anyhow::Result;

pub fn render(bundle: &ContextBundle) -> Result<String> {
    Ok(serde_json::to_string_pretty(bundle)?)
}
