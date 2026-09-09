mod common;

use common::{invoice_bytes, pack_with_tampered_lock, published_invoice_bytes, signed_bytes};
use k2f_reader::ui::{banner_copy, overlay_label, page_inset_y, window_title, Session, HUD_HEIGHT};
use k2f_reader::AppState;

#[test]
fn window_title_is_banner_and_document_title() {
    let app = AppState::open(&invoice_bytes()).unwrap();
    let title = window_title(&app);
    assert_eq!(
        title,
        app.title(),
        "unsigned title is document title only, got {title}"
    );
}

#[test]
fn overlay_shows_banner_and_one_indexed_pager() {
    let mut app = AppState::open(&published_invoice_bytes()).unwrap();
    assert!(app.page_count() >= 2);
    let label = overlay_label(&app);
    assert!(
        !label.contains("UNSIGNED"),
        "quiet unsigned must not prefix HUD with banner code, got {label}"
    );
    assert!(label.contains("1 /"), "web pager is 1-indexed, got {label}");
    assert!(label.contains(&format!("{}", app.page_count())), "{label}");

    app.next_page();
    let next = overlay_label(&app);
    assert!(
        next.contains("2 /"),
        "Right-arrow page change must update the HUD, got {next}"
    );
}

#[test]
fn broken_integrity_chrome_shows_status_code() {
    let app = AppState::open(&pack_with_tampered_lock(&invoice_bytes(), |lock| {
        lock.engine_version = "9.9.9".into();
    }))
    .unwrap();
    assert_eq!(app.banner_str(), "BROKEN_INTEGRITY");
    assert_eq!(app.status_code(), "ENGINE_MISMATCH");

    let title = window_title(&app);
    assert!(title.contains("BROKEN_INTEGRITY"), "{title}");
    assert!(
        title.contains("ENGINE_MISMATCH"),
        "window title still carries status_code, got {title}"
    );

    let label = overlay_label(&app);
    assert!(
        !label.contains("BROKEN_INTEGRITY"),
        "on-screen chrome uses web auto copy, not the banner enum, got {label}"
    );
    assert!(label.contains("Integrity warning"), "{label}");
    assert!(label.contains("ENGINE_MISMATCH"), "{label}");

    let copy = banner_copy(&app).expect("broken integrity shows a banner");
    assert!(
        copy.contains("Content and lock do not match"),
        "web AUTO_COPY for BROKEN_INTEGRITY, got {copy}"
    );
    assert!(
        !copy.contains("BROKEN_INTEGRITY"),
        "painted banner must not lead with the enum name, got {copy}"
    );
    assert!(copy.contains("ENGINE_MISMATCH"), "{copy}");
}

#[test]
fn unsigned_has_no_integrity_banner() {
    let app = AppState::open(&invoice_bytes()).unwrap();
    assert!(banner_copy(&app).is_none());
    assert_eq!(page_inset_y(&app), HUD_HEIGHT);
}

#[test]
fn signed_uses_compact_web_copy() {
    let app = AppState::open(&signed_bytes(&invoice_bytes())).unwrap();
    let copy = banner_copy(&app).expect("signed shows a compact strip");
    assert!(
        copy.starts_with("Signed ·"),
        "web signed auto copy, got {copy}"
    );
    assert!(!copy.contains("SIGNED"), "{copy}");
    assert!(
        page_inset_y(&app) > HUD_HEIGHT,
        "compact signed strip sits above the page"
    );
}

#[test]
fn published_invoice_title_renders_cjk() {
    let session = Session::new(AppState::open(&published_invoice_bytes()).unwrap()).unwrap();
    let title = session.app().title();
    assert!(
        title
            .chars()
            .any(|c| ('\u{4E00}'..='\u{9FFF}').contains(&c)),
        "published invoice title must include CJK, got {title:?}"
    );
    let (w, h) = session.scaled_size();
    let frame = session.compose_frame(w, h);
    let mut ink_cols = 0u32;
    for x in 20..120.min(w) {
        let mut col_ink = false;
        for y in 8..48.min(h) {
            let p = frame[(y * w + x) as usize];
            let r = (p >> 16) & 0xff;
            let g = (p >> 8) & 0xff;
            let b = p & 0xff;
            if r > 200 && g > 200 && b > 200 {
                col_ink = true;
                break;
            }
        }
        if col_ink {
            ink_cols += 1;
        }
    }
    assert!(
        ink_cols >= 8,
        "CJK title must paint multiple light glyph columns in the toolbar, got {ink_cols}"
    );
}

fn light(p: u32) -> bool {
    let r = (p >> 16) & 0xff;
    let g = (p >> 8) & 0xff;
    let b = p & 0xff;
    r > 180 && g > 180 && b > 180
}

fn glyph_bounds(
    frame: &[u32],
    w: u32,
    x0: u32,
    x1: u32,
    y0: u32,
    y1: u32,
    gap: u32,
) -> Vec<(u32, u32, u32, u32)> {
    let mut cols = Vec::new();
    for x in x0..x1 {
        let ys: Vec<u32> = (y0..y1)
            .filter(|&y| light(frame[(y * w + x) as usize]))
            .collect();
        if !ys.is_empty() {
            cols.push((x, *ys.iter().min().unwrap(), *ys.iter().max().unwrap()));
        }
    }
    let mut glyphs = Vec::new();
    let mut cur = Vec::new();
    let mut prev: Option<u32> = None;
    for c in cols {
        if let Some(p) = prev {
            if c.0 > p + gap {
                glyphs.push(std::mem::take(&mut cur));
            }
        }
        cur.push(c);
        prev = Some(c.0);
    }
    if !cur.is_empty() {
        glyphs.push(cur);
    }
    glyphs
        .into_iter()
        .map(|g| {
            let gx0 = g[0].0;
            let gx1 = g.last().unwrap().0;
            let top = g.iter().map(|c| c.1).min().unwrap();
            let bot = g.iter().map(|c| c.2).max().unwrap();
            (gx0, gx1, top, bot)
        })
        .collect()
}

#[test]
#[ignore = "optional maintainer check: set K2F_READER_UI_FIXTURE to a .K2F path"]
fn toolbar_type_sits_on_one_baseline() {
    let fixture = std::env::var("K2F_READER_UI_FIXTURE").expect("K2F_READER_UI_FIXTURE");
    let bytes = std::fs::read(&fixture).expect("read K2F_READER_UI_FIXTURE");
    let mut session = Session::new(AppState::open(&bytes).unwrap()).unwrap();
    session.set_scale(2.0);
    let w = 2560u32;
    let h = 800u32;
    session.set_window_size(w, h);
    let frame = session.compose_frame(w, h);
    let bar_h = 112u32;

    let title = glyph_bounds(&frame, w, 0, 400, 0, 56, 2);
    assert!(title.len() >= 5, "Bookly letters, got {title:?}");
    let bottoms: Vec<u32> = title.iter().map(|g| g.3).collect();
    let base = bottoms[0];
    for (i, bot) in bottoms.iter().enumerate() {
        assert!(
            bot.abs_diff(base) <= 4 || i + 1 == title.len(),
            "title glyph {i} bot={bot} base={base} {title:?}"
        );
    }

    let mut bx0 = w;
    let mut bx1 = 0u32;
    let mut by0 = bar_h;
    let mut by1 = 0u32;
    for y in 0..bar_h {
        for x in (w / 2)..w {
            let p = frame[(y * w + x) as usize];
            let r = (p >> 16) & 0xff;
            let g = (p >> 8) & 0xff;
            let b = p & 0xff;
            if r < 40 && g < 160 && b > 200 {
                bx0 = bx0.min(x);
                bx1 = bx1.max(x);
                by0 = by0.min(y);
                by1 = by1.max(y);
            }
        }
    }
    let export = glyph_bounds(&frame, w, bx0, bx1 + 1, by0, by1 + 1, 1);
    assert!(export.len() >= 5, "Export letters, got {export:?}");
    let e_bot = export[0].3;
    let o_bot = export[3].3;
    assert!(
        e_bot.abs_diff(o_bot) <= 2,
        "Export E/o must share a baseline, {export:?}"
    );
}
