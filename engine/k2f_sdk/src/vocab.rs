use crate::error::{AgentError, INVALID_ARGUMENT, UNKNOWN_ROLE};
use k2f_core::ThemeVocab;

pub fn ensure_role(theme_json: &str, role: &str, variant: Option<&str>) -> Result<(), AgentError> {
    let vocab: ThemeVocab = serde_json::from_str(theme_json)
        .map_err(|e| AgentError::new(INVALID_ARGUMENT, format!("theme: {e}")))?;
    let entry = vocab
        .roles
        .get(role)
        .ok_or_else(|| AgentError::new(UNKNOWN_ROLE, format!("unknown role '{role}'")))?;
    if let Some(v) = variant {
        if !entry.variants.contains_key(v) {
            return Err(AgentError::new(
                UNKNOWN_ROLE,
                format!("unknown variant '{v}' for role '{role}'"),
            ));
        }
    }
    Ok(())
}
