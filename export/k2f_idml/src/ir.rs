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
    /// Unique `(family, InDesign FontStyle)` pairs used in the package.
    pub fonts: Vec<(String, String)>,
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
    /// TTF subfamily of the embedded face (`Book`, `Regular`, `Light`, …).
    pub face_style: String,
}

/// K2F paint shears synthetic italic by 0.2126 (atan ≈ 12°).
const SYNTHETIC_ITALIC_SKEW_DEG: f64 = 12.0;
/// K2F paint strokes synthetic bold at 1/30 em (center path). InDesign's
/// character stroke at that width reads heavier (outer silhouette). 1/45 em
/// with only StrokeWeight + fill StrokeColor matches the templates; 1/18 em,
/// FontStyle="Bold", StrokeType, and StrokeAlignment all went extra-black or
/// dropped the outline.
const SYNTHETIC_BOLD_EM: f64 = 45.0;

impl TextRun {
    /// InDesign `FontStyle` / `FontStyleName` for the embedded face.
    ///
    /// Paint `bold` on Regular is synthetic. Advertising `Bold` makes InDesign
    /// substitute a system Bold cut when one is installed (too heavy) or fall
    /// back to Regular when it is not. Keep this on the TTF that shipped in the
    /// package; faux bold is a same-color character stroke.
    pub fn idml_font_style(&self) -> String {
        let s = self.face_style.trim();
        if s.is_empty() {
            "Regular".into()
        } else {
            s.to_string()
        }
    }

    /// Faux italic when paint asked for italic but the face is not italic.
    pub fn synthetic_skew_deg(&self) -> Option<f64> {
        if self.italic && !k2f_paint::face_style_is_italic(&self.face_style) {
            Some(SYNTHETIC_ITALIC_SKEW_DEG)
        } else {
            None
        }
    }

    /// Stroke width in pt when paint asked for bold on a non-bold face.
    ///
    /// ~2/3 of K2F paint's 1/30 em so InDesign's outer character stroke
    /// matches the centered path stroke. Stroke color must be the run fill
    /// (default is Black — extra-black). Do not emit StrokeType / alignment.
    /// Real Bold files skip this (`font_faces` packs). Do not advertise
    /// `FontStyle="Bold"` on a Regular-only package.
    pub fn synthetic_bold_stroke_pt(&self) -> Option<f64> {
        if self.bold && !k2f_paint::face_style_is_bold(&self.face_style) && self.size_pt > 0.0 {
            Some(self.size_pt / SYNTHETIC_BOLD_EM)
        } else {
            None
        }
    }
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
    /// Paragraph LeftIndent in pt. List wrap lines hang here; 0 = omit.
    pub left_indent_pt: f64,
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
    /// When true, InDesign first baseline is `Leading` from the frame top.
    /// K2F paint uses font ascent at y_offset 0, so this stays false.
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
    /// 255 = opaque. Used for glass/translucent surfaces (`#RRGGBBAA`).
    pub fill_alpha: u8,
    /// Native InDesign `Gradient` fill. `None` = solid or no fill.
    pub gradient: Option<GradientFill>,
    pub corner_pt: f64,
    pub line_hex: Option<String>,
    /// 255 = opaque. Used for glass/translucent rims (`#RRGGBBAA`).
    pub line_alpha: u8,
    pub line_w_pt: f64,
    pub line_dash: LineDash,
}

#[derive(Clone, Debug)]
pub struct GradientFill {
    pub angle_degrees: i64,
    pub stops: Vec<GradientStopFill>,
}

#[derive(Clone, Debug)]
pub struct GradientStopFill {
    /// 0..=1000, same as the lock.
    pub pos: i64,
    pub hex: String,
}

impl ShapeBox {
    /// Graphic.xml `Gradient/…` Self for this shape.
    pub fn gradient_swatch_name(&self) -> Option<String> {
        self.gradient.as_ref()?;
        Some(gradient_swatch_name(&self.node_id))
    }
}

pub(crate) fn gradient_swatch_name(node_id: &str) -> String {
    let slug: String = node_id
        .chars()
        .map(|c| if c.is_ascii_alphanumeric() { c } else { '_' })
        .collect();
    format!("k2f_g_{slug}")
}

#[derive(Clone, Debug)]
pub struct PictureBox {
    pub node_id: String,
    pub rect: Rect,
    pub raster_name: String,
    pub bytes: Vec<u8>,
    pub ext: String,
    pub corner_pt: f64,
    /// Fill the frame proportionally (cover). False = letterboxed dest already.
    pub fill_proportionally: bool,
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
    /// Same-row occupancy; 1 means one InDesign column.
    pub colspan: u32,
    /// Visual start column (0-based); also the Cell `Name` column index.
    pub start_col: usize,
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

    pub fn collect_gradients(&self) -> Vec<(String, GradientFill)> {
        let mut out = Vec::new();
        for page in &self.pages {
            collect_box_gradients(&mut out, &page.elements);
        }
        collect_box_gradients(&mut out, &self.master);
        out
    }
}

fn collect_box_gradients(out: &mut Vec<(String, GradientFill)>, els: &[PageElement]) {
    for el in els {
        let PageElement::Shape(s) = el else {
            continue;
        };
        let Some(g) = &s.gradient else {
            continue;
        };
        let Some(name) = s.gradient_swatch_name() else {
            continue;
        };
        out.push((name, g.clone()));
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
                if let Some(g) = &s.gradient {
                    for stop in &g.stops {
                        out.insert(stop.hex.clone());
                    }
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
