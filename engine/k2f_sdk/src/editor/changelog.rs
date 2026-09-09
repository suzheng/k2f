use super::types::{Change, ChangelogEntry, ChangelogFile};
use crate::error::{AgentError, INVALID_ARGUMENT};
use k2f_core::LockFile;
#[cfg(not(target_arch = "wasm32"))]
use std::time::{SystemTime, UNIX_EPOCH};

pub fn parse_changelog(json: &str) -> Result<ChangelogFile, AgentError> {
    serde_json::from_str(json)
        .map_err(|e| AgentError::new(INVALID_ARGUMENT, format!("changelog: {e}")))
}

pub fn append_entry(
    changelog_json: &str,
    agent: Option<&str>,
    content_hash_before: &str,
    content_hash_after: &str,
    appearance_hash_after: &str,
    changes: &[Change],
) -> Result<String, AgentError> {
    if changes.is_empty() {
        return Ok(changelog_json.to_string());
    }
    let mut file = parse_changelog(changelog_json)?;
    let revision = file.entries.len() as u32 + 1;
    let summary = summary_from_changes(changes);
    file.entries.push(ChangelogEntry {
        revision,
        agent: agent.map(str::to_string),
        timestamp: unix_timestamp_iso(),
        summary,
        content_hash_before: content_hash_before.to_string(),
        content_hash_after: content_hash_after.to_string(),
        appearance_hash_after: appearance_hash_after.to_string(),
        changes: changes.to_vec(),
    });
    serde_json::to_string(&file)
        .map_err(|e| AgentError::new(INVALID_ARGUMENT, format!("serialize changelog: {e}")))
}

pub fn lock_hashes(lock_json: &str) -> Result<(String, String), AgentError> {
    let lock: LockFile = serde_json::from_str(lock_json)
        .map_err(|e| AgentError::new(INVALID_ARGUMENT, format!("lock: {e}")))?;
    Ok((lock.content_hash, lock.appearance_hash))
}

fn summary_from_changes(changes: &[Change]) -> String {
    if let Some(first) = changes.first() {
        format!("{} {}", op_label(&first.op), first.id)
    } else {
        "save".into()
    }
}

fn unix_timestamp_iso() -> String {
    #[cfg(target_arch = "wasm32")]
    {
        let ms = js_sys::Date::now();
        return format!("{}", (ms / 1000.0) as u64);
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let secs = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        format!("{secs}")
    }
}

fn op_label(op: &super::types::ChangeOp) -> &'static str {
    use super::types::ChangeOp;
    match op {
        ChangeOp::Add => "add",
        ChangeOp::Delete => "delete",
        ChangeOp::Update => "update",
    }
}
