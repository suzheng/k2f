use std::fs;
use std::path::{Path, PathBuf};

pub fn collect_md(dir: &Path, out: &mut Vec<PathBuf>) -> anyhow::Result<()> {
    for entry in fs::read_dir(dir)? {
        let path = entry?.path();
        if path.is_dir() {
            collect_md(&path, out)?;
        } else if path.extension().and_then(|e| e.to_str()) == Some("md") {
            out.push(path);
        }
    }
    Ok(())
}

pub fn convert_markdown(
    source: &Path,
    dest: &Path,
    template: &str,
    font_bytes: Option<&[u8]>,
) -> anyhow::Result<()> {
    let md = fs::read_to_string(source)?;
    let title = source
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("Document");
    let mut opts = k2f_sdk::MarkdownOptions::new(title, template)?;
    if let Some(bytes) = font_bytes {
        opts.font_bytes = Some(bytes.to_vec());
    }
    let result = k2f_sdk::markdown_to_k2f(&md, opts).map_err(|e| anyhow::anyhow!("{e}"))?;
    if let Some(parent) = dest.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(dest, result.bytes)?;
    Ok(())
}
