use super::AssetsMap;
use crate::{canonical_json_string, NodeContent, SemanticNode, TableDataSource};
use std::collections::BTreeMap;

/// Write expanded inline rows back to their asset files and restore `content.data.type = asset`.
pub fn collapse_tables_to_assets(
    root: &mut SemanticNode,
    sources: &BTreeMap<String, String>,
    assets: &mut AssetsMap,
) -> Result<(), String> {
    collapse_node(root, sources, assets)
}

fn collapse_node(
    node: &mut SemanticNode,
    sources: &BTreeMap<String, String>,
    assets: &mut AssetsMap,
) -> Result<(), String> {
    match &mut node.content {
        NodeContent::Container { children } => {
            for child in children {
                collapse_node(child, sources, assets)?;
            }
        }
        NodeContent::Table(spec) => {
            if let TableDataSource::Inline { rows } = &mut spec.data {
                for row in rows.iter_mut() {
                    for cell in row {
                        collapse_node(cell, sources, assets)?;
                    }
                }
            }
        }
        _ => {}
    }

    let Some(source) = sources.get(&node.id) else {
        return Ok(());
    };
    let NodeContent::Table(spec) = &mut node.content else {
        return Ok(());
    };
    let TableDataSource::Inline { rows } = &spec.data else {
        return Ok(());
    };
    let payload = serde_json::json!({ "rows": rows });
    let bytes = canonical_json_string(&payload)
        .map_err(|e| format!("table asset '{source}': {e}"))?
        .into_bytes();
    assets.insert(source.clone(), bytes);
    spec.data = TableDataSource::Asset {
        source: source.clone(),
    };
    Ok(())
}
