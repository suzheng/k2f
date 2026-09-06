use crate::AppState;
use k2f_paint::Banner;

const ISSUER_UNKNOWN: &str = "issuer unknown";

/// Web `banner: "auto"` copy (`sdk/js/viewer/banner.js`). Quiet unsigned is hidden.
pub fn banner_copy(app: &AppState) -> Option<String> {
    match app.banner() {
        Banner::Unsigned => None,
        Banner::Signed => {
            let mut parts = vec![format!("Signed · {ISSUER_UNKNOWN}")];
            if let Some(g) = app.generated_by() {
                parts.push(format!("generated_by={g}"));
            }
            Some(parts.join(" · "))
        }
        Banner::Unlocked => Some("Draft — not published yet".into()),
        Banner::SignedButBroken => {
            let code = app.hash_code();
            let code = if code.is_empty() {
                app.status_code()
            } else {
                code
            };
            let mut s = format!(
                "Signature invalid — the lock no longer matches. The page below is the old published lock. ({code})"
            );
            if let Some(fp) = app.fingerprint() {
                s.push(' ');
                s.push_str(fp);
            }
            Some(s)
        }
        Banner::BrokenIntegrity => Some(format!(
            "Content and lock do not match. Do not sign. The page below is the old published lock. ({})",
            app.status_code()
        )),
    }
}

pub fn banner_compact(app: &AppState) -> bool {
    matches!(app.banner(), Banner::Signed | Banner::Unlocked)
}

pub fn banner_height_at(app: &AppState, scale: f32) -> u32 {
    match app.banner() {
        Banner::Unsigned => 0,
        Banner::Signed | Banner::Unlocked => {
            super::hud::dip(super::hud::BANNER_COMPACT_HEIGHT, scale)
        }
        Banner::BrokenIntegrity | Banner::SignedButBroken => {
            super::hud::dip(super::hud::BANNER_ALERT_HEIGHT, scale)
        }
    }
}

#[allow(dead_code)]
pub fn chrome_top(app: &AppState) -> u32 {
    chrome_top_at(app, 1.0)
}

pub fn chrome_top_at(app: &AppState, scale: f32) -> u32 {
    super::hud::dip(super::hud::TOOLBAR_HEIGHT, scale) + banner_height_at(app, scale)
}

/// Window Y of page 0 at scroll 0 (toolbar + optional warning + stage pad).
pub fn page_inset_y(app: &AppState) -> u32 {
    page_inset_y_at(app, 1.0)
}

pub fn page_inset_y_at(app: &AppState, scale: f32) -> u32 {
    chrome_top_at(app, scale) + super::hud::dip(super::hud::STAGE_PAD, scale)
}

pub fn window_chrome_h(app: &AppState) -> u32 {
    window_chrome_h_at(app, 1.0)
}

pub fn window_chrome_h_at(app: &AppState, scale: f32) -> u32 {
    chrome_top_at(app, scale)
        + super::hud::dip(super::hud::STATUS_HEIGHT, scale)
        + super::hud::dip(super::hud::STAGE_PAD, scale) * 2
}

#[allow(dead_code)]
pub fn status_pill(app: &AppState) -> Option<(&'static str, u32)> {
    match app.banner() {
        Banner::Signed => Some(("Signed", 0x0D6B3A)),
        Banner::Unlocked => Some(("Draft", 0x8A5A00)),
        _ => None,
    }
}

pub fn page_status_label(app: &AppState) -> String {
    let pages = app.page_count();
    if pages == 0 {
        "Page —".to_string()
    } else {
        format!("Page {} of {pages}", app.page() + 1)
    }
}

pub fn zoom_label(app: &AppState) -> String {
    format!("{}%", (app.zoom() * 100.0).round())
}

pub fn window_title(app: &AppState) -> String {
    let title = app.title();
    match app.banner() {
        Banner::BrokenIntegrity => format!(
            "K2F Reader — BROKEN_INTEGRITY ({}) — {title}",
            app.status_code()
        ),
        Banner::SignedButBroken => format!("K2F Reader — SIGNED_BUT_BROKEN — {title}"),
        Banner::Unlocked => format!("K2F Reader — Draft — {title}"),
        Banner::Signed => format!("K2F Reader — Signed — {title}"),
        Banner::Unsigned => title.to_string(),
    }
}

/// On-screen strip: status prefix (when needed), 1-indexed pager, and zoom.
pub fn overlay_label(app: &AppState) -> String {
    let pages = app.page_count();
    let page = if pages == 0 {
        "—".to_string()
    } else {
        format!("{} / {pages}", app.page() + 1)
    };
    let zoom = format!("{}%", (app.zoom() * 100.0).round());
    let prefix = match app.banner() {
        Banner::Unsigned => None,
        Banner::Signed => Some("Signed".to_string()),
        Banner::Unlocked => Some("Draft".to_string()),
        Banner::BrokenIntegrity => Some(format!("Integrity warning ({})", app.status_code())),
        Banner::SignedButBroken => {
            let code = app.hash_code();
            let code = if code.is_empty() {
                app.status_code()
            } else {
                code
            };
            Some(format!("Signature invalid ({code})"))
        }
    };
    match prefix {
        Some(p) => format!("{p}  {page}  {zoom}"),
        None => format!("{page}  {zoom}"),
    }
}

pub fn hud_color(app: &AppState) -> u32 {
    match app.banner() {
        Banner::Signed => 0x0D6B3A,
        Banner::Unsigned => 0x222222,
        Banner::Unlocked => 0x8A5A00,
        Banner::BrokenIntegrity => 0x9B1C1C,
        Banner::SignedButBroken => 0x6B0F0F,
    }
}
