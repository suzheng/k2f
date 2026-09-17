use k2f_core::Rect;

#[derive(Clone, Debug)]
pub struct DocIR {
    #[allow(dead_code)]
    pub title: String,
    #[allow(dead_code)]
    pub page_w_pt: f64,
    #[allow(dead_code)]
    pub page_h_pt: f64,
    pub pages: Vec<PageIR>,
    pub master: Vec<PageElement>,
    pub fonts: Vec<String>,
}

#[derive(Clone, Debug)]
pub struct PageIR {
    pub elements: Vec<PageElement>,
}

#[derive(Clone, Debug)]
pub enum PageElement {
    TextBox(TextBox),
    Shape(ShapeBox),
    Picture(PictureBox),
    Table(TableBox),
    Raster(PictureBox),
}

impl PageElement {
    pub(crate) fn rect(&self) -> &Rect {
        match self {
            Self::TextBox(tb) => &tb.rect,
            Self::Shape(s) => &s.rect,
            Self::Picture(p) | Self::Raster(p) => &p.rect,
            Self::Table(t) => &t.rect,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TextAlign {
    Left,
    Center,
    Right,
    Justify,
}

impl TextAlign {
    pub fn justification(self) -> &'static str {
        match self {
            Self::Left => "LeftAlign",
            Self::Center => "CenterAlign",
            Self::Right => "RightAlign",
            Self::Justify => "LeftJustified",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ScriptPos {
    Baseline,
    Sub,
    Super,
}

#[derive(Clone, Debug)]
pub struct TextRun {
    pub text: String,
    pub font_name: String,
    pub size_pt: f64,
    pub bold: bool,
    pub italic: bool,
    pub underline: bool,
    pub strike: bool,
    pub color_hex: String,
    pub hyperlink: Option<String>,
    pub script: ScriptPos,
    pub leading_pt: Option<f64>,
    pub auto_page_number: bool,
    /// InDesign Tracking in 1/1000 em. 0 means omit (do not fake Tracking="0").
    pub tracking: i32,
}

#[derive(Clone, Debug)]
pub struct TextBox {
    pub node_id: String,
    pub rect: Rect,
    pub runs: Vec<TextRun>,
    pub align: TextAlign,
    pub inset_top: f64,
    pub inset_left: f64,
    pub inset_bottom: f64,
    pub inset_right: f64,
    /// Extra first-line indent (pt) beyond `inset_left`. 0 = omit.
    pub first_line_indent_pt: f64,
    pub vert_center: bool,
    /// Grow the frame horizontally so host metrics do not wrap a lock one-liner.
    pub autosize_width: bool,
    /// IDML AutoSizingReferencePoint when `autosize_width`.
    pub autosize_refer: &'static str,
    /// `UseNoLineBreaksForAutoSizing` — single-line lock frames only.
    pub autosize_no_wrap: bool,
    /// Grow height so extra host wrap is not clipped (overset).
    pub autosize_height: bool,
    /// Lock-pinned lines: CharacterStyleRange NoBreak (PPTX wrap=none analog).
    pub no_break: bool,
    /// Author `\n` in `node_text` (seal stacks, pre-broken titles). Must not be
    /// collapsed by the lock-wrap reflow path.
    pub semantic_newlines: bool,
    /// Glyph line count from lock geometry (`body_lines`). Used when `\n` is
    /// absent but lock paint wrapped, or for wide multi-line body like a one-liner beside a stacked heading.
    pub lock_line_count: usize,
    /// Lock-pinned lines already encode baseline y in geometry; `Ascent` adds
    /// extra top slack and oversets the frame bottom in InDesign.
    pub first_baseline_leading_offset: bool,
    /// Any lock line fills the frame edge-to-edge (wrapped column body).
    pub full_width_lock_line: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LineDash {
    Solid,
    Dash,
    Dot,
}

#[derive(Clone, Debug)]
pub struct ShapeBox {
    pub node_id: String,
    pub rect: Rect,
    pub fill_hex: Option<String>,
    pub fill_alpha: u8,
    pub corner_pt: f64,
    pub line_hex: Option<String>,
    pub line_w_pt: f64,
    pub line_dash: LineDash,
}

#[derive(Clone, Debug)]
pub struct PictureBox {
    pub node_id: String,
    pub rect: Rect,
    pub raster_name: String,
    pub bytes: Vec<u8>,
    pub ext: String,
}

#[derive(Clone, Debug)]
pub struct TableBox {
    pub node_id: String,
    pub rect: Rect,
    pub header_rows: usize,
    pub col_widths_pt: Vec<f64>,
    pub rows: Vec<TableRow>,
}

#[derive(Clone, Debug)]
pub struct TableRow {
    pub height_pt: f64,
    pub cells: Vec<TableCell>,
}

#[derive(Clone, Debug)]
pub struct TableCell {
    #[allow(dead_code)]
    pub node_id: String,
    pub runs: Vec<TextRun>,
    pub align: TextAlign,
    pub fill_hex: Option<String>,
    pub borders: CellBorders,
    pub vert_center: bool,
    pub inset_top: f64,
    pub inset_left: f64,
    pub inset_bottom: f64,
    pub inset_right: f64,
}

#[derive(Clone, Debug, Default)]
pub struct CellBorders {
    pub top: Option<BorderStroke>,
    pub left: Option<BorderStroke>,
    pub bottom: Option<BorderStroke>,
    pub right: Option<BorderStroke>,
}

#[derive(Clone, Debug)]
pub struct BorderStroke {
    pub color_hex: String,
    pub weight_pt: f64,
    pub dash: LineDash,
}

impl DocIR {
    pub fn collect_color_hexes(&self) -> std::collections::BTreeSet<String> {
        let mut out = std::collections::BTreeSet::new();
        for page in &self.pages {
            collect_box_colors(&mut out, &page.elements);
        }
        collect_box_colors(&mut out, &self.master);
        out
    }
}

fn collect_box_colors(out: &mut std::collections::BTreeSet<String>, els: &[PageElement]) {
    for el in els {
        match el {
            PageElement::TextBox(tb) => collect_run_colors(out, &tb.runs),
            PageElement::Shape(s) => {
                if let Some(hex) = &s.fill_hex {
                    out.insert(hex.clone());
                }
                if let Some(hex) = &s.line_hex {
                    out.insert(hex.clone());
                }
            }
            PageElement::Table(t) => {
                for row in &t.rows {
                    for cell in &row.cells {
                        collect_run_colors(out, &cell.runs);
                        if let Some(hex) = &cell.fill_hex {
                            out.insert(hex.clone());
                        }
                        collect_stroke_color(out, &cell.borders.top);
                        collect_stroke_color(out, &cell.borders.left);
                        collect_stroke_color(out, &cell.borders.bottom);
                        collect_stroke_color(out, &cell.borders.right);
                    }
                }
            }
            PageElement::Picture(_) | PageElement::Raster(_) => {}
        }
    }
}

fn collect_run_colors(out: &mut std::collections::BTreeSet<String>, runs: &[TextRun]) {
    for run in runs {
        if !run.color_hex.is_empty() {
            out.insert(run.color_hex.clone());
        }
    }
}

fn collect_stroke_color(
    out: &mut std::collections::BTreeSet<String>,
    stroke: &Option<BorderStroke>,
) {
    if let Some(s) = stroke {
        out.insert(s.color_hex.clone());
    }
}
