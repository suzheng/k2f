use crate::error::ToolError;
use crate::ops::K2fState;
use crate::session::SessionStore;
use rmcp::model::{ReadResourceResponse, Resource, ResourceContents, ResourceTemplate};
use serde_json::json;
use uuid::Uuid;

pub fn list_session_resources(state: &K2fState) -> Vec<Resource> {
    state
        .sessions
        .ids()
        .into_iter()
        .map(session_resource)
        .collect()
}

pub fn resource_templates() -> Vec<ResourceTemplate> {
    vec![
        ResourceTemplate::new("k2f://session/{session_id}", "k2f_session")
            .with_description("Session metadata (label, kind, baseline). No lock JSON.")
            .with_mime_type("application/json"),
        ResourceTemplate::new("k2f://session/{session_id}/node/{node_id}", "k2f_node")
            .with_description("One semantic node by stable id.")
            .with_mime_type("application/json"),
    ]
}

pub fn read_resource(state: &K2fState, uri: &str) -> Result<ReadResourceResponse, ToolError> {
    if let Some(rest) = uri.strip_prefix("k2f://session/") {
        if let Some((session_raw, node_part)) = rest.split_once("/node/") {
            let id = SessionStore::parse_id(session_raw)?;
            let node = state.get_node(&id.to_string(), node_part)?;
            let body = serde_json::to_string(&node)
                .map_err(|e| ToolError::internal(e.to_string()))?;
            return Ok(ReadResourceResponse::Complete(
                rmcp::model::ReadResourceResult::new(vec![ResourceContents::text(
                    body,
                    uri.to_string(),
                )]),
            ));
        }
        let id = SessionStore::parse_id(rest)?;
        let session = state.sessions.get(&id)?;
        let body = json!({
            "session_id": id.to_string(),
            "label": session.meta.label,
            "kind": session.meta.kind,
            "source_path": session.meta.source_path.as_ref().map(|p| p.display().to_string()),
        })
        .to_string();
        return Ok(ReadResourceResponse::Complete(
            rmcp::model::ReadResourceResult::new(vec![ResourceContents::text(
                body,
                uri.to_string(),
            )]),
        ));
    }
    Err(ToolError::not_found(format!("unknown resource uri {uri}")))
}

fn session_resource(id: Uuid) -> Resource {
    Resource::new(format!("k2f://session/{id}"), format!("session {id}"))
        .with_description("K2F editor/document session handle")
        .with_mime_type("application/json")
}
