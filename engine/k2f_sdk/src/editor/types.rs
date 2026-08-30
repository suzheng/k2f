use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OutlineNode {
    pub id: String,
    pub role: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub preview: Option<String>,
    pub children: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ChangeOp {
    Add,
    Delete,
    Update,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Change {
    pub id: String,
    pub op: ChangeOp,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text_before: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text_after: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub role_before: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub role_after: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChangelogFile {
    pub entries: Vec<ChangelogEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChangelogEntry {
    pub revision: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub agent: Option<String>,
    pub timestamp: String,
    pub summary: String,
    pub content_hash_before: String,
    pub content_hash_after: String,
    pub appearance_hash_after: String,
    pub changes: Vec<Change>,
}
