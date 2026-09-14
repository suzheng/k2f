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
    pub vert_center: bool,
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
