use k2f_core::{CanvasMode, PageConfig, RunningBlockNode};
use serde::{Deserialize, Serialize};

/// On-disk package manifest. Root content lives in `content/root.json`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PackageManifest {
    pub title: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub author: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub created_at: Option<i64>,
    pub canvas_mode: CanvasMode,
    pub page_config: PageConfig,
    pub engine_version: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub generated_by: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub running_blocks: Vec<RunningBlockNode>,
}
