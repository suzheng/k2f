use k2f_text::FontLibrary;
use std::path::PathBuf;

pub fn test_fonts() -> FontLibrary {
    let mut fonts = FontLibrary::new();
    let font_path: PathBuf =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../assets/fonts/Roboto-Regular.ttf");
    let data = std::fs::read(&font_path).expect("Failed to read bundled test font");
    fonts.add_font("default", data);
    fonts
}
