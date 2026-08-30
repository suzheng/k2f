mod collapse;
mod sources;

use crate::{Manifest, NodeContent, SemanticNode, TableDataSource, TableSpec};
use serde::de::DeserializeOwned;
use serde::Deserialize;
use std::collections::HashMap;

pub use collapse::collapse_tables_to_assets;
pub use sources::table_asset_sources;

pub type AssetsMap = HashMap<String, Vec<u8>>;

#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum TableAssetFormat {
    RowsOnly(Vec<Vec<SemanticNode>>),
    Object { rows: Vec<Vec<SemanticNode>> },
}

fn parse_json_unbounded<T: DeserializeOwned>(bytes: &[u8]) -> Result<T, String> {
    let s = std::str::from_utf8(bytes).map_err(|e| format!("Asset is not valid UTF-8: {e}"))?;
    let mut deserializer = serde_json::Deserializer::from_str(s);
    deserializer.disable_recursion_limit();
    T::deserialize(&mut deserializer).map_err(|e| format!("Asset JSON error: {e}"))
}

fn load_table_rows_from_assets(
    source: &str,
    assets: &AssetsMap,
) -> Result<Vec<Vec<SemanticNode>>, String> {
    let bytes = assets
        .get(source)
        .ok_or_else(|| format!("Missing required asset '{source}' for table data"))?;
    let parsed: TableAssetFormat = parse_json_unbounded(bytes)?;
    Ok(match parsed {
        TableAssetFormat::RowsOnly(rows) => rows,
        TableAssetFormat::Object { rows } => rows,
    })
}

pub fn semantic_tree_needs_assets(root: &SemanticNode) -> bool {
    match &root.content {
        NodeContent::Container { children } => children.iter().any(semantic_tree_needs_assets),
        NodeContent::Table(TableSpec { data, .. }) => match data {
            TableDataSource::Inline { rows } => rows
                .iter()
                .flat_map(|r| r.iter())
                .any(semantic_tree_needs_assets),
            TableDataSource::Asset { .. } => true,
        },
        _ => false,
    }
}

pub fn expand_manifest_tables_with_assets(
    manifest: &Manifest,
    assets: &AssetsMap,
) -> Result<Manifest, String> {
    let mut out = manifest.clone();
    out.root = expand_node(&manifest.root, assets)?;
    Ok(out)
}

fn expand_node(node: &SemanticNode, assets: &AssetsMap) -> Result<SemanticNode, String> {
    let mut out = node.clone();
    out.content = match &node.content {
        NodeContent::Container { children } => NodeContent::Container {
            children: children
                .iter()
                .map(|c| expand_node(c, assets))
                .collect::<Result<Vec<_>, _>>()?,
        },
        NodeContent::Table(spec) => NodeContent::Table(expand_table_spec(&node.id, spec, assets)?),
        other => other.clone(),
    };
    Ok(out)
}

fn expand_table_spec(
    node_id: &str,
    spec: &TableSpec,
    assets: &AssetsMap,
) -> Result<TableSpec, String> {
    let mut out = spec.clone();
    let rows = match &spec.data {
        TableDataSource::Inline { rows } => rows.clone(),
        TableDataSource::Asset { source } => load_table_rows_from_assets(source, assets)
            .map_err(|e| format!("Table '{node_id}' failed to load asset-backed rows: {e}"))?,
    };
    let expanded_rows = rows
        .into_iter()
        .map(|row| {
            row.into_iter()
                .map(|cell| expand_node(&cell, assets))
                .collect::<Result<Vec<_>, _>>()
        })
        .collect::<Result<Vec<_>, _>>()?;
    out.data = TableDataSource::Inline {
        rows: expanded_rows,
    };
    Ok(out)
}
