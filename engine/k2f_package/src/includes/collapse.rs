use crate::error::PackageError;
use crate::paths;
use k2f_core::{canonical_json_string, NodeContent, SemanticNode};
use serde_json::{json, Value};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug)]
pub struct CollapsedContent {
    pub files: BTreeMap<String, String>,
    #[allow(dead_code)]
    pub include_map: BTreeMap<String, String>,
}

pub fn collapse_content_tree(
    root: &SemanticNode,
    include_map: &BTreeMap<String, String>,
) -> Result<CollapsedContent, PackageError> {
    let mut files = BTreeMap::new();
    let mut active_map = BTreeMap::new();
    let root_v = collapse_node(root, include_map, &mut files, &mut active_map)?;
    files.insert(
        paths::ROOT.to_string(),
        canonical_json_string(&root_v).map_err(|e| PackageError::Other(e.to_string()))?,
    );
    Ok(CollapsedContent {
        files,
        include_map: active_map,
    })
}

fn collapse_node(
    node: &SemanticNode,
    include_map: &BTreeMap<String, String>,
    files: &mut BTreeMap<String, String>,
    active_map: &mut BTreeMap<String, String>,
) -> Result<Value, PackageError> {
    if let Some(path) = include_map.get(&node.id) {
        let subtree = node_to_value(node, include_map, files, active_map)?;
        files.insert(
            path.clone(),
            canonical_json_string(&subtree).map_err(|e| PackageError::Other(e.to_string()))?,
        );
        active_map.insert(node.id.clone(), path.clone());
        return Ok(json!({ "include": path }));
    }
    node_to_value(node, include_map, files, active_map)
}

fn node_to_value(
    node: &SemanticNode,
    include_map: &BTreeMap<String, String>,
    files: &mut BTreeMap<String, String>,
    active_map: &mut BTreeMap<String, String>,
) -> Result<Value, PackageError> {
    let mut v = serde_json::to_value(node).map_err(|e| PackageError::Other(e.to_string()))?;
    if let NodeContent::Container { children } = &node.content {
        let mut out_children = Vec::with_capacity(children.len());
        for child in children {
            out_children.push(collapse_node(child, include_map, files, active_map)?);
        }
        v["content"]["value"]["children"] = json!(out_children);
    }
    Ok(v)
}

pub fn collect_orphan_content_paths(
    content_dir_files: &BTreeSet<String>,
    referenced: &BTreeSet<String>,
) -> Vec<String> {
    content_dir_files
        .iter()
        .filter(|p| *p != paths::ROOT && !referenced.contains(*p))
        .cloned()
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use k2f_core::SemanticNode;

    #[test]
    fn roundtrip_preserves_include_stub() {
        let ch01 = SemanticNode {
            id: "ch01".into(),
            role: "section".into(),
            content: NodeContent::Text("hello".into()),
            ..Default::default()
        };
        let root = SemanticNode {
            id: "root".into(),
            role: "document".into(),
            content: NodeContent::Container {
                children: vec![ch01.clone()],
            },
            ..Default::default()
        };
        let mut include_map = BTreeMap::new();
        include_map.insert("ch01".into(), "content/ch01.json".into());
        let collapsed = collapse_content_tree(&root, &include_map).unwrap();
        assert!(collapsed.files.contains_key("content/ch01.json"));
        let root_json: serde_json::Value =
            serde_json::from_str(&collapsed.files[paths::ROOT]).unwrap();
        assert_eq!(
            root_json["content"]["value"]["children"][0]["include"],
            "content/ch01.json"
        );
    }
}
