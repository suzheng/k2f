use crate::{for_each_node, Manifest, NodeContent, SemanticNode, TableDataSource};
use std::collections::BTreeMap;

/// Table node id → asset path, before expand turns those tables inline.
pub fn table_asset_sources(manifest: &Manifest) -> BTreeMap<String, String> {
    let mut out = BTreeMap::new();
    collect(&manifest.root, &mut out);
    for rb in &manifest.running_blocks {
        collect(&rb.node, &mut out);
    }
    out
}

fn collect(node: &SemanticNode, out: &mut BTreeMap<String, String>) {
    for_each_node(node, &mut |n| {
        if let NodeContent::Table(spec) = &n.content {
            if let TableDataSource::Asset { source } = &spec.data {
                out.insert(n.id.clone(), source.clone());
            }
        }
    });
}
