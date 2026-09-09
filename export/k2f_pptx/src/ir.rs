use std::collections::BTreeMap;

/// Crate-local deck model. PaintOps are classified into this, then written as OOXML.
#[derive(Clone, Debug)]
pub struct DeckIR {
    pub title: String,
    pub slides: Vec<SlideIR>,
}

#[derive(Clone, Debug)]
pub struct SlideIR {
    pub width_emu: i64,
    pub height_emu: i64,
    pub bg_hex: String,
    pub elements: Vec<SlideElement>,
}

#[derive(Clone, Debug)]
pub enum SlideElement {
    TextBox(TextBox),
    Shape(ShapeBox),
    Picture(PictureBox),
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
    pub preserve_whitespace: bool,
    pub wrap: bool,
    /// DrawingML `a:spcPts` (hundredths of a point), from lock line-to-line delta.
    pub line_spc_pts: Option<i32>,
    /// Role `padding_pt.top` baked into first-line glyph `y_offset`.
    pub t_ins_emu: i64,
    /// Lock glyph left gap (role padding) as DrawingML `lIns`.
    pub l_ins_emu: i64,
    /// Lock glyph right gap as DrawingML `rIns` (right-aligned only).
    pub r_ins_emu: i64,
    /// Hanging indent for list markers (DrawingML `marL` / negative `indent`).
    pub mar_l_emu: i64,
    /// 1-based `a:buAutoNum startAt`. Each list item is its own text box.
    pub list_start: u32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TextAlign {
    Left,
    Center,
    Right,
    Justify,
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
    pub sz_hundredths_pt: i32,
    pub bold: bool,
    pub italic: bool,
    pub underline: bool,
    pub strike: bool,
    pub color_hex: String,
    pub hyperlink: Option<String>,
    pub script: ScriptPos,
    /// DrawingML `a:rPr spc` (hundredths of a point). Extra lock glyph advance.
    pub tracking_spc: i32,
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
    #[allow(dead_code)]
    pub fill_alpha_ppt: Option<u16>,
    pub corner_emu: i64,
    pub line_hex: Option<String>,
    pub line_w_emu: i64,
    pub line_dash: LineDash,
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
}

#[derive(Clone, Debug)]
pub struct TableBox {
    pub node_id: String,
    pub x_emu: i64,
    pub y_emu: i64,
    pub cx_emu: i64,
    pub cy_emu: i64,
    pub col_widths_emu: Vec<i64>,
    pub rows: Vec<TableRow>,
}

#[derive(Clone, Debug)]
pub struct TableRow {
    pub height_emu: i64,
    pub cells: Vec<TableCell>,
}

#[derive(Clone, Debug)]
pub struct TableCell {
    pub node_id: String,
    pub runs: Vec<TextRun>,
    pub align: TextAlign,
    pub fill_hex: Option<String>,
    pub preserve_whitespace: bool,
    pub borders: CellBorders,
    pub vert_center: bool,
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
    pub w_emu: i64,
    pub dash: LineDash,
}

/// Most common lock color among clickable hyperlink runs, if any.
pub fn hyperlink_theme_hex(deck: &DeckIR) -> Option<String> {
    let mut counts: BTreeMap<String, usize> = BTreeMap::new();
    for slide in &deck.slides {
        for el in &slide.elements {
            match el {
                SlideElement::TextBox(tb) => count_link_colors(&mut counts, &tb.runs),
                SlideElement::Table(t) => {
                    for row in &t.rows {
                        for cell in &row.cells {
                            count_link_colors(&mut counts, &cell.runs);
                        }
                    }
                }
                SlideElement::Shape(_) | SlideElement::Picture(_) | SlideElement::Raster(_) => {}
            }
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
