use crate::error::{AgentError, DUPLICATE_ID, INVALID_ID, UNCLOSED_SECTION, UNKNOWN_ID, WRONG_CONTENT};
use crate::lock::write_lock;
use crate::nodes::{image_node, text_node};
use crate::page::PageSize;
use crate::units::{mm_to_pt, scale_height};
use k2f_core::{
    find_in_trees, find_node_mut, is_valid_node_id, validate_manifest_node_ids,
    validate_manifest_running_blocks, validate_semantic_tree, validate_semantic_tree_with_theme_vocab,
    NodeContent, RunningBlockNode, RunningBlockPosition, SemanticNode, ThemeVocab,
};
use k2f_package::{pack_bytes, Package};
use std::path::Path;

const ROOT_ID: &str = "root";

/// Crate-private tree builder for Markdown import only.
pub(crate) struct MarkdownBuilder {
    package: Package,
    sections: Vec<String>,
    font_override: Option<Vec<u8>>,
}

impl MarkdownBuilder {
    pub(crate) fn from_template(
        template: impl AsRef<Path>,
        title: impl Into<String>,
        page_size: PageSize,
    ) -> Result<Self, AgentError> {
        let mut package = crate::templates::load_package(template)?;
        package.manifest.title = title.into();
        package.manifest.page_config = page_size.page_config();
        package.lock_json = None;
        package.signatures_json = None;
        Ok(Self {
            package,
            sections: vec![ROOT_ID.to_string()],
            font_override: None,
        })
    }

    pub(crate) fn set_font_bytes(&mut self, bytes: Vec<u8>) {
        self.font_override = Some(bytes);
    }

    pub(crate) fn set_running_header(&mut self, text: &str) -> Result<(), AgentError> {
        self.set_running(
            RunningBlockPosition::Header,
            "running.header",
            "running_header",
            text,
        )
    }

    pub(crate) fn set_running_footer(&mut self, text: &str) -> Result<(), AgentError> {
        self.set_running(
            RunningBlockPosition::Footer,
            "running.footer",
            "running_footer",
            text,
        )
    }

    pub(crate) fn finish(mut self) -> Result<Vec<u8>, AgentError> {
        self.validate_inner()?;
        if let Some(bytes) = self.font_override.take() {
            if let Some((path, _)) = self.package.fonts.iter_mut().next() {
                let path = path.clone();
                self.package.fonts.insert(path, bytes);
            }
        }
        write_lock(&mut self.package)?;
        Ok(pack_bytes(&self.package).map_err(AgentError::from)?)
    }

    fn validate_inner(&self) -> Result<(), AgentError> {
        if self.sections.len() != 1 {
            return Err(AgentError::new(
                UNCLOSED_SECTION,
                "unclosed section stack in markdown import",
            ));
        }
        let manifest = self.package.engine_manifest();
        validate_semantic_tree(&manifest.root)?;
        let vocab: ThemeVocab = serde_json::from_str(&self.package.theme_json)
            .map_err(|e| AgentError::new(crate::error::INVALID_ARGUMENT, format!("theme: {e}")))?;
        validate_semantic_tree_with_theme_vocab(&manifest.root, &vocab)?;
        validate_manifest_running_blocks(&manifest, &vocab)?;
        validate_manifest_node_ids(&manifest)?;
        crate::validate_agent_json(&crate::agent_json::to_agent_value(&self.package.root)?)?;
        for rb in &self.package.manifest.running_blocks {
            let v = crate::agent_json::to_agent_value(&rb.node)?;
            crate::validate_agent_json(&v)?;
        }
        Ok(())
    }

    fn set_running(
        &mut self,
        position: RunningBlockPosition,
        id: &str,
        role: &str,
        text: &str,
    ) -> Result<(), AgentError> {
        self.package
            .manifest
            .running_blocks
            .retain(|rb| rb.position != position);
        if self.find(id).is_some() {
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

    pub(crate) fn append(&mut self, node: SemanticNode) -> Result<(), AgentError> {
        self.ensure_tree_ids(&node)?;
        self.append_unchecked(node)
    }

    pub(crate) fn attach_image(
        &mut self,
        id: &str,
        bytes: &[u8],
        declared_width_mm: f64,
    ) -> Result<(), AgentError> {
        let width = mm_to_pt(declared_width_mm)?;
        let (px_w, px_h, ext) = crate::raster::raster_size(bytes)?;
        let height = scale_height(width, px_w, px_h)?;
        let src = crate::raster::asset_path(id, ext);
        self.ensure_new_id(id)?;
        if self.package.assets.contains_key(&src) {
            return Err(AgentError::new(
                DUPLICATE_ID,
                format!("image asset path '{src}' already used"),
            ));
        }
        self.package.assets.insert(src.clone(), bytes.to_vec());
        self.append_unchecked(image_node(id, &src, width, height))
    }

    fn append_unchecked(&mut self, node: SemanticNode) -> Result<(), AgentError> {
        let parent_id = self
            .sections
            .last()
            .cloned()
            .unwrap_or_else(|| ROOT_ID.to_string());
        let parent = find_node_mut(&mut self.package.root, &parent_id).ok_or_else(|| {
            AgentError::new(UNKNOWN_ID, format!("open section '{parent_id}' is gone"))
        })?;
        match &mut parent.content {
            NodeContent::Container { children } => children.push(node),
            _ => {
                return Err(AgentError::new(
                    WRONG_CONTENT,
                    format!("section '{parent_id}' is not a container"),
                ))
            }
        }
        Ok(())
    }

    fn ensure_tree_ids(&self, node: &SemanticNode) -> Result<(), AgentError> {
        let mut err = Ok(());
        k2f_core::for_each_node(node, &mut |n| {
            if err.is_err() {
                return;
            }
            err = self.ensure_new_id(&n.id);
        });
        err
    }

    fn ensure_new_id(&self, id: &str) -> Result<(), AgentError> {
        if id.is_empty() || !is_valid_node_id(id) {
            return Err(AgentError::new(
                INVALID_ID,
                format!("invalid node id '{id}'"),
            ));
        }
        if self.find(id).is_some() {
            return Err(AgentError::new(
                DUPLICATE_ID,
                format!("duplicate node id '{id}'"),
            ));
        }
        Ok(())
    }

    fn find(&self, id: &str) -> Option<&SemanticNode> {
        find_in_trees(
            &self.package.root,
            &self.package.manifest.running_blocks,
            id,
        )
    }
}
