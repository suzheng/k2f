use super::{ensure_extension, export_file_name, ExportFormat};
use std::path::{Path, PathBuf};

pub fn pick_save_path(
    format: ExportFormat,
    title: &str,
    page_count: usize,
    source: Option<&Path>,
) -> Option<PathBuf> {
    let (label, exts) = format.dialog_filter();
    let mut dlg = rfd::FileDialog::new()
        .add_filter(label, exts)
        .set_file_name(export_file_name(title, format, page_count));
    if let Some(dir) = source
        .and_then(Path::parent)
        .filter(|p| !p.as_os_str().is_empty())
    {
        dlg = dlg.set_directory(dir);
    }
    dlg.save_file()
    .map(|p| ensure_extension(p, format))
}
