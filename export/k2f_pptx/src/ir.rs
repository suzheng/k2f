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
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TextAlign {
    Left,
    #[allow(dead_code)]
    Center,
    #[allow(dead_code)]
    Right,
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
    pub fill_hex: Option<String>,
    pub preserve_whitespace: bool,
}
