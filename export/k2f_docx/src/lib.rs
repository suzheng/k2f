mod classify;
mod coord;
mod effect;
mod error;
mod export;
mod geo;
mod header;
mod ir;
mod ooxml;
mod picture;
mod shape;
mod table;
mod text;
mod xml;

#[cfg(test)]
mod shape_tests;

pub use coord::{
    millipt_to_emu, millipt_to_twips, pt_to_emu, pt_to_twips, EMU_PER_MILLIPT_DEN,
    EMU_PER_MILLIPT_NUM,
};
pub use effect::{filter_chrome_ops, ChromeKeep};
pub use error::DocxError;
pub use export::{export_bytes, export_opened};
pub use ir::{ScriptPos, TextAlign, TextBox, TextRun};
pub use table::{can_emit_native_table, table_cell_wml};
pub use text::{infer_text_align, textbox_from_draw, textbox_wml};
pub use xml::escape_xml;
