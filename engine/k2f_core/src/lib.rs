use serde::{Deserialize, Serialize};
use std::fmt;
use std::ops::{Add, AddAssign, Div, Mul, Sub, SubAssign};
use thiserror::Error;

mod appearance_hash;
mod break_policy;
mod canonical_json;
mod columns;
mod compositing;
mod content_hash;
pub mod effects;
mod engine_identity;
mod hit_test;
mod layout_hints;
mod nfc;
mod node_edit;
mod node_ids;
mod node_walk;
mod paint_plan;
mod paint_types;
mod role_variant_validation;
mod running_blocks;
mod running_blocks_validation;
mod search;
mod selection;
mod semantic_code_blocks;
mod semantic_lists;
mod semantic_math;
mod sha256_hex;
mod table_assets;
mod theme_vocab;
pub mod visual_primitives;

#[cfg(test)]
mod semantic_code_blocks_tests;
#[cfg(test)]
mod semantic_math_tests;

pub use appearance_hash::{hash_appearance_binding, AppearanceHashInput};
pub use break_policy::{BreakBefore, BreakInside};
pub use canonical_json::{canonical_json_string, canonicalize_json_value};
pub use columns::ColumnSpan;
pub use compositing::*;
pub use content_hash::{hash_content, hash_manifest_semantic};
pub use engine_identity::{engine_commit_sha, engine_version};
pub use hit_test::{
    box_contains, clusters_usable, collect_boxes, hit_char_range, hit_geometry, hit_layout,
    hit_page, semantic_ids, with_text_range, Hit,
};
pub use layout_hints::*;
pub use nfc::{nfc, normalize_manifest_nfc};
pub use node_edit::{
    apply_role, collect_ids_under, find_in_trees, find_in_trees_mut, insert_child, node_text,
    remove_node, replace_node_text, tree_contains_id, NodeEditError,
};
pub use node_ids::{
    collect_sorted_node_ids, is_valid_node_id, validate_manifest_node_ids, NODE_ID_PATTERN,
};
pub use node_walk::{find_node, find_node_mut, for_each_node, for_each_node_mut};
pub use paint_plan::*;
pub use paint_types::*;
pub use role_variant_validation::validate_semantic_tree_with_theme_vocab;
pub use running_blocks::*;
pub use running_blocks_validation::validate_manifest_running_blocks;
pub use search::{search_tree, search_trees};
pub use selection::{clipboard_of, selection_of, selection_with_ids, Clipboard, Selection};
pub use semantic_code_blocks::CodeBlockValue;
pub use semantic_lists::ListMarkerType;
pub use semantic_math::ROLE_MATH;
pub use sha256_hex::sha256_hex;
pub use table_assets::{
    collapse_tables_to_assets, expand_manifest_tables_with_assets, semantic_tree_needs_assets,
    table_asset_sources, AssetsMap,
};
pub use theme_vocab::ThemeVocab;
pub use visual_primitives::*;

/// Fixed-point coordinate type (1/1000th of a point).
/// Wraps an i128 to prevent overflow and ensure determinism.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default)]
pub struct Pt(pub i128);

impl Pt {
    pub const ZERO: Pt = Pt(0);

    pub fn new(val: i128) -> Self {
        Pt(val)
    }

    /// Lossy conversion from float, useful for debugging or initial inputs
    pub fn from_f64_pt(val: f64) -> Self {
        Pt((val * 1000.0) as i128)
    }

    pub fn as_f64_pt(&self) -> f64 {
        self.0 as f64 / 1000.0
    }
}

// JSON numbers are effectively i64/u64. Keep `Pt` internal arithmetic as i128, but
// serialize/deserialize using i64 for compatibility with serde_json.
impl Serialize for Pt {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let v: i64 = self
            .0
            .try_into()
            .map_err(|_| serde::ser::Error::custom("Pt value out of i64 range"))?;
        serializer.serialize_i64(v)
    }
}

impl<'de> Deserialize<'de> for Pt {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let v = i64::deserialize(deserializer)?;
        Ok(Pt(v as i128))
    }
}

impl fmt::Display for Pt {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}pt", self.as_f64_pt())
    }
}

impl Add for Pt {
    type Output = Pt;
    fn add(self, rhs: Pt) -> Pt {
        Pt(self.0 + rhs.0)
    }
}

impl Sub for Pt {
    type Output = Pt;
    fn sub(self, rhs: Pt) -> Pt {
        Pt(self.0 - rhs.0)
    }
}

// Multiplication by scalar
impl Mul<i128> for Pt {
    type Output = Pt;
    fn mul(self, rhs: i128) -> Pt {
        Pt(self.0 * rhs)
    }
}

// Division by scalar
impl Div<i128> for Pt {
    type Output = Pt;
    fn div(self, rhs: i128) -> Pt {
        Pt(self.0 / rhs)
    }
}

impl AddAssign for Pt {
    fn add_assign(&mut self, rhs: Pt) {
        self.0 += rhs.0;
    }
}

impl SubAssign for Pt {
    fn sub_assign(&mut self, rhs: Pt) {
        self.0 -= rhs.0;
    }
}

/// Represents a semantic node in the K2F document tree (State A).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SemanticNode {
    /// Hierarchical ID, e.g., "financials.q3.table"
    pub id: String,
    /// Semantic role, e.g., "critical_warning"
    pub role: String,
    /// Optional visual variant name for this role.
    ///
    /// The allowed variants are defined by the theme (role-driven).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub variant: Option<String>,
    /// Optional whitespace preservation hint.
    ///
    /// When role is "code_block", engine validation requires this to be true (or omitted).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub preserve_whitespace: Option<bool>,
    /// Optional list group identifier (required by validation when role == "list_item").
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub list_id: Option<String>,
    /// Optional nesting depth (defaults to 0 when omitted).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub depth: Option<u32>,
    /// Optional marker style ("bullet" | "number").
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub marker_type: Option<ListMarkerType>,
    /// Content of the node
    pub content: NodeContent,
    /// Semantic modifiers (max 50)
    #[serde(default)]
    pub modifiers: Vec<Modifier>,
    /// Optional deterministic layout hint for containers (stack/grid). If absent, engine uses defaults.
    #[serde(default)]
    pub layout: Option<LayoutHint>,
    /// Page-split policy. Agents choose an enum; they cannot invent break math.
    #[serde(default, skip_serializing_if = "BreakInside::is_auto")]
    pub break_inside: BreakInside,
    /// If true, this node and the next sibling must start on the same page when possible.
    #[serde(default, skip_serializing_if = "break_policy::is_false")]
    pub keep_with_next: bool,
    /// When `page`, start this node on a new page when not already at the content top.
    #[serde(default, skip_serializing_if = "BreakBefore::is_auto")]
    pub break_before: BreakBefore,
    /// When `all`, this node spans the full width of an enclosing columns container.
    #[serde(default, skip_serializing_if = "ColumnSpan::is_none")]
    pub column_span: ColumnSpan,
}

impl Default for SemanticNode {
    fn default() -> Self {
        Self {
            id: String::new(),
            role: "body".to_string(),
            variant: None,
            preserve_whitespace: None,
            list_id: None,
            depth: None,
            marker_type: None,
            content: NodeContent::Text(String::new()),
            modifiers: vec![],
            layout: None,
            break_inside: BreakInside::Auto,
            keep_with_next: false,
            break_before: BreakBefore::Auto,
            column_span: ColumnSpan::None,
        }
    }
}

/// Deterministic layout hint for container nodes.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum LayoutHint {
    Stack {
        #[serde(default)]
        direction: StackDirection,
        /// Gap between children in fixed-point units (1/1000th of a point).
        ///
        /// NOTE: This is stored as an integer (not `Pt`) to avoid deserialization paths
        /// that go through `serde_json::Value` (internally tagged enums), where `i128`
        /// is not supported. Layout computations convert this to `Pt` at the boundary.
        #[serde(default)]
        gap: i64,
        /// Cross-axis alignment for stack children.
        #[serde(default)]
        align_items: Align,
        /// Main-axis distribution for stack children.
        #[serde(default)]
        justify_content: JustifyContent,
        /// Optional fixed outer size for the container.
        #[serde(flatten, default)]
        size: FixedSizeHint,
    },
    Grid {
        columns: Vec<GridTrack>,
        rows: Vec<GridTrack>,
        /// Gap between tracks in fixed-point units (1/1000th of a point).
        #[serde(default)]
        gap: i64,
        /// Optional row gap override. Falls back to `gap` when omitted.
        #[serde(default)]
        row_gap: Option<i64>,
        /// Optional column gap override. Falls back to `gap` when omitted.
        #[serde(default)]
        column_gap: Option<i64>,
        /// Default per-cell alignment for grid children.
        #[serde(default)]
        cell_align: Option<CellAlign>,
        /// Optional fixed outer size (needed for `fr` tracks in unbounded flow).
        #[serde(flatten, default)]
        size: FixedSizeHint,
    },
    /// Children are visually stacked in source order (first = back, last = front).
    ///
    /// This exists to model deterministic layering without numeric z-index semantics.
    Overlay {
        /// Optional fixed outer size for the container.
        #[serde(flatten, default)]
        size: FixedSizeHint,
    },
    /// Continuous multi-column flow: children pack left-to-right across equal-width columns.
    Columns {
        /// Number of columns (engine validates 2..=4).
        count: u32,
        /// Gap between columns in fixed-point Pt (1/1000 pt units).
        #[serde(default)]
        gap: i64,
    },
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum StackDirection {
    Vertical,
    Horizontal,
}

impl Default for StackDirection {
    fn default() -> Self {
        StackDirection::Vertical
    }
}

/// Explicit deterministic track sizes (`pt`, `fr`, or content-sized `auto`).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(untagged)]
pub enum GridTrack {
    /// Fixed track size in Pt (1/1000 pt units)
    Pt { pt: i64 },
    /// Fractional track size (fr units). Distributed from remaining space.
    Fr { fr: i64 },
    /// Content-sized track. `{ "auto": true }` only. Measure cells, then give leftover to `fr`.
    Auto { auto: bool },
}

impl GridTrack {
    pub fn is_auto(&self) -> bool {
        matches!(self, GridTrack::Auto { auto: true })
    }
}

/// Strict table payload for `content.type = "table"` (State A).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TableSpec {
    /// Explicit deterministic column tracks (Pt/Fr). Must be non-empty.
    pub column_widths: Vec<GridTrack>,
    /// Number of initial rows to repeat at page breaks in paged mode.
    #[serde(default)]
    pub header_rows: usize,
    /// Fixed-point Pt gap between columns/rows (1/1000 pt units).
    ///
    /// Stored as i64 for serde_json compatibility (same rationale as LayoutHint::Stack.gap).
    #[serde(default)]
    pub gap: i64,
    /// Table data source (v1: inline rows only).
    pub data: TableDataSource,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum TableDataSource {
    /// Inline rows (AI-friendly): each cell is a normal SemanticNode.
    Inline { rows: Vec<Vec<SemanticNode>> },
    /// Asset-backed rows: callers must provide an explicit assets map at compile time.
    ///
    /// The `source` string is a logical key into the assets map (e.g. `"assets/data/table.json"`).
    Asset { source: String },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type", content = "value", rename_all = "snake_case")]
pub enum NodeContent {
    Text(String),
    CodeBlock(CodeBlockValue),
    /// TeX-subset math source (NFC-normalized). Display formulas use role `"math"`.
    Math(String),
    /// Deterministic image sizing (in Pt). The engine does not inspect binary image assets.
    /// Callers must provide a stable size hint to eliminate placeholders.
    Image {
        src: String,
        width: Pt,
        height: Pt,
    },
    Container {
        children: Vec<SemanticNode>,
    },
    /// Native strict table node (v1: inline rows only).
    Table(TableSpec),
    /// Deterministic table sizing (in Pt). The engine does not load external table datasets at layout time.
    /// Callers must provide a stable size hint to eliminate placeholders.
    TableReference {
        source: String,
        view_mode: String,
        width: Pt,
        height: Pt,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Modifier {
    pub range: [usize; 2],
    #[serde(rename = "type")]
    pub mod_type: String,
    pub intent: String,
}

/// `link` modifier `intent` is a URL (spec). Theme variant keys such as
/// `default` are style lookups, not hrefs, and must not become Office targets.
pub fn hyperlink_href(intent: &str) -> Option<&str> {
    let t = intent.trim();
    if t.is_empty() {
        return None;
    }
    let lower = t.to_ascii_lowercase();
    if lower == "default" {
        return None;
    }
    if lower.contains("://")
        || lower.starts_with("mailto:")
        || lower.starts_with("tel:")
        || lower.starts_with("www.")
        || t.starts_with('#')
        || t.starts_with('/')
    {
        Some(t)
    } else {
        None
    }
}

/// Solid fill rectangle in page coordinates (millipt). Used for math fraction/radical rules.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct FillRect {
    pub x: Pt,
    pub y: Pt,
    pub width: Pt,
    pub height: Pt,
}

/// Represents a geometry node in the K2F layout tree (State C).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct GeometryNode {
    pub id: String,
    pub x: Pt,
    pub y: Pt,
    pub width: Pt,
    pub height: Pt,
    #[serde(default)]
    pub glyphs: Vec<GlyphPosition>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub text_runs: Vec<TextGlyphRun>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub fill_rects: Vec<FillRect>,
    #[serde(default)]
    pub children: Vec<GeometryNode>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct GlyphPosition {
    pub glyph_id: u32,
    /// UTF-8 **character** index into the node's source string (not a byte offset).
    /// rustybuzz reports byte clusters; layout converts at shape time so this matches
    /// `Selection.char_range`. `CLUSTER_NOT_SOURCE` marks decorative glyphs (list markers).
    /// Old locks without the field deserialize as `0`; all-zero clusters mean "unknown".
    #[serde(default)]
    pub cluster: u32,
    pub x_offset: Pt,
    pub y_offset: Pt,
    pub x_advance: Pt,
    pub y_advance: Pt,
}

impl GlyphPosition {
    /// Glyph is not from the node's source string (e.g. a list marker).
    pub const CLUSTER_NOT_SOURCE: u32 = u32::MAX;
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum CanvasMode {
    Paged,
    Infinite,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PageConfig {
    pub width: Pt,
    pub height: Pt,
    // Top, Right, Bottom, Left
    pub margin: [Pt; 4],
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Page {
    pub index: usize,
    pub width: Pt,
    pub height: Pt,
    pub root: GeometryNode,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct LayoutResult {
    pub pages: Vec<Page>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Manifest {
    pub title: String,
    pub canvas_mode: CanvasMode,
    pub page_config: PageConfig,
    pub root: SemanticNode,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub running_blocks: Vec<RunningBlockNode>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pt_math() {
        let a = Pt(1000); // 1.0pt
        let b = Pt(2000); // 2.0pt
        assert_eq!(a + b, Pt(3000));
        assert_eq!(b - a, Pt(1000));
        assert_eq!(a * 2, Pt(2000));
        assert_eq!(b / 2, Pt(1000));
    }

    #[test]
    fn test_pt_serialization() {
        let pt = Pt(1500);
        let json = serde_json::to_string(&pt).unwrap();
        assert_eq!(json, "1500");
    }

    #[test]
    fn link_intent_default_is_not_an_href() {
        assert_eq!(hyperlink_href("default"), None);
        assert_eq!(hyperlink_href(""), None);
        assert_eq!(hyperlink_href("bold"), None);
        assert_eq!(
            hyperlink_href("https://example.com"),
            Some("https://example.com")
        );
        assert_eq!(hyperlink_href("mailto:a@b.c"), Some("mailto:a@b.c"));
        assert_eq!(hyperlink_href("#section"), Some("#section"));
    }

    #[test]
    fn test_semantic_node_determinism() {
        let node = SemanticNode {
            id: "test".to_string(),
            role: "body".to_string(),
            variant: None,
            preserve_whitespace: None,
            list_id: None,
            depth: None,
            marker_type: None,
            content: NodeContent::Text("Hello".to_string()),
            modifiers: vec![],
            layout: None,
            ..Default::default()
        };

        let json1 = serde_json::to_string(&node).unwrap();
        let json2 = serde_json::to_string(&node).unwrap();
        assert_eq!(json1, json2);
    }

    #[test]
    fn test_hash_content_determinism() {
        let node = SemanticNode {
            id: "test".to_string(),
            role: "body".to_string(),
            variant: None,
            preserve_whitespace: None,
            list_id: None,
            depth: None,
            marker_type: None,
            content: NodeContent::Text("Hello World".to_string()),
            modifiers: vec![],
            layout: None,
            ..Default::default()
        };

        let hash1 = hash_content(&node);
        let hash2 = hash_content(&node);
        assert_eq!(hash1, hash2);
        assert_eq!(hash1.len(), 64); // SHA256 hex string
    }

    #[test]
    fn test_validate_semantic_tree_rejects_non_positive_image_size() {
        let node = SemanticNode {
            id: "img".to_string(),
            role: "body".to_string(),
            variant: None,
            preserve_whitespace: None,
            list_id: None,
            depth: None,
            marker_type: None,
            content: NodeContent::Image {
                src: "a.png".to_string(),
                width: Pt(0),
                height: Pt(1000),
            },
            modifiers: vec![],
            layout: None,
            ..Default::default()
        };

        let err = validate_semantic_tree(&node).unwrap_err();
        match err {
            K2FError::InvalidImageSize { .. } => {}
            other => panic!("expected InvalidImageSize, got {:?}", other),
        }
    }

    #[test]
    fn test_validate_semantic_tree_rejects_non_positive_table_reference_size() {
        let node = SemanticNode {
            id: "tbl".to_string(),
            role: "table".to_string(),
            variant: None,
            preserve_whitespace: None,
            list_id: None,
            depth: None,
            marker_type: None,
            content: NodeContent::TableReference {
                source: "asset://data/table.csv".to_string(),
                view_mode: "full".to_string(),
                width: Pt(2000),
                height: Pt(0),
            },
            modifiers: vec![],
            layout: None,
            ..Default::default()
        };

        let err = validate_semantic_tree(&node).unwrap_err();
        match err {
            K2FError::InvalidTableReferenceSize { .. } => {}
            other => panic!("expected InvalidTableReferenceSize, got {:?}", other),
        }
    }

    #[test]
    fn test_validate_semantic_tree_rejects_negative_fixed_size() {
        let node = SemanticNode {
            id: "box".to_string(),
            role: "body".to_string(),
            variant: None,
            preserve_whitespace: None,
            list_id: None,
            depth: None,
            marker_type: None,
            content: NodeContent::Container { children: vec![] },
            modifiers: vec![],
            layout: Some(LayoutHint::Overlay {
                size: FixedSizeHint {
                    width: Some(Pt(-1)),
                    height: None,
                },
            }),
            ..Default::default()
        };

        let err = validate_semantic_tree(&node).unwrap_err();
        match err {
            K2FError::InvalidFixedSize { .. } => {}
            other => panic!("expected InvalidFixedSize, got {:?}", other),
        }
    }

    #[test]
    fn test_validate_role_vocab_rejects_unknown_role() {
        let vocab: ThemeVocab = serde_json::from_str(r#"{ "roles": { "body": {} } }"#).unwrap();
        let node = SemanticNode {
            id: "n".to_string(),
            role: "unknown".to_string(),
            variant: None,
            preserve_whitespace: None,
            list_id: None,
            depth: None,
            marker_type: None,
            content: NodeContent::Text("x".to_string()),
            modifiers: vec![],
            layout: None,
            ..Default::default()
        };

        let err = validate_semantic_tree_with_theme_vocab(&node, &vocab).unwrap_err();
        match err {
            K2FError::UnknownRole { .. } => {}
            other => panic!("expected UnknownRole, got {:?}", other),
        }
    }

    #[test]
    fn test_validate_role_vocab_rejects_unknown_variant() {
        let vocab: ThemeVocab =
            serde_json::from_str(r#"{ "roles": { "body": { "variants": { "glass": {} } } } }"#)
                .unwrap();
        let node = SemanticNode {
            id: "n".to_string(),
            role: "body".to_string(),
            variant: Some("not_defined".to_string()),
            preserve_whitespace: None,
            list_id: None,
            depth: None,
            marker_type: None,
            content: NodeContent::Text("x".to_string()),
            modifiers: vec![],
            layout: None,
            ..Default::default()
        };

        let err = validate_semantic_tree_with_theme_vocab(&node, &vocab).unwrap_err();
        match err {
            K2FError::UnknownVariant { .. } => {}
            other => panic!("expected UnknownVariant, got {:?}", other),
        }
    }

    #[test]
    fn test_validate_role_vocab_allows_defined_variant() {
        let vocab: ThemeVocab =
            serde_json::from_str(r#"{ "roles": { "body": { "variants": { "glass": {} } } } }"#)
                .unwrap();
        let node = SemanticNode {
            id: "n".to_string(),
            role: "body".to_string(),
            variant: Some("glass".to_string()),
            preserve_whitespace: None,
            list_id: None,
            depth: None,
            marker_type: None,
            content: NodeContent::Text("x".to_string()),
            modifiers: vec![],
            layout: None,
            ..Default::default()
        };

        validate_semantic_tree_with_theme_vocab(&node, &vocab).unwrap();
    }

    fn make_cell_text(id: &str) -> SemanticNode {
        SemanticNode {
            id: id.to_string(),
            role: "body".to_string(),
            variant: None,
            preserve_whitespace: None,
            list_id: None,
            depth: None,
            marker_type: None,
            content: NodeContent::Text("x".to_string()),
            modifiers: vec![],
            layout: None,
            ..Default::default()
        }
    }

    #[test]
    fn test_validate_semantic_tree_rejects_table_with_empty_column_widths() {
        let node = SemanticNode {
            id: "t".to_string(),
            role: "table".to_string(),
            variant: None,
            preserve_whitespace: None,
            list_id: None,
            depth: None,
            marker_type: None,
            content: NodeContent::Table(TableSpec {
                column_widths: vec![],
                header_rows: 0,
                gap: 0,
                data: TableDataSource::Inline { rows: vec![] },
            }),
            modifiers: vec![],
            layout: None,
            ..Default::default()
        };

        let err = validate_semantic_tree(&node).unwrap_err();
        match err {
            K2FError::TableRequiresColumns { .. } => {}
            other => panic!("expected TableRequiresColumns, got {:?}", other),
        }
    }

    #[test]
    fn grid_track_auto_roundtrips_json() {
        let t: GridTrack = serde_json::from_str(r#"{"auto":true}"#).unwrap();
        assert!(t.is_auto());
        let back = serde_json::to_string(&t).unwrap();
        assert!(back.contains("auto"));
        let pt: GridTrack = serde_json::from_str(r#"{"pt":1000}"#).unwrap();
        assert_eq!(pt, GridTrack::Pt { pt: 1000 });
    }

    #[test]
    fn test_validate_semantic_tree_rejects_table_auto_column() {
        let node = SemanticNode {
            id: "t".to_string(),
            role: "table".to_string(),
            variant: None,
            preserve_whitespace: None,
            list_id: None,
            depth: None,
            marker_type: None,
            content: NodeContent::Table(TableSpec {
                column_widths: vec![GridTrack::Auto { auto: true }],
                header_rows: 0,
                gap: 0,
                data: TableDataSource::Inline {
                    rows: vec![vec![make_cell_text("c0")]],
                },
            }),
            modifiers: vec![],
            layout: None,
            ..Default::default()
        };
        let err = validate_semantic_tree(&node).unwrap_err();
        match err {
            K2FError::TableAutoTrack { .. } => {}
            other => panic!("expected TableAutoTrack, got {other:?}"),
        }
    }

    #[test]
    fn test_validate_semantic_tree_rejects_table_row_len_mismatch() {
        let node = SemanticNode {
            id: "t".to_string(),
            role: "table".to_string(),
            variant: None,
            preserve_whitespace: None,
            list_id: None,
            depth: None,
            marker_type: None,
            content: NodeContent::Table(TableSpec {
                column_widths: vec![GridTrack::Pt { pt: 1000 }, GridTrack::Pt { pt: 1000 }],
                header_rows: 0,
                gap: 0,
                data: TableDataSource::Inline {
                    rows: vec![vec![make_cell_text("c0")]], // should be 2 cells
                },
            }),
            modifiers: vec![],
            layout: None,
            ..Default::default()
        };

        let err = validate_semantic_tree(&node).unwrap_err();
        match err {
            K2FError::TableRowLengthMismatch { .. } => {}
            other => panic!("expected TableRowLengthMismatch, got {:?}", other),
        }
    }

    #[test]
    fn test_validate_semantic_tree_rejects_table_header_rows_too_large() {
        let node = SemanticNode {
            id: "t".to_string(),
            role: "table".to_string(),
            variant: None,
            preserve_whitespace: None,
            list_id: None,
            depth: None,
            marker_type: None,
            content: NodeContent::Table(TableSpec {
                column_widths: vec![GridTrack::Pt { pt: 1000 }],
                header_rows: 1,
                gap: 0,
                data: TableDataSource::Inline { rows: vec![] },
            }),
            modifiers: vec![],
            layout: None,
            ..Default::default()
        };

        let err = validate_semantic_tree(&node).unwrap_err();
        match err {
            K2FError::TableHeaderRowsTooLarge { .. } => {}
            other => panic!("expected TableHeaderRowsTooLarge, got {:?}", other),
        }
    }

    #[test]
    fn test_validate_semantic_tree_recurses_into_table_cells() {
        let bad_cell = SemanticNode {
            id: "bad_img".to_string(),
            role: "body".to_string(),
            variant: None,
            preserve_whitespace: None,
            list_id: None,
            depth: None,
            marker_type: None,
            content: NodeContent::Image {
                src: "a.png".to_string(),
                width: Pt(1000),
                height: Pt(0), // invalid
            },
            modifiers: vec![],
            layout: None,
            ..Default::default()
        };

        let node = SemanticNode {
            id: "t".to_string(),
            role: "table".to_string(),
            variant: None,
            preserve_whitespace: None,
            list_id: None,
            depth: None,
            marker_type: None,
            content: NodeContent::Table(TableSpec {
                column_widths: vec![GridTrack::Pt { pt: 1000 }],
                header_rows: 0,
                gap: 0,
                data: TableDataSource::Inline {
                    rows: vec![vec![bad_cell]],
                },
            }),
            modifiers: vec![],
            layout: None,
            ..Default::default()
        };

        let err = validate_semantic_tree(&node).unwrap_err();
        match err {
            K2FError::InvalidImageSize { .. } => {}
            other => panic!("expected InvalidImageSize, got {:?}", other),
        }
    }

    #[test]
    fn test_validate_semantic_tree_rejects_list_item_missing_list_id() {
        let node = SemanticNode {
            id: "li".to_string(),
            role: semantic_lists::ROLE_LIST_ITEM.to_string(),
            variant: None,
            preserve_whitespace: None,
            list_id: None,
            depth: None,
            marker_type: None,
            content: NodeContent::Text("Item".to_string()),
            modifiers: vec![],
            layout: None,
            ..Default::default()
        };

        let err = validate_semantic_tree(&node).unwrap_err();
        match err {
            K2FError::ListItemMissingListId { .. } => {}
            other => panic!("expected ListItemMissingListId, got {:?}", other),
        }
    }

    #[test]
    fn test_validate_semantic_tree_rejects_list_item_non_text_content() {
        let node = SemanticNode {
            id: "li".to_string(),
            role: semantic_lists::ROLE_LIST_ITEM.to_string(),
            variant: None,
            preserve_whitespace: None,
            list_id: Some("list_a".to_string()),
            depth: Some(0),
            marker_type: None,
            content: NodeContent::Image {
                src: "a.png".to_string(),
                width: Pt(1000),
                height: Pt(1000),
            },
            modifiers: vec![],
            layout: None,
            ..Default::default()
        };

        let err = validate_semantic_tree(&node).unwrap_err();
        match err {
            K2FError::ListItemRequiresTextContent { .. } => {}
            other => panic!("expected ListItemRequiresTextContent, got {:?}", other),
        }
    }

    #[test]
    fn test_validate_semantic_tree_allows_list_item_with_default_depth() {
        let node = SemanticNode {
            id: "li".to_string(),
            role: semantic_lists::ROLE_LIST_ITEM.to_string(),
            variant: None,
            preserve_whitespace: None,
            list_id: Some("list_a".to_string()),
            depth: None,
            marker_type: Some(ListMarkerType::Bullet),
            content: NodeContent::Text("Item".to_string()),
            modifiers: vec![],
            layout: None,
            ..Default::default()
        };

        validate_semantic_tree(&node).unwrap();
    }
}

/// A "Geometry Lock" file representing the final compiled state of a document.
/// It cryptographically binds the semantic input (Content Hash) to the geometric output.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct LockFile {
    pub engine_version: String,
    pub engine_commit_sha: String,
    pub content_hash: String,
    /// Semantic hash + theme + fonts + page_config + engine identity.
    /// Official VALID for sign-off is this binding, not content_hash alone.
    pub appearance_hash: String,
    pub geometry: LayoutResult,
    #[serde(default)]
    pub render_plan: RenderPlan,
}

impl LockFile {
    pub fn has_unknown_paint_ops(&self) -> bool {
        crate::render_plan_has_unknown_ops(&self.render_plan)
    }
}

#[derive(Debug, Error)]
pub enum K2FError {
    #[error("modifier limit exceeded: {count} (max {max}) on node '{node_id}'")]
    ModifierLimitExceeded {
        node_id: String,
        count: usize,
        max: usize,
    },
    #[error("invalid modifier range {range:?} on node '{node_id}'")]
    InvalidModifierRange { node_id: String, range: [usize; 2] },
    #[error("modifier range is not on UTF-8 char boundary {range:?} on node '{node_id}'")]
    ModifierRangeNotOnCharBoundary { node_id: String, range: [usize; 2] },
    #[error("grid layout requires at least 1 column and 1 row on node '{node_id}'")]
    GridRequiresTracks { node_id: String },
    #[error("grid layout has more children ({children}) than cells ({cells}) on node '{node_id}'")]
    GridTooManyChildren {
        node_id: String,
        children: usize,
        cells: usize,
    },
    #[error("image node '{node_id}' must have positive size (width={width}, height={height})")]
    InvalidImageSize {
        node_id: String,
        width: Pt,
        height: Pt,
    },
    #[error(
        "table_reference node '{node_id}' must have positive size (width={width}, height={height})"
    )]
    InvalidTableReferenceSize {
        node_id: String,
        width: Pt,
        height: Pt,
    },
    #[error("table node '{node_id}' must have non-negative gap (gap={gap})")]
    InvalidTableGap { node_id: String, gap: i64 },
    #[error("table node '{node_id}' must have at least 1 column_widths track")]
    TableRequiresColumns { node_id: String },
    #[error("table node '{node_id}' column_widths do not support auto tracks; use pt or fr")]
    TableAutoTrack { node_id: String },
    #[error("table node '{node_id}' has header_rows={header_rows} but only {rows} rows")]
    TableHeaderRowsTooLarge {
        node_id: String,
        header_rows: usize,
        rows: usize,
    },
    #[error("table node '{node_id}' row {row} has {got} cells but expected {expected}")]
    TableRowLengthMismatch {
        node_id: String,
        row: usize,
        expected: usize,
        got: usize,
    },
    #[error("table node '{node_id}' uses asset-backed data source '{asset_source}', but assets were not provided")]
    TableAssetDataRequiresAssets {
        node_id: String,
        asset_source: String,
    },
    #[error("fixed container size must be non-negative on node '{node_id}' (width={width:?}, height={height:?})")]
    InvalidFixedSize {
        node_id: String,
        width: Option<Pt>,
        height: Option<Pt>,
    },
    #[error("unknown role '{role}' on node '{node_id}'")]
    UnknownRole { node_id: String, role: String },
    #[error("unknown variant '{variant}' for role '{role}' on node '{node_id}'")]
    UnknownVariant {
        node_id: String,
        role: String,
        variant: String,
    },
    #[error("list item node '{node_id}' is missing required list_id")]
    ListItemMissingListId { node_id: String },
    #[error("list item node '{node_id}' has an empty list_id")]
    ListItemEmptyListId { node_id: String },
    #[error("list item node '{node_id}' must have text content (got {got})")]
    ListItemRequiresTextContent { node_id: String, got: String },
    #[error("code block node '{node_id}' requires content.type = \"code_block\" (got {got})")]
    CodeBlockRoleRequiresCodeBlockContent { node_id: String, got: String },
    #[error("node '{node_id}' has content.type = \"code_block\" but role is '{role}' (expected role \"code_block\")")]
    CodeBlockContentRequiresCodeBlockRole { node_id: String, role: String },
    #[error("code block node '{node_id}' must have preserve_whitespace=true (or omitted)")]
    CodeBlockPreserveWhitespaceMustBeTrue { node_id: String },
    #[error("code block node '{node_id}' cannot specify layout (must be omitted/null)")]
    CodeBlockLayoutNotAllowed { node_id: String },
    #[error("code block node '{node_id}' has disallowed modifier type '{mod_type}' (only \"syntax_highlight\" is allowed)")]
    CodeBlockDisallowedModifierType { node_id: String, mod_type: String },
    #[error("math node '{node_id}' requires content.type = \"math\" (got {got})")]
    MathRoleRequiresMathContent { node_id: String, got: String },
    #[error("node '{node_id}' has content.type = \"math\" but role is '{role}' (expected role \"math\")")]
    MathContentRequiresMathRole { node_id: String, role: String },
    #[error("math node '{node_id}' cannot specify layout (must be omitted/null)")]
    MathLayoutNotAllowed { node_id: String },
    #[error("math node '{node_id}' cannot have modifiers in v1")]
    MathModifiersNotAllowed { node_id: String },
    #[error("math node '{node_id}' has empty TeX source")]
    MathEmpty { node_id: String },
    #[error("running blocks are only supported when canvas_mode is 'paged'")]
    RunningBlocksRequirePagedMode,
    #[error("running block node '{node_id}' uses unknown placeholder '{{{{{token}}}}}'")]
    RunningBlockInvalidPlaceholder { node_id: String, token: String },
    #[error("running block node '{node_id}' has unbalanced placeholder braces")]
    RunningBlockUnbalancedPlaceholders { node_id: String },
    #[error("node id is empty")]
    EmptyNodeId,
    #[error("invalid node id '{node_id}' (use dotted ASCII letters, digits, underscore)")]
    InvalidNodeId { node_id: String },
    #[error("duplicate node id '{node_id}'")]
    DuplicateNodeId { node_id: String },
    #[error("COLUMNS_INVALID: node '{node_id}': {reason}")]
    ColumnsInvalid { node_id: String, reason: String },
    #[error("COLUMNS_NESTED: nested columns layout on node '{node_id}'")]
    ColumnsNested { node_id: String },
    #[error("COLUMN_SPAN_OUTSIDE_COLUMNS: node '{node_id}' has column_span all outside a columns container")]
    ColumnSpanOutsideColumns { node_id: String },
}

/// Structural validation that is required by the spec:
/// - max 50 modifiers per node
/// - modifier ranges must be valid (for Text nodes)
pub fn validate_semantic_tree(root: &SemanticNode) -> Result<(), K2FError> {
    validate_node(root)?;
    columns::validate_columns_tree(root)?;
    Ok(())
}

fn validate_node(node: &SemanticNode) -> Result<(), K2FError> {
    const MAX_MODIFIERS: usize = 50;
    if node.modifiers.len() > MAX_MODIFIERS {
        return Err(K2FError::ModifierLimitExceeded {
            node_id: node.id.clone(),
            count: node.modifiers.len(),
            max: MAX_MODIFIERS,
        });
    }

    semantic_lists::validate_list_item_invariants(node)?;
    semantic_code_blocks::validate_code_block_invariants(node)?;
    semantic_math::validate_math_invariants(node)?;

    match &node.content {
        NodeContent::Text(text) => {
            validate_modifier_ranges_on_text(&node.id, text, &node.modifiers)?
        }
        NodeContent::CodeBlock(value) => {
            let canonical = value.to_canonical_text();
            validate_modifier_ranges_on_text(&node.id, canonical.as_ref(), &node.modifiers)?;
        }
        _ => {}
    }

    match &node.content {
        NodeContent::Image { width, height, .. } => {
            if width.0 <= 0 || height.0 <= 0 {
                return Err(K2FError::InvalidImageSize {
                    node_id: node.id.clone(),
                    width: *width,
                    height: *height,
                });
            }
        }
        NodeContent::Container { children } => {
            // Validate layout hints that are statically checkable
            if let Some(LayoutHint::Stack { size, .. }) = &node.layout {
                validate_fixed_size_hint(&node.id, size)?;
            }
            if let Some(LayoutHint::Overlay { size }) = &node.layout {
                validate_fixed_size_hint(&node.id, size)?;
            }
            if let Some(LayoutHint::Grid {
                columns,
                rows,
                size,
                ..
            }) = &node.layout
            {
                validate_fixed_size_hint(&node.id, size)?;
                if columns.is_empty() || rows.is_empty() {
                    return Err(K2FError::GridRequiresTracks {
                        node_id: node.id.clone(),
                    });
                }
                let cells = columns.len() * rows.len();
                if children.len() > cells {
                    return Err(K2FError::GridTooManyChildren {
                        node_id: node.id.clone(),
                        children: children.len(),
                        cells,
                    });
                }
            }
            if let Some(LayoutHint::Columns { count, gap }) = &node.layout {
                if !(2..=4).contains(count) {
                    return Err(K2FError::ColumnsInvalid {
                        node_id: node.id.clone(),
                        reason: format!("count must be 2..=4 (got {count})"),
                    });
                }
                if *gap < 0 {
                    return Err(K2FError::ColumnsInvalid {
                        node_id: node.id.clone(),
                        reason: format!("gap must be >= 0 (got {gap})"),
                    });
                }
            }

            for c in children {
                validate_node(c)?;
            }
        }
        NodeContent::Table(spec) => {
            if spec.gap < 0 {
                return Err(K2FError::InvalidTableGap {
                    node_id: node.id.clone(),
                    gap: spec.gap,
                });
            }
            if spec.column_widths.is_empty() {
                return Err(K2FError::TableRequiresColumns {
                    node_id: node.id.clone(),
                });
            }
            if spec.column_widths.iter().any(|t| matches!(t, GridTrack::Auto { .. })) {
                return Err(K2FError::TableAutoTrack {
                    node_id: node.id.clone(),
                });
            }

            let rows = match &spec.data {
                TableDataSource::Inline { rows } => rows,
                TableDataSource::Asset { source } => {
                    return Err(K2FError::TableAssetDataRequiresAssets {
                        node_id: node.id.clone(),
                        asset_source: source.clone(),
                    });
                }
            };

            if spec.header_rows > rows.len() {
                return Err(K2FError::TableHeaderRowsTooLarge {
                    node_id: node.id.clone(),
                    header_rows: spec.header_rows,
                    rows: rows.len(),
                });
            }

            let expected_cols = spec.column_widths.len();
            for (r, row) in rows.iter().enumerate() {
                if row.len() != expected_cols {
                    return Err(K2FError::TableRowLengthMismatch {
                        node_id: node.id.clone(),
                        row: r,
                        expected: expected_cols,
                        got: row.len(),
                    });
                }
                for cell in row {
                    validate_node(cell)?;
                }
            }
        }
        NodeContent::TableReference { width, height, .. } => {
            if width.0 <= 0 || height.0 <= 0 {
                return Err(K2FError::InvalidTableReferenceSize {
                    node_id: node.id.clone(),
                    width: *width,
                    height: *height,
                });
            }
        }
        _ => {}
    }

    Ok(())
}

fn validate_modifier_ranges_on_text(
    node_id: &str,
    text: &str,
    modifiers: &[Modifier],
) -> Result<(), K2FError> {
    let len = text.len();
    for m in modifiers {
        let [s, e] = m.range;
        if s >= e || e > len {
            return Err(K2FError::InvalidModifierRange {
                node_id: node_id.to_string(),
                range: m.range,
            });
        }
        if !text.is_char_boundary(s) || !text.is_char_boundary(e) {
            return Err(K2FError::ModifierRangeNotOnCharBoundary {
                node_id: node_id.to_string(),
                range: m.range,
            });
        }
    }
    Ok(())
}

fn validate_fixed_size_hint(node_id: &str, size: &FixedSizeHint) -> Result<(), K2FError> {
    let width = size.width;
    let height = size.height;
    if width.map_or(false, |w| w.0 < 0) || height.map_or(false, |h| h.0 < 0) {
        return Err(K2FError::InvalidFixedSize {
            node_id: node_id.to_string(),
            width,
            height,
        });
    }
    Ok(())
}
