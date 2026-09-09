use crate::error::PackageError;
use crate::includes::path::{validate_include_path, MAX_INCLUDE_DEPTH};
use k2f_core::SemanticNode;
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug)]
pub struct ResolvedContent {
    pub root: SemanticNode,
    pub include_map: BTreeMap<String, String>,
    pub referenced_paths: BTreeSet<String>,
}

pub fn resolve_content_files(
    root_raw: &str,
    content_files: &BTreeMap<String, String>,
) -> Result<ResolvedContent, PackageError> {
    let root_v: Value = serde_json::from_str(root_raw)
        .map_err(|e| PackageError::Other(format!("content/root.json: {e}")))?;
    let mut include_map = BTreeMap::new();
    let mut referenced_paths = BTreeSet::new();
    let expanded = resolve_value(
        &root_v,
        content_files,
        &mut include_map,
        &mut referenced_paths,
        0,
    )?;
    let root: SemanticNode = serde_json::from_value(expanded)
        .map_err(|e| PackageError::Other(format!("content/root.json: {e}")))?;
    Ok(ResolvedContent {
        root,
        include_map,
        referenced_paths,
    })
}

fn resolve_value(
    value: &Value,
    content_files: &BTreeMap<String, String>,
    include_map: &mut BTreeMap<String, String>,
    referenced_paths: &mut BTreeSet<String>,
    depth: usize,
) -> Result<Value, PackageError> {
    if depth > MAX_INCLUDE_DEPTH {
        return Err(PackageError::Other(format!(
            "INCLUDE_DEPTH: exceeded max include depth {MAX_INCLUDE_DEPTH}"
        )));
    }
    if let Some(include_path) = value.get("include").and_then(|v| v.as_str()) {
        if value.as_object().is_some_and(|m| m.len() == 1) {
            validate_include_path(include_path)?;
            if referenced_paths.contains(include_path) {
                return Err(PackageError::Other(format!(
                    "INCLUDE_CYCLE: '{include_path}' included more than once in the same branch"
                )));
            }
            referenced_paths.insert(include_path.to_string());
            let raw = content_files
                .get(include_path)
                .ok_or_else(|| PackageError::Other(format!("INCLUDE_MISSING: {include_path}")))?;
            let node_v: Value = serde_json::from_str(raw)
                .map_err(|e| PackageError::Other(format!("{include_path}: {e}")))?;
            let id = node_v
                .get("id")
                .and_then(|v| v.as_str())
                .ok_or_else(|| PackageError::Other(format!("{include_path}: missing node id")))?;
            if include_map
                .insert(id.to_string(), include_path.to_string())
                .is_some()
            {
                return Err(PackageError::Other(format!(
                    "INCLUDE_DUPLICATE_ID: node id '{id}' mapped to more than one include file"
                )));
            }
            return resolve_value(
                &node_v,
                content_files,
                include_map,
                referenced_paths,
                depth + 1,
            );
        }
    }
    match value {
        Value::Object(map) => {
            let mut out = serde_json::Map::new();
            for (k, v) in map {
                out.insert(
                    k.clone(),
                    resolve_value(v, content_files, include_map, referenced_paths, depth)?,
                );
            }
            Ok(Value::Object(out))
        }
        Value::Array(items) => {
            let mut out = Vec::with_capacity(items.len());
            for item in items {
                out.push(resolve_value(
                    item,
                    content_files,
                    include_map,
                    referenced_paths,
                    depth,
                )?);
            }
            Ok(Value::Array(out))
        }
        _ => Ok(value.clone()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn files(pairs: &[(&str, &str)]) -> BTreeMap<String, String> {
        pairs
            .iter()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect()
    }

    #[test]
    fn expands_include_stub() {
        let root = json!({
            "id": "root",
            "role": "document",
            "content": {
                "type": "container",
                "value": {
                    "children": [
                        { "include": "content/ch01.json" }
                    ]
                }
            }
        });
        let ch01 = json!({
            "id": "ch01",
            "role": "section",
            "content": { "type": "text", "value": "hello" }
        });
        let mut content = files(&[("content/ch01.json", &ch01.to_string())]);
        let mut include_map = BTreeMap::new();
        let mut referenced = BTreeSet::new();
        let expanded =
            resolve_value(&root, &content, &mut include_map, &mut referenced, 0).unwrap();
        assert_eq!(expanded["content"]["value"]["children"][0]["id"], "ch01");
        assert_eq!(
            include_map.get("ch01").map(String::as_str),
            Some("content/ch01.json")
        );
        content.insert("content/root.json".into(), root.to_string());
        let resolved = resolve_content_files(&root.to_string(), &content).unwrap();
        assert_eq!(resolved.root.id, "root");
    }

    #[test]
    fn rejects_missing_include_file() {
        let root = json!({
            "id": "root",
            "role": "document",
            "content": {
                "type": "container",
                "value": { "children": [{ "include": "content/missing.json" }] }
            }
        });
        let err = resolve_content_files(&root.to_string(), &BTreeMap::new()).unwrap_err();
        assert!(err.to_string().contains("INCLUDE_MISSING"));
    }
}
