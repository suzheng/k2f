use crate::align::{infer_text_align, line_spacing_spc_pts, should_wrap_lock, source_lines};
use crate::coord::pt_to_emu;
use crate::ir::{ScriptPos, TextAlign, TextBox, TextRun};
use k2f_core::{
    GeometryNode, GlyphPosition, ListMarkerType, Modifier, NodeContent, Pt, Rect, SemanticNode,
    TableDataSource, TextGlyphRun, TextPaintStyle,
};
use k2f_paint::parse_hex_rgba;
use std::collections::BTreeMap;
use ttf_parser::{name_id, Face, Language};

pub(crate) struct FontCtx {
    default_family: String,
    bytes: BTreeMap<String, Vec<u8>>,
}

impl FontCtx {
    pub(crate) fn new(fonts: &BTreeMap<String, Vec<u8>>) -> Self {
        Self {
            default_family: embedded_family(fonts).unwrap_or_else(|| "Roboto".into()),
            bytes: fonts.clone(),
        }
    }

    fn typeface(&self, family: &str) -> String {
        if let Some(data) = self.bytes_for(family) {
            if let Some(name) = family_from_bytes(data) {
                return name;
            }
        }
        if family == "default" {
            return self.default_family.clone();
        }
        family.to_string()
    }

    fn bytes_for(&self, family: &str) -> Option<&[u8]> {
        if let Some(b) = self.bytes.get(family) {
            return Some(b.as_slice());
        }
        for (path, b) in &self.bytes {
            let p = std::path::Path::new(path);
            if p.file_stem().is_some_and(|s| s == family) {
                return Some(b.as_slice());
            }
            if p.file_name().is_some_and(|s| s == family) {
                return Some(b.as_slice());
            }
        }
        for b in self.bytes.values() {
            if family_from_bytes(b).as_deref() == Some(family) {
                return Some(b.as_slice());
            }
        }
        if family == "default" {
            if let Some(b) = self.bytes.get("default") {
                return Some(b.as_slice());
            }
            if let Some((_, b)) = self.bytes.iter().find(|(k, _)| k.contains("Roboto")) {
                return Some(b.as_slice());
            }
            return self.bytes.values().next().map(|b| b.as_slice());
        }
        None
    }
}

pub(crate) fn textbox_from_draw(
    node: &SemanticNode,
    rect: &Rect,
    paint_runs: &[TextGlyphRun],
    geo: Option<&GeometryNode>,
    fonts: &FontCtx,
    page_idx: usize,
    total_pages: usize,
    list_start: u32,
) -> Option<TextBox> {
    if node.role == "math" || matches!(node.content, NodeContent::Math(_)) {
        return None;
    }
    let raw = k2f_core::node_text(node)?;
    if raw.is_empty() {
        return None;
    }
    let running = node.role.starts_with("running_");
    let runs = if running {
        let text = expand_page_vars(&raw, page_idx, total_pages);
        if text.is_empty() {
            return None;
        }
        let style = paint_runs
            .first()
            .map(|r| r.style.clone())
            .unwrap_or_else(fallback_style);
        let font_name = fonts.typeface(&style.font_family);
        split_runs(&text, &style, &node.modifiers, &font_name, 0)
    } else {
        runs_from_paint(&raw, paint_runs, &node.modifiers, geo, fonts)
    };
    if runs.is_empty() {
        return None;
    }
    let numbered = node.marker_type == Some(ListMarkerType::Number);
    let bullet = node.role == "list_item" || node.marker_type.is_some();
    let align = geo
        .map(|g| infer_text_align(g, &raw))
        .unwrap_or(TextAlign::Left);
    Some(TextBox {
        node_id: node.id.clone(),
        x_emu: pt_to_emu(rect.x),
        y_emu: pt_to_emu(rect.y),
        cx_emu: pt_to_emu(rect.width),
        cy_emu: pt_to_emu(rect.height),
        runs,
        align,
        bullet,
        numbered,
        preserve_whitespace: node.preserve_whitespace == Some(true) || node.role == "code_block",
        wrap: !running && should_wrap_lock(geo),
        line_spc_pts: line_spacing_spc_pts(geo),
        t_ins_emu: top_inset_emu(geo),
        mar_l_emu: if numbered || bullet {
            list_hanging_emu(geo)
        } else {
            0
        },
        list_start: if numbered { list_start.max(1) } else { 1 },
    })
}

pub(crate) fn list_start_at(root: &SemanticNode) -> BTreeMap<String, u32> {
    let mut out = BTreeMap::new();
    walk_list_starts(root, &mut out);
    out
}

fn walk_list_starts(node: &SemanticNode, out: &mut BTreeMap<String, u32>) {
    match &node.content {
        NodeContent::Container { children } => {
            scan_list_starts(children, out);
            for child in children {
                walk_list_starts(child, out);
            }
        }
        NodeContent::Table(spec) => {
            if let TableDataSource::Inline { rows } = &spec.data {
                for row in rows {
                    scan_list_starts(row, out);
                    for cell in row {
                        walk_list_starts(cell, out);
                    }
                }
            }
        }
        _ => {}
    }
}

fn scan_list_starts(nodes: &[SemanticNode], out: &mut BTreeMap<String, u32>) {
    let mut by_list: BTreeMap<String, u32> = BTreeMap::new();
    for node in nodes {
        if node.role != "list_item" || node.marker_type != Some(ListMarkerType::Number) {
            continue;
        }
        let Some(list_id) = node.list_id.as_deref() else {
            continue;
        };
        let n = by_list.entry(list_id.to_string()).or_insert(0);
        *n = n.saturating_add(1);
        out.insert(node.id.clone(), *n);
    }
}

fn list_hanging_emu(geo: Option<&GeometryNode>) -> i64 {
    let Some(geo) = geo else {
        return 0;
    };
    let lines = crate::align::source_lines(geo);
    if lines.is_empty() {
        return 0;
    }
    let box_w = geo.width.0;
    let min_left = lines
        .iter()
        .map(|g| crate::align::line_gaps(g, box_w).1)
        .min()
        .unwrap_or(0)
        .max(0);
    pt_to_emu(Pt(min_left))
}

fn top_inset_emu(geo: Option<&GeometryNode>) -> i64 {
    let geo = match geo {
        Some(g) => g,
        None => return 0,
    };
    let Some(line) = source_lines(geo).into_iter().next() else {
        return 0;
    };
    let y = line.iter().map(|g| g.y_offset.0).min().unwrap_or(0);
    if y < 1_000 {
        return 0;
    }
    pt_to_emu(Pt(y))
}

fn expand_page_vars(text: &str, page_idx: usize, total_pages: usize) -> String {
    if !text.contains("{{") {
        return text.to_string();
    }
    text.replace("{{page_current}}", &(page_idx + 1).to_string())
        .replace("{{page_total}}", &total_pages.to_string())
}

pub(crate) fn cell_runs(
    node: Option<&SemanticNode>,
    paint_runs: &[TextGlyphRun],
    fonts: &FontCtx,
    header_bold: bool,
) -> (Vec<TextRun>, bool) {
    let Some(node) = node else {
        return (Vec::new(), false);
    };
    let Some(text) = k2f_core::node_text(node) else {
        return (Vec::new(), false);
    };
    let mut style = paint_runs
        .first()
        .map(|r| r.style.clone())
        .unwrap_or_else(|| table_fallback_style(header_bold));
    if paint_runs.is_empty() && header_bold {
        style.bold = true;
    }
    let font_name = fonts.typeface(&style.font_family);
    let preserve = node.preserve_whitespace == Some(true) || node.role == "code_block";
    (
        split_runs(text, &style, &node.modifiers, &font_name, 0),
        preserve,
    )
}

fn runs_from_paint(
    text: &str,
    paint_runs: &[TextGlyphRun],
    modifiers: &[Modifier],
    geo: Option<&GeometryNode>,
    fonts: &FontCtx,
) -> Vec<TextRun> {
    if paint_runs.is_empty() {
        let style = fallback_style();
        let font_name = fonts.typeface(&style.font_family);
        return split_runs(text, &style, modifiers, &font_name, 0);
    }
    let default_style = &paint_runs[0].style;
    let mut spans: Vec<(usize, usize, &TextPaintStyle, Vec<&GlyphPosition>)> = Vec::new();
    for pr in paint_runs {
        let Some((bs, be, glyphs)) = paint_byte_range(text, pr, geo) else {
            continue;
        };
        if bs < be {
            spans.push((bs, be, &pr.style, glyphs));
        }
    }
    spans.sort_by_key(|(bs, _, _, _)| *bs);
    let mut out = Vec::new();
    if let Some(&(first, _, _, _)) = spans.first() {
        let mut pos = first;
        for (bs, be, style, glyphs) in spans {
            if be <= pos {
                continue;
            }
            let start = bs.max(pos);
            if start > pos {
                out.extend(split_piece(
                    text,
                    pos,
                    start,
                    default_style,
                    modifiers,
                    fonts,
                    &[],
                ));
            }
            out.extend(split_piece(
                text, start, be, style, modifiers, fonts, &glyphs,
            ));
            pos = pos.max(be);
        }
    }
    if out.is_empty() {
        out.extend(split_piece(
            text,
            0,
            text.len(),
            default_style,
            modifiers,
            fonts,
            &[],
        ));
    }
    out
}

fn paint_byte_range<'a>(
    text: &str,
    pr: &TextGlyphRun,
    geo: Option<&'a GeometryNode>,
) -> Option<(usize, usize, Vec<&'a GlyphPosition>)> {
    let geo = geo?;
    let start = pr.glyph_range[0];
    let end = pr.glyph_range[1].min(geo.glyphs.len());
    if start >= end {
        return None;
    }
    let glyphs: Vec<&GlyphPosition> = geo.glyphs[start..end]
        .iter()
        .filter(|g| g.cluster != GlyphPosition::CLUSTER_NOT_SOURCE)
        .collect();
    if glyphs.is_empty() {
        return None;
    }
    let cs = glyphs.iter().map(|g| g.cluster).min()? as usize;
    let ce = glyphs.iter().map(|g| g.cluster).max()? as usize + 1;
    Some((char_to_byte(text, cs), char_to_byte(text, ce), glyphs))
}

fn split_piece(
    text: &str,
    bs: usize,
    be: usize,
    style: &TextPaintStyle,
    modifiers: &[Modifier],
    fonts: &FontCtx,
    glyphs: &[&GlyphPosition],
) -> Vec<TextRun> {
    let font_name = fonts.typeface(&style.font_family);
    let slice =
        if bs < be && be <= text.len() && text.is_char_boundary(bs) && text.is_char_boundary(be) {
            &text[bs..be]
        } else {
            return Vec::new();
        };
    split_runs(
        slice,
        style,
        &shift_modifiers(modifiers, bs, be),
        &font_name,
        tracking_spc(glyphs, style, fonts),
    )
}

/// Extra lock advance vs the face's native advance, as DrawingML `spc`
/// (hundredths of a point). Matches Word `w:spacing` inferred from the same glyphs.
fn tracking_spc(glyphs: &[&GlyphPosition], style: &TextPaintStyle, fonts: &FontCtx) -> i32 {
    if glyphs.len() < 2 {
        return 0;
    }
    let Some(data) = fonts.bytes_for(&style.font_family) else {
        return 0;
    };
    if data.len() > 512_000 {
        return 0;
    }
    let Ok(face) = ttf_parser::Face::parse(data, 0) else {
        return 0;
    };
    let upem = i128::from(face.units_per_em());
    if upem == 0 {
        return 0;
    }
    let mut extras = Vec::new();
    for g in glyphs.iter().take(glyphs.len() - 1) {
        let Ok(gid) = u16::try_from(g.glyph_id) else {
            return 0;
        };
        let Some(adv) = face.glyph_hor_advance(ttf_parser::GlyphId(gid)) else {
            return 0;
        };
        let font_adv = i128::from(adv) * style.font_size.0 / upem;
        extras.push(g.x_advance.0 - font_adv);
    }
    if extras.is_empty() {
        return 0;
    }
    extras.sort_unstable();
    let extra = extras[extras.len() / 2];
    i32::try_from(extra / 10).unwrap_or(0)
}

fn shift_modifiers(modifiers: &[Modifier], bs: usize, be: usize) -> Vec<Modifier> {
    modifiers
        .iter()
        .filter_map(|m| {
            let [s, e] = m.range;
            let start = s.max(bs);
            let end = e.min(be);
            if start >= end {
                return None;
            }
            Some(Modifier {
                range: [start - bs, end - bs],
                mod_type: m.mod_type.clone(),
                intent: m.intent.clone(),
            })
        })
        .collect()
}

fn char_to_byte(text: &str, char_idx: usize) -> usize {
    text.char_indices()
        .nth(char_idx)
        .map(|(i, _)| i)
        .unwrap_or(text.len())
}

fn fallback_style() -> TextPaintStyle {
    TextPaintStyle {
        font_family: "default".into(),
        font_size: Pt(12_000),
        color: "#000000".into(),
        bold: false,
        italic: false,
        strikethrough: false,
        underline: false,
    }
}

fn table_fallback_style(bold: bool) -> TextPaintStyle {
    TextPaintStyle {
        font_family: "default".into(),
        font_size: Pt(11_000),
        color: "#000000".into(),
        bold,
        italic: false,
        strikethrough: false,
        underline: false,
    }
}

fn split_runs(
    text: &str,
    style: &TextPaintStyle,
    modifiers: &[Modifier],
    font_name: &str,
    tracking_spc: i32,
) -> Vec<TextRun> {
    let base = run_from_style("", style, font_name, tracking_spc);
    let mut cuts = vec![0usize, text.len()];
    for m in modifiers {
        let [s, e] = m.range;
        if s <= e && e <= text.len() && text.is_char_boundary(s) && text.is_char_boundary(e) {
            cuts.push(s);
            cuts.push(e);
        }
    }
    cuts.sort_unstable();
    cuts.dedup();
    let mut out = Vec::new();
    for w in cuts.windows(2) {
        let (a, b) = (w[0], w[1]);
        if a >= b {
            continue;
        }
        let Ok(piece) = std::str::from_utf8(&text.as_bytes()[a..b]) else {
            continue;
        };
        if piece.is_empty() {
            continue;
        }
        let mut run = base.clone();
        run.text = piece.to_string();
        for m in modifiers {
            let [s, e] = m.range;
            if s <= a && b <= e {
                apply_modifier(&mut run, m);
            }
        }
        out.push(run);
    }
    out
}

fn run_from_style(
    text: &str,
    style: &TextPaintStyle,
    font_name: &str,
    tracking_spc: i32,
) -> TextRun {
    let sz = (style.font_size.0 / 10).clamp(100, i32::MAX as i128) as i32;
    TextRun {
        text: text.to_string(),
        font_name: font_name.to_string(),
        sz_hundredths_pt: sz,
        bold: style.bold,
        italic: style.italic,
        underline: style.underline,
        strike: style.strikethrough,
        color_hex: color_hex(&style.color),
        hyperlink: None,
        script: ScriptPos::Baseline,
        tracking_spc,
    }
}

fn apply_modifier(run: &mut TextRun, m: &Modifier) {
    match m.mod_type.as_str() {
        "emphasis" => run.bold = true,
        "underline" => run.underline = true,
        "strikethrough" => run.strike = true,
        "link" => run.hyperlink = Some(m.intent.clone()),
        "subscript" => run.script = ScriptPos::Sub,
        "superscript" => run.script = ScriptPos::Super,
        "math" | "syntax_highlight" => {}
        _ => {}
    }
}

fn color_hex(color: &str) -> String {
    let hex = parse_hex_rgba(color)
        .map(|[r, g, b, _]| format!("{r:02X}{g:02X}{b:02X}"))
        .unwrap_or_else(|| "000000".into());
    pin_office_srgb(&hex)
}

/// PowerPoint maps RGB `000000` / `FFFFFF` onto theme `tx1`/`bg1`. Dark Mode
/// remaps those slots even when they are stored as `a:srgbClr`.
fn pin_office_srgb(hex: &str) -> String {
    match hex {
        "000000" => "000001".into(),
        "FFFFFF" => "FFFFFE".into(),
        _ => hex.to_string(),
    }
}

fn embedded_family(fonts: &BTreeMap<String, Vec<u8>>) -> Option<String> {
    fonts
        .get("default")
        .or_else(|| fonts.values().next())
        .and_then(|b| family_from_bytes(b))
}

fn family_from_bytes(data: &[u8]) -> Option<String> {
    let face = Face::parse(data, 0).ok()?;
    let mut fallback = None;
    for name in face.names() {
        if name.name_id != name_id::FAMILY || !name.is_unicode() {
            continue;
        }
        let Some(s) = name.to_string() else {
            continue;
        };
        if name.language() == Language::English_UnitedStates {
            return Some(s);
        }
        if fallback.is_none() {
            fallback = Some(s);
        }
    }
    fallback
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn emphasis_splits_and_sets_bold() {
        let style = fallback_style();
        let mods = [Modifier {
            range: [0, 5],
            mod_type: "emphasis".into(),
            intent: "critical".into(),
        }];
        let runs = split_runs("Hello world", &style, &mods, "Roboto", 0);
        assert_eq!(runs.len(), 2);
        assert_eq!(runs[0].text, "Hello");
        assert!(runs[0].bold);
        assert_eq!(runs[1].text, " world");
        assert!(!runs[1].bold);
    }

    #[test]
    fn superscript_sets_script_not_italic() {
        let style = fallback_style();
        let mods = [Modifier {
            range: [8, 11],
            mod_type: "superscript".into(),
            intent: "superscript".into(),
        }];
        let runs = split_runs("Zheng Su1,* x", &style, &mods, "Roboto", 0);
        let super_run = runs.iter().find(|r| r.text == "1,*").expect("super");
        assert_eq!(super_run.script, ScriptPos::Super);
        assert!(!super_run.italic);
    }

    #[test]
    fn black_and_white_are_not_office_automatic() {
        assert_eq!(color_hex("#000000"), "000001");
        assert_eq!(color_hex("#FFFFFF"), "FFFFFE");
        assert_eq!(color_hex("#C1002A"), "C1002A");
    }

    #[test]
    fn paint_range_skips_unpainted_prefix() {
        let text = "HelloWorld";
        let fonts = FontCtx {
            default_family: "Roboto".into(),
            bytes: BTreeMap::new(),
        };
        let style = fallback_style();
        let geo = GeometryNode {
            id: "g".into(),
            x: Pt(0),
            y: Pt(0),
            width: Pt(80_000),
            height: Pt(20_000),
            glyphs: (5..10)
                .map(|i| GlyphPosition {
                    glyph_id: 1,
                    cluster: i,
                    x_offset: Pt((i - 5) as i128 * 8_000),
                    y_offset: Pt(12_000),
                    x_advance: Pt(8_000),
                    y_advance: Pt(0),
                })
                .collect(),
            text_runs: vec![],
            fill_rects: vec![],
            children: vec![],
        };
        let runs = runs_from_paint(
            text,
            &[TextGlyphRun {
                glyph_range: [0, 5],
                style,
            }],
            &[],
            Some(&geo),
            &fonts,
        );
        let blob: String = runs.iter().map(|r| r.text.as_str()).collect();
        assert_eq!(blob, "World");
    }

    #[test]
    fn alias_stem_resolves_to_ttf_family() {
        let mut fonts = BTreeMap::new();
        let path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../assets/fonts/Roboto-Regular.ttf");
        fonts.insert(
            "assets/fonts/Roboto-Regular.ttf".into(),
            std::fs::read(path).unwrap(),
        );
        let ctx = FontCtx::new(&fonts);
        assert_eq!(ctx.typeface("Roboto-Regular"), "Roboto");
        assert_eq!(ctx.typeface("default"), "Roboto");
        assert!(ctx.bytes_for("Roboto-Regular").is_some());
    }

    #[test]
    fn tracking_spc_from_lock_extra_advance() {
        let mut fonts = BTreeMap::new();
        let path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../assets/fonts/Roboto-Regular.ttf");
        let bytes = std::fs::read(path).unwrap();
        fonts.insert("assets/fonts/Roboto-Regular.ttf".into(), bytes.clone());
        let ctx = FontCtx::new(&fonts);
        let face = ttf_parser::Face::parse(&bytes, 0).unwrap();
        let gid = face.glyph_index('A').unwrap();
        let native = i128::from(face.glyph_hor_advance(gid).unwrap()) * 12_000
            / i128::from(face.units_per_em());
        let extra = 3_500i128;
        let glyphs = [
            GlyphPosition {
                glyph_id: u32::from(gid.0),
                cluster: 0,
                x_offset: Pt(0),
                y_offset: Pt(0),
                x_advance: Pt(native + extra),
                y_advance: Pt(0),
            },
            GlyphPosition {
                glyph_id: u32::from(gid.0),
                cluster: 1,
                x_offset: Pt(native + extra),
                y_offset: Pt(0),
                x_advance: Pt(native),
                y_advance: Pt(0),
            },
        ];
        let refs: Vec<&GlyphPosition> = glyphs.iter().collect();
        let style = TextPaintStyle {
            font_family: "Roboto-Regular".into(),
            font_size: Pt(12_000),
            color: "#000000".into(),
            bold: false,
            italic: false,
            strikethrough: false,
            underline: false,
        };
        assert_eq!(tracking_spc(&refs, &style, &ctx), 350);
    }
}
