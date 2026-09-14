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
    #[allow(dead_code)]
    Table(TableBox),
    #[allow(dead_code)]
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

#[allow(dead_code)]
#[derive(Clone, Debug)]
pub struct TableBox {
    pub node_id: String,
    pub rect: Rect,
}

impl PageElement {
    pub fn textbox(&self) -> Option<&TextBox> {
        match self {
            Self::TextBox(tb) => Some(tb),
            _ => None,
        }
    }
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
            PageElement::TextBox(tb) => {
                for run in &tb.runs {
                    if !run.color_hex.is_empty() {
                        out.insert(run.color_hex.clone());
                    }
                }
            }
            PageElement::Shape(s) => {
                if let Some(hex) = &s.fill_hex {
                    out.insert(hex.clone());
                }
                if let Some(hex) = &s.line_hex {
                    out.insert(hex.clone());
                }
            }
            PageElement::Picture(_) | PageElement::Table(_) | PageElement::Raster(_) => {}
        }
    }
}
