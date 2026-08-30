use k2f_core::{
    clipboard_of, collect_boxes, find_in_trees, hit_layout, node_text, search_trees, selection_of,
    selection_with_ids, semantic_ids, with_text_range, Clipboard, GeometryNode, Hit, Pt, Selection,
};

use crate::document::OpenedDocument;

#[derive(Debug, Clone, serde::Serialize)]
pub struct LocatedBox {
    pub page: usize,
    pub id: String,
    pub x: i64,
    pub y: i64,
    pub width: i64,
    pub height: i64,
}

impl OpenedDocument {
    /// Hit test in millipts (1/1000 pt). Uses the published lock, never the live tree.
    pub fn hit_test(&self, page: usize, x_milli_pt: i64, y_milli_pt: i64) -> Option<Hit> {
        let lock = self.lock.as_ref()?;
        let mut hit = hit_layout(
            &lock.geometry,
            page,
            Pt(x_milli_pt as i128),
            Pt(y_milli_pt as i128),
        )?;
        let exists = |id: &str| {
            find_in_trees(&self.query_root, &self.query_running, id).is_some()
        };
        hit.ids = semantic_ids(&hit.ids, exists);
        if hit.ids.is_empty() {
            return None;
        }
        if hit.char_range.is_some() {
            return Some(hit);
        }
        let text = hit
            .leaf()
            .and_then(|id| find_in_trees(&self.query_root, &self.query_running, id))
            .and_then(node_text);
        Some(with_text_range(hit, text))
    }

    pub fn selection(&self, id: &str) -> Option<Selection> {
        let node = find_in_trees(&self.query_root, &self.query_running, id)?;
        selection_of(node)
    }

    pub fn selection_from_hit(&self, hit: &Hit) -> Option<Selection> {
        let id = hit.leaf()?;
        let node = find_in_trees(&self.query_root, &self.query_running, id)?;
        let mut sel = selection_with_ids(node, hit.ids.clone())?;
        sel.char_range = hit.char_range;
        Some(sel)
    }

    pub fn clipboard(&self, id: &str) -> Option<Clipboard> {
        let node = find_in_trees(&self.query_root, &self.query_running, id)?;
        clipboard_of(node)
    }

    /// Selection ranges → Markdown (no K2F HTML hints). Empty ranges → empty string.
    pub fn selection_to_markdown(
        &self,
        ranges: &[k2f_markdown::NodeCharRange],
    ) -> String {
        k2f_markdown::selection_to_markdown(
            &self.query_root,
            &self.query_running,
            ranges,
            k2f_markdown::MarkdownEmitOptions::clipboard(),
        )
    }

    /// Same as [`Self::selection_to_markdown`], parsing `[{node_id, char_start, char_end, ...}]`.
    pub fn selection_markdown_json(&self, ranges_json: &str) -> Result<String, String> {
        let ranges: Vec<k2f_markdown::NodeCharRange> = serde_json::from_str(ranges_json)
            .map_err(|e| format!("selection ranges: {e}"))?;
        Ok(self.selection_to_markdown(&ranges))
    }

    pub fn search(&self, query: &str) -> Vec<String> {
        search_trees(&self.query_root, &self.query_running, query)
    }

    pub fn boxes_for(&self, id: &str) -> Vec<LocatedBox> {
        let Some(lock) = &self.lock else {
            return vec![];
        };
        let mut out = Vec::new();
        for (page, p) in lock.geometry.pages.iter().enumerate() {
            let mut nodes: Vec<&GeometryNode> = Vec::new();
            collect_boxes(&p.root, id, &mut nodes);
            for n in nodes {
                out.push(LocatedBox {
                    page,
                    id: n.id.clone(),
                    x: n.x.0 as i64,
                    y: n.y.0 as i64,
                    width: n.width.0 as i64,
                    height: n.height.0 as i64,
                });
            }
        }
        out
    }
}
