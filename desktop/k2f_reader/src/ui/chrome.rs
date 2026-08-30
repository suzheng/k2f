use crate::AppState;
use k2f_paint::Banner;

pub fn window_title(app: &AppState) -> String {
    let banner = app.banner_str();
    let title = app.title();
    if app.banner() == Banner::BrokenIntegrity {
        format!("K2F Reader — {banner} ({}) — {title}", app.status_code())
    } else {
        format!("K2F Reader — {banner} — {title}")
    }
}

/// On-screen strip: banner (+ status under broken) and 1-indexed pager, like the web HUD.
pub fn overlay_label(app: &AppState) -> String {
    let pages = app.page_count();
    let page = if pages == 0 {
        "—".to_string()
    } else {
        format!("{} / {pages}", app.page() + 1)
    };
    let zoom = format!("{}%", (app.zoom() * 100.0).round());
    if app.banner() == Banner::BrokenIntegrity {
        format!(
            "{} ({})  {page}  {zoom}",
            app.banner_str(),
            app.status_code()
        )
    } else {
        format!("{}  {page}  {zoom}", app.banner_str())
    }
}

pub fn hud_color(app: &AppState) -> u32 {
    match app.banner() {
        Banner::Signed => 0x0D6B3A,
        Banner::Unsigned => 0x3D5A80,
        Banner::Unlocked => 0x8A5A00,
        Banner::BrokenIntegrity => 0x9B1C1C,
        Banner::SignedButBroken => 0x6B0F0F,
    }
}
