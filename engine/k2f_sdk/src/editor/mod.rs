mod changelog;
mod diff;
mod outline;
mod types;

pub use types::{Change, ChangeOp, ChangelogEntry, ChangelogFile, OutlineNode};

use self::changelog::{append_entry, lock_hashes};
use self::diff::diff_trees;
use self::outline::build_outline;
use crate::error::{
    AgentError, CONTENT_HASH_MISMATCH, DUPLICATE_ID, INVALID_ARGUMENT, INVALID_ID, UNKNOWN_ID,
    WRONG_CONTENT,
};
use crate::lock::relock;
use crate::nodes::text_node;
use crate::PdfScale;
use crate::vocab::ensure_role;
use k2f_core::{
    apply_role, clipboard_of, find_in_trees, find_in_trees_mut, hash_manifest_semantic,
    insert_child, is_valid_node_id, node_text, remove_node, replace_node_text, search_trees,
    selection_of, tree_contains_id, validate_manifest_node_ids, validate_manifest_running_blocks,
    validate_semantic_tree, validate_semantic_tree_with_theme_vocab, Clipboard, RunningBlockNode,
    RunningBlockPosition, Selection, SemanticNode, ThemeVocab,
};
use k2f_package::{load_dir, pack_bytes, unpack_bytes, write_dir, Package, WriteDirOpts};
use serde_json::Value;
use std::collections::BTreeMap;
use std::path::Path;

/// Surgical editor: mutate the semantic tree, then relock. Does not edit lock x/y.
pub struct Editor {
    package: Package,
    suggestions: BTreeMap<String, String>,
    table_assets: BTreeMap<String, String>,
    base_root: SemanticNode,
    base_running: Vec<RunningBlockNode>,
    base_content_hash: String,
    generated_by: Option<String>,
}

impl Editor {
    pub fn open(bytes: &[u8]) -> Result<Self, AgentError> {
        Self::from_package(unpack_bytes(bytes).map_err(AgentError::from)?)
    }

    pub fn open_dir(dir: &Path) -> Result<Self, AgentError> {
        Self::from_package(load_dir(dir).map_err(AgentError::from)?)
    }

    pub fn open_template(template: impl AsRef<Path>) -> Result<Self, AgentError> {
        let mut package = crate::templates::load_package(template)?;
        package.lock_json = None;
        package.signatures_json = None;
        Self::from_package(package)
    }

    fn from_package(mut package: Package) -> Result<Self, AgentError> {
        let table_assets = package.expand_table_assets().map_err(AgentError::from)?;
        let base_content_hash = package
            .lock_json
            .as_ref()
            .and_then(|j| lock_hashes(j).ok())
            .map(|(c, _)| c)
            .unwrap_or_else(|| hash_manifest_semantic(&package.engine_manifest()));
        let base_root = package.root.clone();
        let base_running = package.manifest.running_blocks.clone();
        Ok(Self {
            package,
            suggestions: BTreeMap::new(),
            table_assets,
            base_root,
            base_running,
            base_content_hash,
            generated_by: None,
        })
    }

    pub fn set_generated_by(&mut self, id: impl Into<String>) {
        self.generated_by = Some(id.into());
    }

    pub fn outline(&self) -> Vec<OutlineNode> {
        let mut nodes = build_outline(&self.package.root);
        for rb in &self.package.manifest.running_blocks {
            nodes.extend(build_outline(&rb.node));
        }
        nodes
    }

    pub fn outline_json(&self) -> Result<String, AgentError> {
        serde_json::to_string(&self.outline())
            .map_err(|e| AgentError::new(INVALID_ARGUMENT, format!("serialize outline: {e}")))
    }

    pub fn diff(&self) -> Vec<Change> {
        diff_trees(
            &self.base_root,
            &self.base_running,
            &self.package.root,
            &self.package.manifest.running_blocks,
        )
    }

    pub fn diff_json(&self) -> Result<String, AgentError> {
        serde_json::to_string(&self.diff())
            .map_err(|e| AgentError::new(INVALID_ARGUMENT, format!("serialize diff: {e}")))
    }

    pub fn insert_node(
        &mut self,
        parent_id: &str,
        index: usize,
        node_json: &str,
    ) -> Result<(), AgentError> {
        let value: Value = serde_json::from_str(node_json)
            .map_err(|e| AgentError::new(INVALID_ARGUMENT, format!("node json: {e}")))?;
        reject_coordinate_fields(&value)?;
        let node: SemanticNode = serde_json::from_value(value)
            .map_err(|e| AgentError::new(INVALID_ARGUMENT, format!("node shape: {e}")))?;
        if node.id.is_empty() || !is_valid_node_id(&node.id) {
            return Err(AgentError::new(
                INVALID_ID,
                format!("invalid node id '{}'", node.id),
            ));
        }
        if tree_contains_id(
            &self.package.root,
            &self.package.manifest.running_blocks,
            &node.id,
        ) {
            return Err(AgentError::new(
                DUPLICATE_ID,
                format!("duplicate node id '{}'", node.id),
            ));
        }
        let agent_value = crate::agent_json::to_agent_value(&node)?;
        crate::validate_agent_json(&agent_value)?;
        ensure_role(
            &self.package.theme_json,
            &node.role,
            node.variant.as_deref(),
        )?;
        insert_child(
            &mut self.package.root,
            &mut self.package.manifest.running_blocks,
            parent_id,
            index,
            node,
        )?;
        Ok(())
    }

    pub fn delete_node(&mut self, id: &str) -> Result<(), AgentError> {
        remove_node(
            &mut self.package.root,
            &mut self.package.manifest.running_blocks,
            id,
        )?;
        Ok(())
    }

    pub fn get_node_json(&self, id: &str) -> Result<String, AgentError> {
        let node = self.node(id)?;
        serde_json::to_string(&crate::agent_json::to_agent_value(node)?)
            .map_err(|e| AgentError::new(INVALID_ARGUMENT, format!("serialize node: {e}")))
    }

    pub fn selection(&self, id: &str) -> Result<Selection, AgentError> {
        selection_of(self.node(id)?)
            .ok_or_else(|| AgentError::new(UNKNOWN_ID, format!("no node with id '{id}'")))
    }

    pub fn selection_json(&self, id: &str) -> Result<String, AgentError> {
        serde_json::to_string(&self.selection(id)?)
            .map_err(|e| AgentError::new(INVALID_ARGUMENT, format!("serialize: {e}")))
    }

    pub fn clipboard(&self, id: &str) -> Result<Clipboard, AgentError> {
        clipboard_of(self.node(id)?)
            .ok_or_else(|| AgentError::new(UNKNOWN_ID, format!("no node with id '{id}'")))
    }

    pub fn clipboard_json(&self, id: &str) -> Result<String, AgentError> {
        serde_json::to_string(&self.clipboard(id)?)
            .map_err(|e| AgentError::new(INVALID_ARGUMENT, format!("serialize: {e}")))
    }

    pub fn search(&self, query: &str) -> Vec<String> {
        search_trees(
            &self.package.root,
            &self.package.manifest.running_blocks,
            query,
        )
    }

    pub fn node_text(&self, id: &str) -> Result<String, AgentError> {
        node_text(self.node(id)?)
            .map(str::to_string)
            .ok_or_else(|| AgentError::new(WRONG_CONTENT, format!("node '{id}' is not text")))
    }

    pub fn set_running_header(&mut self, text: &str) -> Result<(), AgentError> {
        self.set_running_block(
            RunningBlockPosition::Header,
            "running.header",
            "running_header",
            text,
        )
    }

    pub fn set_running_footer(&mut self, text: &str) -> Result<(), AgentError> {
        self.set_running_block(
            RunningBlockPosition::Footer,
            "running.footer",
            "running_footer",
            text,
        )
    }

    pub fn replace_text(&mut self, id: &str, text: &str) -> Result<(), AgentError> {
        replace_node_text(
            &mut self.package.root,
            &mut self.package.manifest.running_blocks,
            id,
            text,
        )?;
        Ok(())
    }

    pub fn set_role(
        &mut self,
        id: &str,
        role: &str,
        variant: Option<&str>,
    ) -> Result<(), AgentError> {
        ensure_role(&self.package.theme_json, role, variant)?;
        let node = find_in_trees_mut(
            &mut self.package.root,
            &mut self.package.manifest.running_blocks,
            id,
        )
        .ok_or_else(|| AgentError::new(UNKNOWN_ID, format!("no node with id '{id}'")))?;
        apply_role(node, role, variant);
        Ok(())
    }

    pub fn suggest(&mut self, id: &str, text: &str) -> Result<(), AgentError> {
        let _ = self.node_text(id)?;
        self.suggestions.insert(id.to_string(), text.to_string());
        Ok(())
    }

    pub fn reject_suggestion(&mut self, id: &str) -> Result<(), AgentError> {
        self.suggestions
            .remove(id)
            .ok_or_else(|| AgentError::new(UNKNOWN_ID, format!("no suggestion for '{id}'")))?;
        Ok(())
    }

    pub fn accept_suggestion(&mut self, id: &str) -> Result<(), AgentError> {
        let text = self
            .suggestions
            .remove(id)
            .ok_or_else(|| AgentError::new(UNKNOWN_ID, format!("no suggestion for '{id}'")))?;
        self.replace_text(id, &text)
    }

    pub fn suggestions_json(&self) -> Result<String, AgentError> {
        serde_json::to_string(&self.suggestions)
            .map_err(|e| AgentError::new(INVALID_ARGUMENT, format!("serialize: {e}")))
    }

    pub fn export_pdf_bytes(&self) -> Result<Vec<u8>, AgentError> {
        self.export_pdf_bytes_at(PdfScale::DEFAULT)
    }

    pub fn export_pdf_bytes_at(&self, scale: PdfScale) -> Result<Vec<u8>, AgentError> {
        crate::export_pdf_at(&pack_bytes(&self.package).map_err(AgentError::from)?, scale)
    }

    pub fn save_with(
        &mut self,
        expected_content_hash: Option<&str>,
    ) -> Result<Vec<u8>, AgentError> {
        if let Some(expected) = expected_content_hash {
            if expected != self.base_content_hash {
                return Err(AgentError::new(
                    CONTENT_HASH_MISMATCH,
                    format!(
                        "expected content_hash {expected}, baseline is {}",
                        self.base_content_hash
                    ),
                ));
            }
        }
        let changes = self.diff();
        let content_hash_before = self.base_content_hash.clone();
        self.package
            .collapse_table_assets(&self.table_assets)
            .map_err(AgentError::from)?;
        relock(&mut self.package)?;
        if !changes.is_empty() {
            let (content_hash_after, appearance_hash_after) =
                lock_hashes(self.package.lock_json.as_ref().ok_or_else(|| {
                    AgentError::new(INVALID_ARGUMENT, "relock did not write lock")
                })?)?;
            self.package.changelog_json = append_entry(
                &self.package.changelog_json,
                self.generated_by.as_deref(),
                &content_hash_before,
                &content_hash_after,
                &appearance_hash_after,
                &changes,
            )?;
        }
        let bytes = pack_bytes(&self.package).map_err(AgentError::from)?;
        self.base_root = self.package.root.clone();
        self.base_running = self.package.manifest.running_blocks.clone();
        self.base_content_hash = hash_manifest_semantic(&self.package.engine_manifest());
        self.package
            .expand_table_assets()
            .map_err(AgentError::from)?;
        Ok(bytes)
    }

    pub fn save_bytes(&mut self) -> Result<Vec<u8>, AgentError> {
        self.save_with(None)
    }

    /// Write author source files (no lock, no embedded schemas). Does not recompile.
    pub fn save_dir(&self, dir: &Path) -> Result<(), AgentError> {
        self.validate_package()?;
        write_dir(
            &self.package,
            dir,
            WriteDirOpts {
                include_lock: false,
                include_schema: false,
            },
        )
        .map_err(AgentError::from)
    }

    pub fn validate_package(&self) -> Result<(), AgentError> {
        let manifest = self.package.engine_manifest();
        validate_semantic_tree(&manifest.root)?;
        let vocab: ThemeVocab = serde_json::from_str(&self.package.theme_json)
            .map_err(|e| AgentError::new(INVALID_ARGUMENT, format!("theme: {e}")))?;
        validate_semantic_tree_with_theme_vocab(&manifest.root, &vocab)?;
        validate_manifest_running_blocks(&manifest, &vocab)?;
        validate_manifest_node_ids(&manifest)?;
        Ok(())
    }

    fn node(&self, id: &str) -> Result<&SemanticNode, AgentError> {
        find_in_trees(
            &self.package.root,
            &self.package.manifest.running_blocks,
            id,
        )
        .ok_or_else(|| AgentError::new(UNKNOWN_ID, format!("no node with id '{id}'")))
    }

    fn set_running_block(
        &mut self,
        position: RunningBlockPosition,
        id: &str,
        role: &str,
        text: &str,
    ) -> Result<(), AgentError> {
        ensure_role(&self.package.theme_json, role, None)?;
        self.package
            .manifest
            .running_blocks
            .retain(|rb| rb.position != position);
        if tree_contains_id(
            &self.package.root,
            &self.package.manifest.running_blocks,
            id,
        ) {
            return Err(AgentError::new(
                DUPLICATE_ID,
                format!("node id '{id}' already exists"),
            ));
        }
        self.package.manifest.running_blocks.push(RunningBlockNode {
            position,
            node: text_node(id, role, text),
        });
        Ok(())
    }
}

fn reject_coordinate_fields(value: &Value) -> Result<(), AgentError> {
    match value {
        Value::Object(map) => {
            if map.contains_key("x") || map.contains_key("y") {
                return Err(AgentError::new(
                    INVALID_ARGUMENT,
                    "node json must not contain x/y coordinates",
                ));
            }
            for v in map.values() {
                reject_coordinate_fields(v)?;
            }
        }
        Value::Array(items) => {
            for v in items {
                reject_coordinate_fields(v)?;
            }
        }
        _ => {}
    }
    Ok(())
}
