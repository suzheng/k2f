mod emit;
mod fonts;
mod kind;
mod map;

use k2f_core::{find_in_trees, node_text, PaintOp};
use k2f_paint::{placed_glyphs, OpenedDocument, PlacedGlyph};
use std::collections::{BTreeMap, HashMap};
use ttf_parser::Face;

use crate::draw::PageDraw;
use crate::error::PdfError;
use crate::ids::Alloc;
use crate::source::caption_glyphs;

pub use fonts::{reserve, write, FontSet};

pub fn emit_page(page: &mut PageDraw, glyphs: &[(PlacedGlyph, String)], fonts: &FontSet) {
    emit::emit(page, glyphs, &fonts.family_to_font);
}

pub fn collect_page(
    doc: &OpenedDocument,
    page_idx: usize,
    faces: &HashMap<String, Face<'_>>,
) -> Result<Vec<(PlacedGlyph, String)>, PdfError> {
    let lock = doc.lock().ok_or(PdfError::Unlocked)?;
    let page = lock
        .geometry
        .pages
        .get(page_idx)
        .ok_or(k2f_paint::PaintError::PageOutOfRange(page_idx))?;
    let plan = lock
        .render_plan
        .pages
        .iter()
        .find(|p| p.index == page.index)
        .ok_or(k2f_paint::PaintError::MissingRenderPlan(page.index))?;
    let mut geo_by_id = HashMap::new();
    k2f_paint::index_geometry_multi_str(&page.root, &mut geo_by_id);
    let mut out = Vec::new();
    for op in &plan.ops {
        let PaintOp::DrawText {
            node_id,
            rect,
            runs,
        } = op
        else {
            continue;
        };
        let Some(geo) = k2f_paint::geo_for_op_str(&geo_by_id, node_id, rect) else {
            continue;
        };
        let Some(text) =
            find_in_trees(doc.semantic_root(), doc.running_blocks(), node_id).and_then(node_text)
        else {
            continue;
        };
        let placed = placed_glyphs(faces, geo, rect, runs)?;
        let chars: Vec<char> = text.chars().collect();
        let clusters: Vec<u32> = placed.iter().map(|g| g.cluster).collect();
        let mut last_cluster = None;
        for (i, g) in placed.iter().enumerate() {
            if last_cluster == Some(g.cluster) {
                continue;
            }
            let uni = map::unicode_for(g.cluster, map::next_source_cluster(&clusters, i), &chars);
            if uni.is_empty() {
                continue;
            }
            last_cluster = Some(g.cluster);
            out.push((g.clone(), uni));
        }
    }
    Ok(out)
}

pub fn collect_caption(
    page_h: f64,
    faces: &HashMap<String, Face<'_>>,
    hash: &str,
) -> Vec<(PlacedGlyph, String)> {
    caption_glyphs(page_h, faces, hash)
}

pub fn merge_gid_maps(
    fonts: &FontSet,
    glyphs: &[(PlacedGlyph, String)],
    maps: &mut [BTreeMap<u16, String>],
) {
    for (g, uni) in glyphs {
        if uni.is_empty() {
            continue;
        }
        let Some(i) = fonts.slot_index(&g.font_family) else {
            continue;
        };
        maps[i].entry(g.glyph_id).or_insert_with(|| uni.clone());
    }
}

impl FontSet {
    pub fn slot_index(&self, family: &str) -> Option<usize> {
        let name = self
            .family_to_font
            .get(family)
            .or_else(|| self.family_to_font.get("default"))?;
        self.slots.iter().position(|s| s.name == *name)
    }
}

pub fn alloc_fonts(alloc: &mut Alloc, fonts: &BTreeMap<String, Vec<u8>>) -> FontSet {
    reserve(alloc, fonts)
}
