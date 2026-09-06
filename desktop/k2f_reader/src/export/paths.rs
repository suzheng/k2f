use super::ExportFormat;
use std::path::{Path, PathBuf};

fn strip_suffix<'a>(name: &'a str, ext: &str) -> &'a str {
    let bytes = name.as_bytes();
    let needle = format!(".{ext}");
    let nlen = needle.len();
    if bytes.len() >= nlen && bytes[bytes.len() - nlen..].eq_ignore_ascii_case(needle.as_bytes()) {
        &name[..bytes.len() - nlen]
    } else {
        name
    }
}

/// Safe default filename for `title` and `format`.
pub fn export_file_name(title: &str, format: ExportFormat, page_count: usize) -> String {
    let cleaned: String = title
        .chars()
        .map(|c| {
            if c.is_control() || matches!(c, '/' | '\\' | ':' | '<' | '>' | '"' | '|' | '?' | '*') {
                '_'
            } else {
                c
            }
        })
        .collect();
    let stem = strip_suffix(cleaned.trim(), format.extension()).trim();
    let stem = if stem.is_empty() { "document" } else { stem };
    match format {
        ExportFormat::Png if page_count > 1 => format!("{stem}-pages.zip"),
        ExportFormat::Jpg if page_count > 1 => format!("{stem}-pages.zip"),
        _ => format!("{stem}.{}", format.extension()),
    }
}

pub fn default_export_path(
    source: &Path,
    title: &str,
    format: ExportFormat,
    page_count: usize,
) -> PathBuf {
    source.with_file_name(export_file_name(title, format, page_count))
}

pub fn ensure_extension(path: PathBuf, format: ExportFormat) -> PathBuf {
    let ext = format.extension();
    match path.extension().and_then(|e| e.to_str()) {
        Some(e) if e.eq_ignore_ascii_case(ext) => path,
        Some(e) if format == ExportFormat::Jpg && e.eq_ignore_ascii_case("jpeg") => path,
        _ => {
            let mut raw = path.into_os_string();
            raw.push(".");
            raw.push(ext);
            PathBuf::from(raw)
        }
    }
}

/// Back-compat helpers used in tests.
pub fn pdf_file_name(title: &str) -> String {
    export_file_name(title, ExportFormat::Pdf, 1)
}

pub fn default_pdf_path(source: &Path, title: &str) -> PathBuf {
    default_export_path(source, title, ExportFormat::Pdf, 1)
}

pub fn ensure_pdf_path(path: PathBuf) -> PathBuf {
    ensure_extension(path, ExportFormat::Pdf)
}
