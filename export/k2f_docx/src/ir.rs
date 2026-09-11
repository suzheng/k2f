use std::collections::BTreeMap;

#[derive(Clone, Debug)]
pub struct DocIR {
    pub title: String,
    #[allow(dead_code)]
    pub page_width_emu: i64,
    #[allow(dead_code)]
    pub page_height_emu: i64,
    pub page_width_twips: i64,
    pub page_height_twips: i64,
    pub pages: Vec<PageIR>,
    pub header: Vec<PageElement>,
    pub footer: Vec<PageElement>,
}

#[derive(Clone, Debug)]
pub struct PageIR {
    pub bg_hex: String,
    pub elements: Vec<PageElement>,
}

#[derive(Clone, Debug)]
pub enum PageElement {
    TextBox(TextBox),
    Shape(ShapeBox),
    Picture(PictureBox),
    #[allow(dead_code)]
    Table(TableBox),
    Raster(PictureBox),
}

#[derive(Clone, Debug)]
pub struct TextBox {
    pub node_id: String,
    pub x_emu: i64,
    pub y_emu: i64,
    pub cx_emu: i64,
    pub cy_emu: i64,
    pub runs: Vec<TextRun>,
    pub align: TextAlign,
    pub bullet: bool,
    pub numbered: bool,
    /// Legacy Word `w:numId`. Current export keeps this at 0 and paints
    /// list markers as literal runs (host auto-numbering is unreliable in
    /// floating text boxes).
    pub num_id: u32,
    pub ilvl: u32,
    pub l_ins_emu: i64,
    /// Word `w:ind` hanging for literal list markers. 0 = no hanging.
    /// Separate from [`l_ins_emu`] (bodyPr left pad / outer marker pad).
    pub hang_emu: i64,
    pub t_ins_emu: i64,
    pub r_ins_emu: i64,
    pub b_ins_emu: i64,
    pub line_twips: Option<i64>,
    /// Last pinned paragraph: face size, not inter-line delta (no trailing leading).
    pub last_line_twips: Option<i64>,
    /// When non-empty, exact line pitch (twips) per paragraph — used when folding
    /// stacked labels with different face sizes into one shell text box.
    pub para_line_twips: Vec<Option<i64>>,
    /// Extra space after each paragraph (twips); length may be shorter than paras.
    pub para_after_twips: Vec<i64>,
    pub vert_center: bool,
    pub preserve_whitespace: bool,
    pub relative_height: u32,
    /// Opaque underlay so Word Dark Mode does not invert noFill text.
    pub fill_hex: Option<String>,
    /// 255 = opaque. Translucent pill fills are copied from the matching DrawBox.
    pub fill_alpha: u8,
    /// Copied from the matching DrawBox when the node is a decorated chip/pill.
    pub corner_emu: i64,
    /// Outline copied from the matching DrawBox (outlined badges). `None` = noFill.
    pub line_hex: Option<String>,
    /// 255 = opaque. Folded from the matching DrawBox stroke.
    pub line_alpha: u8,
    pub line_w_emu: i64,
    pub line_dash: LineDash,
    /// DrawingML wrap. False for glyph-tight single-line lock boxes.
    pub wrap: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TextAlign {
    Left,
    Center,
    Right,
    Justify,
}

impl TextAlign {
    pub fn jc_val(self) -> &'static str {
        match self {
            Self::Left => "left",
            Self::Center => "center",
            Self::Right => "right",
            Self::Justify => "both",
        }
    }
}

#[derive(Clone, Debug)]
pub struct TextRun {
    pub text: String,
    pub font_name: String,
    pub sz_half_points: i32,
    pub bold: bool,
    pub italic: bool,
    pub underline: bool,
    pub strike: bool,
    pub color_hex: String,
    pub hyperlink: Option<String>,
    pub script: ScriptPos,
    pub tracking_twips: i32,
    pub field: Option<DocField>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ScriptPos {
    Baseline,
    Sub,
    Super,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DocField {
    Page,
    NumPages,
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
    pub x_emu: i64,
    pub y_emu: i64,
    pub cx_emu: i64,
    pub cy_emu: i64,
    pub fill_hex: Option<String>,
    /// 255 = opaque. Used for glass/translucent surfaces (`#RRGGBBAA`).
    pub fill_alpha: u8,
    pub gradient: Option<GradientFill>,
    pub corner_emu: i64,
    pub line_hex: Option<String>,
    /// 255 = opaque. `#RRGGBBAA` borders keep lock alpha on DrawingML `a:ln`.
    pub line_alpha: u8,
    pub line_w_emu: i64,
    pub line_dash: LineDash,
    pub behind_doc: bool,
    pub relative_height: u32,
    /// In-front opaque fills pin extent with an empty txBox. Shapes that later
    /// paint sits on must not: Writer paints that empty frame over later labels.
    pub pin_empty_txbox: bool,
}

#[derive(Clone, Debug)]
pub struct GradientFill {
    pub angle_degrees: i64,
    pub stops: Vec<GradientStopFill>,
}

#[derive(Clone, Debug)]
pub struct GradientStopFill {
    pub pos: i64,
    pub hex: String,
    pub alpha: u8,
}

#[derive(Clone, Debug)]
pub struct PictureBox {
    pub node_id: String,
    pub x_emu: i64,
    pub y_emu: i64,
    pub cx_emu: i64,
    pub cy_emu: i64,
    pub media_name: String,
    pub bytes: Vec<u8>,
    pub relative_height: u32,
    /// Effect slices pin extent with an empty txBox when nothing later overlaps.
    /// Slices (and lock images) under later paint must not: Writer paints that
    /// empty frame over later labels.
    pub pin_empty_txbox: bool,
}

#[derive(Clone, Debug)]
pub struct TableBox {
    pub node_id: String,
    pub x_emu: i64,
    pub y_emu: i64,
    pub cx_emu: i64,
    pub cy_emu: i64,
    pub width_twips: i64,
    pub col_widths_twips: Vec<i64>,
    pub rows: Vec<TableRow>,
    pub relative_height: u32,
    /// Opaque wrapper fill so Word does not paint `wps:style` fillRef black
    /// through `a:noFill` on the table text box.
    pub fill_hex: Option<String>,
}

#[derive(Clone, Debug)]
pub struct TableRow {
    pub height_twips: i64,
    pub cells: Vec<TableCell>,
}

#[derive(Clone, Debug)]
pub struct TableCell {
    pub node_id: String,
    pub width_twips: i64,
    pub runs: Vec<TextRun>,
    pub align: TextAlign,
    pub fill_hex: Option<String>,
    pub preserve_whitespace: bool,
    pub borders: CellBorders,
    /// Lock glyphs vertically centered in the cell box (`w:vAlign`).
    pub vert_center: bool,
    pub line_twips: Option<i64>,
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
    pub sz: i64,
    pub val: &'static str,
}

impl PageElement {
    pub fn textbox(&self) -> Option<&TextBox> {
        match self {
            Self::TextBox(tb) => Some(tb),
            _ => None,
        }
    }

    pub fn picture(&self) -> Option<&PictureBox> {
        match self {
            Self::Picture(p) | Self::Raster(p) => Some(p),
            _ => None,
        }
    }

    pub fn relative_height(&self) -> u32 {
        match self {
            Self::TextBox(tb) => tb.relative_height,
            Self::Shape(s) => s.relative_height,
            Self::Picture(p) | Self::Raster(p) => p.relative_height,
            Self::Table(t) => t.relative_height,
        }
    }
}

pub fn collect_pictures(elements: &[PageElement]) -> Vec<&PictureBox> {
    elements.iter().filter_map(PageElement::picture).collect()
}

pub fn has_lists(elements: &[PageElement]) -> bool {
    // Only legacy packages that still set num_id need numbering.xml.
    // Current export paints list markers as literal runs.
    elements
        .iter()
        .any(|e| e.textbox().is_some_and(|t| t.num_id > 0))
}

pub fn collect_hyperlink_urls(elements: &[PageElement]) -> Vec<String> {
    let mut urls = Vec::new();
    for e in elements {
        match e {
            PageElement::TextBox(tb) => push_run_urls(&mut urls, &tb.runs),
            PageElement::Table(tbl) => {
                for row in &tbl.rows {
                    for cell in &row.cells {
                        push_run_urls(&mut urls, &cell.runs);
                    }
                }
            }
            PageElement::Shape(_) | PageElement::Picture(_) | PageElement::Raster(_) => {}
        }
    }
    urls
}

fn push_run_urls(urls: &mut Vec<String>, runs: &[TextRun]) {
    for run in runs {
        if let Some(url) = &run.hyperlink {
            if !urls.iter().any(|u| u == url) {
                urls.push(url.clone());
            }
        }
    }
}

/// Most common lock color among clickable hyperlink runs, if any.
pub fn hyperlink_theme_hex(elements: &[PageElement]) -> Option<String> {
    let mut counts: BTreeMap<String, usize> = BTreeMap::new();
    for e in elements {
        match e {
            PageElement::TextBox(tb) => count_link_colors(&mut counts, &tb.runs),
            PageElement::Table(tbl) => {
                for row in &tbl.rows {
                    for cell in &row.cells {
                        count_link_colors(&mut counts, &cell.runs);
                    }
                }
            }
            PageElement::Shape(_) | PageElement::Picture(_) | PageElement::Raster(_) => {}
        }
    }
    counts.into_iter().max_by_key(|(_, n)| *n).map(|(c, _)| c)
}

fn count_link_colors(counts: &mut BTreeMap<String, usize>, runs: &[TextRun]) {
    for run in runs {
        if run.hyperlink.is_some() {
            *counts.entry(run.color_hex.clone()).or_default() += 1;
        }
    }
}
