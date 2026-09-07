use miniz_oxide::deflate::{compress_to_vec_zlib, CompressionLevel};
use pdf_writer::types::{CidFontType, FontFlags, SystemInfo, UnicodeCmap};
use pdf_writer::{Filter, Name, Pdf, Rect, Ref, Str};
use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::path::Path;
use ttf_parser::{Face, GlyphId};

use crate::ids::Alloc;
use crate::select::kind::{font_kind, FontKind};

const SYS: SystemInfo<'static> = SystemInfo {
    registry: Str(b"Adobe"),
    ordering: Str(b"Identity"),
    supplement: 0,
};

pub struct FontSet {
    pub slots: Vec<FontSlot>,
    pub family_to_font: HashMap<String, String>,
}

pub struct FontSlot {
    pub name: String,
    pub type0: Ref,
    pub cid: Ref,
    pub desc: Ref,
    pub file: Ref,
    pub cmap: Ref,
    pub path: String,
    pub kind: FontKind,
    pub ps_name: String,
    pub cmap_name: String,
}

pub fn reserve(alloc: &mut Alloc, fonts: &BTreeMap<String, Vec<u8>>) -> FontSet {
    let mut slots = Vec::new();
    let mut family_to_font = HashMap::new();
    for (i, (path, bytes)) in fonts
        .iter()
        .filter(|(path, _)| k2f_core::is_font_face_path(path))
        .enumerate()
    {
        let name = format!("F{i}");
        let ps_name = pdf_font_name(path, i);
        let cmap_name = format!("K2FU{i}");
        let slot = FontSlot {
            name: name.clone(),
            type0: alloc.bump(),
            cid: alloc.bump(),
            desc: alloc.bump(),
            file: alloc.bump(),
            cmap: alloc.bump(),
            path: path.clone(),
            kind: font_kind(bytes),
            ps_name,
            cmap_name,
        };
        family_to_font.insert(path.clone(), name.clone());
        if let Some(stem) = Path::new(path).file_stem() {
            family_to_font
                .entry(stem.to_string_lossy().into_owned())
                .or_insert_with(|| name.clone());
        }
        if !family_to_font.contains_key("default") {
            family_to_font.insert("default".into(), name.clone());
        }
        slots.push(slot);
    }
    FontSet {
        slots,
        family_to_font,
    }
}

pub fn write(
    pdf: &mut Pdf,
    set: &FontSet,
    fonts: &BTreeMap<String, Vec<u8>>,
    faces: &HashMap<String, Face<'_>>,
    gid_maps: &[BTreeMap<u16, String>],
) {
    let empty = BTreeMap::new();
    for (i, slot) in set.slots.iter().enumerate() {
        let Some(bytes) = fonts.get(&slot.path) else {
            continue;
        };
        let face = faces
            .get(&slot.path)
            .or_else(|| k2f_paint::face_for(faces, "default"));
        write_slot(pdf, slot, bytes, face, gid_maps.get(i).unwrap_or(&empty));
    }
}

fn write_slot(
    pdf: &mut Pdf,
    slot: &FontSlot,
    bytes: &[u8],
    face: Option<&Face<'_>>,
    gid_map: &BTreeMap<u16, String>,
) {
    let ps = Name(slot.ps_name.as_bytes());
    {
        let mut t0 = pdf.type0_font(slot.type0);
        t0.base_font(ps);
        t0.encoding_predefined(Name(b"Identity-H"));
        t0.descendant_font(slot.cid);
        t0.to_unicode(slot.cmap);
    }
    {
        let mut cid = pdf.cid_font(slot.cid);
        match slot.kind {
            FontKind::TrueType => cid.subtype(CidFontType::Type2),
            FontKind::CffOpenType => cid.subtype(CidFontType::Type0),
        };
        cid.base_font(ps);
        cid.system_info(SYS);
        cid.font_descriptor(slot.desc);
        if slot.kind == FontKind::TrueType {
            cid.cid_to_gid_map_predefined(Name(b"Identity"));
        }
        if let Some(face) = face {
            write_widths(&mut cid, face, &gid_map.keys().copied().collect());
        }
    }
    if let Some(face) = face {
        write_descriptor(pdf, slot, face);
    } else {
        let mut d = pdf.font_descriptor(slot.desc);
        d.name(ps);
        d.flags(FontFlags::SYMBOLIC);
        d.bbox(Rect::new(0.0, 0.0, 1000.0, 1000.0));
        d.italic_angle(0.0);
        d.ascent(800.0);
        d.descent(-200.0);
        d.cap_height(700.0);
        d.stem_v(80.0);
        match slot.kind {
            FontKind::TrueType => d.font_file2(slot.file),
            FontKind::CffOpenType => d.font_file3(slot.file),
        };
    }
    write_font_file(pdf, slot, bytes);
    write_cmap(pdf, slot, gid_map);
}

fn write_descriptor(pdf: &mut Pdf, slot: &FontSlot, face: &Face<'_>) {
    let units = face.units_per_em().max(1) as f32;
    let scale = 1000.0 / units;
    let bb = face.global_bounding_box();
    let mut d = pdf.font_descriptor(slot.desc);
    d.name(Name(slot.ps_name.as_bytes()));
    d.flags(FontFlags::SYMBOLIC);
    d.bbox(Rect::new(
        bb.x_min as f32 * scale,
        bb.y_min as f32 * scale,
        bb.x_max as f32 * scale,
        bb.y_max as f32 * scale,
    ));
    d.italic_angle(face.italic_angle().unwrap_or(0.0));
    d.ascent(face.ascender() as f32 * scale);
    d.descent(face.descender() as f32 * scale);
    d.cap_height(face.capital_height().unwrap_or(face.ascender()) as f32 * scale);
    d.stem_v(80.0);
    match slot.kind {
        FontKind::TrueType => d.font_file2(slot.file),
        FontKind::CffOpenType => d.font_file3(slot.file),
    };
}

fn write_widths(cid: &mut pdf_writer::writers::CidFont<'_>, face: &Face<'_>, gids: &BTreeSet<u16>) {
    if gids.is_empty() {
        return;
    }
    let units = face.units_per_em() as f32;
    if units == 0.0 {
        return;
    }
    let ids: Vec<u16> = gids.iter().copied().collect();
    let mut w = cid.widths();
    let mut i = 0;
    while i < ids.len() {
        let start = ids[i];
        let mut widths = vec![advance_1000(face, start, units)];
        let mut j = i + 1;
        while j < ids.len() && ids[j] == ids[j - 1] + 1 {
            widths.push(advance_1000(face, ids[j], units));
            j += 1;
        }
        w.consecutive(start, widths);
        i = j;
    }
}

fn advance_1000(face: &Face<'_>, gid: u16, units: f32) -> f32 {
    face.glyph_hor_advance(GlyphId(gid)).unwrap_or(0) as f32 * 1000.0 / units
}

fn write_font_file(pdf: &mut Pdf, slot: &FontSlot, bytes: &[u8]) {
    let compressed = compress_to_vec_zlib(bytes, CompressionLevel::DefaultLevel as u8);
    let mut stream = pdf.stream(slot.file, &compressed);
    stream.filter(Filter::FlateDecode);
    match slot.kind {
        FontKind::TrueType => {
            stream.pair(Name(b"Length1"), bytes.len() as i32);
        }
        FontKind::CffOpenType => {
            stream.pair(Name(b"Subtype"), Name(b"OpenType"));
        }
    }
}

fn write_cmap(pdf: &mut Pdf, slot: &FontSlot, gid_map: &BTreeMap<u16, String>) {
    let mut cmap = UnicodeCmap::new(Name(slot.cmap_name.as_bytes()), SYS);
    for (gid, uni) in gid_map {
        if uni.is_empty() {
            continue;
        }
        cmap.pair_with_multiple(*gid, uni.chars());
    }
    let data = cmap.finish();
    let mut c = pdf.cmap(slot.cmap, data.as_ref());
    c.name(Name(slot.cmap_name.as_bytes()));
    c.system_info(SYS);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ids::Alloc;

    #[test]
    fn reserve_skips_license_sidecars() {
        let mut fonts = BTreeMap::new();
        fonts.insert(
            "assets/fonts/licenses/Roboto-Apache.txt".into(),
            b"Apache-2.0".to_vec(),
        );
        fonts.insert("assets/fonts/z.ttf".into(), vec![0, 1, 0, 0]);
        let set = reserve(&mut Alloc::new(), &fonts);
        assert_eq!(set.slots.len(), 1);
        assert_eq!(set.slots[0].path, "assets/fonts/z.ttf");
        assert!(!set
            .family_to_font
            .contains_key("assets/fonts/licenses/Roboto-Apache.txt"));
        assert!(set.family_to_font.contains_key("z"));
        assert_eq!(
            set.family_to_font.get("default").map(String::as_str),
            Some("F0")
        );
    }
}

fn pdf_font_name(path: &str, i: usize) -> String {
    let stem = Path::new(path)
        .file_stem()
        .map(|s| s.to_string_lossy().into_owned())
        .unwrap_or_else(|| format!("F{i}"));
    let safe: String = stem
        .chars()
        .map(|c| if c.is_ascii_alphanumeric() { c } else { '-' })
        .collect();
    format!("K2F+{safe}")
}
