use std::path::Path;

use k2f_paint::Banner;

use crate::AppState;

/// Draw the published lock to PDF. No window; still works when the banner is broken.
pub fn export_pdf(app: &AppState, out: &Path) -> anyhow::Result<()> {
    app.export_pdf_to(out)?;
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
