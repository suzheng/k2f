use crate::error::AgentError;
use k2f_markdown::{document_to_markdown, MarkdownEmitOptions};
use k2f_package::unpack_bytes;

pub fn k2f_to_markdown(bytes: &[u8]) -> Result<String, AgentError> {
    let pkg = unpack_bytes(bytes)?;
    Ok(document_to_markdown(
        &pkg.root,
        &pkg.manifest.running_blocks,
        MarkdownEmitOptions { hints: true },
    ))
}
