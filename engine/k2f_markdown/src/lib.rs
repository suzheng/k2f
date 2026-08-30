//! K2F semantic tree → Markdown (no layout / no package unpack).
//!
//! Used by `k2f_sdk::k2f_to_markdown` (full package) and by viewers for
//! selection copy without pulling `k2f_layout`.

mod emit;
mod encode;
mod escape;
mod range;
mod select;
mod slice;

pub use emit::{document_to_markdown, nodes_to_markdown, MarkdownEmitOptions};
pub use encode::apply_inline_markdown;
pub use escape::escape_md;
pub use range::{byte_to_char, char_to_byte};
pub use select::{selection_to_markdown, NodeCharRange};
pub use slice::slice_text_node;
