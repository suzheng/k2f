use crate::SemanticNode;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum RunningBlockPosition {
    Header,
    Footer,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RunningBlockNode {
    pub position: RunningBlockPosition,
    pub node: SemanticNode,
}
