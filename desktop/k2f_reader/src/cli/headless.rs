use std::path::Path;

use k2f_paint::Banner;

use crate::export::ExportFormat;
use crate::AppState;

/// Draw the published lock to PDF. No window; still works when the banner is broken.
pub fn export_pdf(app: &AppState, out: &Path) -> anyhow::Result<()> {
    app.export_pdf_to(out)?;
    eprintln!("wrote {}", out.display());
    Ok(())
}

/// Draw the published lock to PowerPoint. Same bytes as the GUI PPTX export.
pub fn export_pptx(app: &AppState, out: &Path) -> anyhow::Result<()> {
    app.export_to(ExportFormat::Pptx, out)?;
    eprintln!("wrote {}", out.display());
    Ok(())
}

/// Draw the published lock to Word. Same bytes as the GUI DOCX export.
pub fn export_docx(app: &AppState, out: &Path) -> anyhow::Result<()> {
    app.export_to(ExportFormat::Docx, out)?;
    eprintln!("wrote {}", out.display());
    Ok(())
}

/// Print banner on stdout. Broken / unlocked exits 1; status_code on stderr.
pub fn verify(app: &AppState) -> anyhow::Result<()> {
    println!("{}", app.banner_str());
    if matches!(app.banner(), Banner::Unsigned | Banner::Signed) {
        return Ok(());
    }
    eprintln!("{}", app.status_code());
    std::process::exit(1);
}
