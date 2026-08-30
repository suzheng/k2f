use crate::{canonical_json_string, collect_sorted_node_ids, sha256_hex, Manifest, SemanticNode};
use serde::Serialize;

/// Hash a single semantic node (tests and callers that do not have a Manifest).
pub fn hash_content(node: &SemanticNode) -> String {
    sha256_hex(
        canonical_json_string(node)
            .expect("Serialization failed")
            .as_bytes(),
    )
}

/// Hashes semantic inputs bound to the lock:
/// the main tree, running blocks, and the sorted stable node-ID set.
pub fn hash_manifest_semantic(manifest: &Manifest) -> String {
    #[derive(Serialize)]
    struct HashShape<'a> {
        root: &'a SemanticNode,
        running_blocks: &'a [crate::RunningBlockNode],
        node_ids: &'a [String],
    }

    let node_ids = collect_sorted_node_ids(manifest);
    let shape = HashShape {
        root: &manifest.root,
        running_blocks: &manifest.running_blocks,
        node_ids: &node_ids,
    };
    sha256_hex(
        canonical_json_string(&shape)
            .expect("Serialization failed")
            .as_bytes(),
    )
}
