use k2f_paint::{load_faces, OpenedDocument};
use miniz_oxide::deflate::{compress_to_vec_zlib, CompressionLevel};
use pdf_writer::{Filter, Finish, Name, Pdf, Rect, TextStr};
use std::collections::BTreeMap;

use crate::error::PdfError;
use crate::ids::Alloc;
use crate::image::{collect_image_srcs, embed_images, embed_rgb, ImageRes};
use crate::ops::paint_page;
use crate::select::{self, FontSet};
use crate::source::{draw_source_caption, source_line};
use crate::stamp::{draw_full_page_stamp, page_needs_stamp, PdfScale};
use crate::verify_page;

/// PDF export settings. By default exports only the lock pages (no captions or verify page).
#[derive(Clone, Copy, Debug)]
pub struct PdfExportOptions {
    pub scale: PdfScale,
    /// When true, each page gets a source caption and a final integrity verification page.
    pub trust_pack: bool,
}

impl PdfExportOptions {
    pub const fn new(scale: PdfScale) -> Self {
        Self {
            scale,
            trust_pack: false,
        }
    }

    pub const fn with_trust_pack(mut self) -> Self {
        self.trust_pack = true;
        self
    }
}

impl From<PdfScale> for PdfExportOptions {
    fn from(scale: PdfScale) -> Self {
        Self::new(scale)
    }
}

pub fn export_bytes(package_bytes: &[u8]) -> Result<Vec<u8>, PdfError> {
    export_bytes_at(package_bytes, PdfScale::DEFAULT)
}

pub fn export_bytes_at(package_bytes: &[u8], scale: PdfScale) -> Result<Vec<u8>, PdfError> {
    export_opened(&OpenedDocument::open(package_bytes)?, scale)
}

pub fn export_opened(
    doc: &OpenedDocument,
    options: impl Into<PdfExportOptions>,
) -> Result<Vec<u8>, PdfError> {
    let options = options.into();
    let lock = doc.lock().ok_or(PdfError::Unlocked)?;
    if doc.fonts().is_empty() {
        return Err(PdfError::Write("package has no embedded font".into()));
    }
    let faces = load_faces(doc.fonts())?;
    let mut alloc = Alloc::new();
    let catalog_id = alloc.bump();
    let pages_id = alloc.bump();
    let info_id = alloc.bump();

    let mut pdf = Pdf::new();
    pdf.set_version(1, 7);

    let srcs = collect_image_srcs(lock);
    let images = embed_images(&mut pdf, &mut alloc, doc.assets(), &srcs)?;
    let fonts = select::alloc_fonts(&mut alloc, doc.fonts());

    let n = lock.geometry.pages.len();
    let total = if options.trust_pack { n + 1 } else { n };
    let mut page_ids = Vec::with_capacity(total);
    let mut content_ids = Vec::with_capacity(total);
    for _ in 0..total {
        page_ids.push(alloc.bump());
        content_ids.push(alloc.bump());
    }

    let hash = lock.appearance_hash.as_str();
    {
        let mut info = pdf.document_info(info_id);
        info.title(TextStr(doc.title()));
        if options.trust_pack {
            info.subject(TextStr(&source_line(hash)));
        }
        info.creator(TextStr("K2F"));
        info.producer(TextStr("K2F PDF bridge (lock paint plan)"));
        info.finish();
    }

    pdf.catalog(catalog_id).pages(pages_id);
    pdf.pages(pages_id)
        .kids(page_ids.iter().copied())
        .count(total as i32);

    let paint_scale = options.scale.as_f32();
    let mut gid_maps = vec![BTreeMap::new(); fonts.slots.len()];
    let mut pages: Vec<(f64, f64, Vec<u8>, Vec<ImageRes>)> = Vec::with_capacity(total);
    for i in 0..n {
        let page = &lock.geometry.pages[i];
        let plan = lock
            .render_plan
            .pages
            .iter()
            .find(|p| p.index == page.index)
            .ok_or(k2f_paint::PaintError::MissingRenderPlan(page.index))?;
        let stamp_ops = page_needs_stamp(&plan.ops);
        let mut page_stamps = Vec::new();
        let mut page_draw = if stamp_ops {
            let (w_px, h_px, rgb) = doc.render_page_rgb(i, paint_scale)?;
            let stamp = embed_rgb(
                &mut pdf,
                &mut alloc,
                w_px,
                h_px,
                &rgb,
                &format!("St{i}"),
            );
            let mut out = crate::draw::PageDraw::new(page.width.as_f64_pt(), page.height.as_f64_pt());
            draw_full_page_stamp(&mut out, &stamp);
            page_stamps.push(stamp);
            out
        } else {
            paint_page(lock, i, &faces, &images)?
        };
        if options.trust_pack {
            draw_source_caption(&mut page_draw, &faces, hash);
        }
        let mut glyphs = select::collect_page(doc, i, &faces)?;
        if options.trust_pack {
            glyphs.extend(select::collect_caption(page_draw.page_h, &faces, hash));
        }
        select::merge_gid_maps(&fonts, &glyphs, &mut gid_maps);
        select::emit_page(&mut page_draw, &glyphs, &fonts);
        pages.push((
            page_draw.page_w,
            page_draw.page_h,
            page_draw.finish(),
            page_stamps,
        ));
    }
    if options.trust_pack {
        let verify = verify_page::build_page(doc, &faces, &fonts, &mut gid_maps)?;
        pages.push((verify.0, verify.1, verify.2, Vec::new()));
    }

    select::write(&mut pdf, &fonts, doc.fonts(), &faces, &gid_maps);

    for (i, (w, h, raw, stamps)) in pages.into_iter().enumerate() {
        write_page(
            &mut pdf,
            pages_id,
            page_ids[i],
            content_ids[i],
            w,
            h,
            &raw,
            &images,
            &stamps,
            &fonts,
        );
    }

    Ok(pdf.finish())
}

#[allow(clippy::too_many_arguments)]
fn write_page(
    pdf: &mut Pdf,
    pages_id: pdf_writer::Ref,
    page_id: pdf_writer::Ref,
    content_id: pdf_writer::Ref,
    width_pt: f64,
    height_pt: f64,
    raw: &[u8],
    images: &BTreeMap<String, ImageRes>,
    stamps: &[ImageRes],
    fonts: &FontSet,
) {
    {
        let mut p = pdf.page(page_id);
        p.parent(pages_id);
        p.media_box(Rect::new(0.0, 0.0, width_pt as f32, height_pt as f32));
        p.contents(content_id);
        let mut res = p.resources();
        if !images.is_empty() || !stamps.is_empty() {
            let mut xo = res.x_objects();
            for img in images.values() {
                xo.pair(Name(img.name.as_bytes()), img.id);
            }
            for stamp in stamps {
                xo.pair(Name(stamp.name.as_bytes()), stamp.id);
            }
        }
        if !fonts.slots.is_empty() {
            let mut fo = res.fonts();
            for slot in &fonts.slots {
                fo.pair(Name(slot.name.as_bytes()), slot.type0);
            }
        }
    }
    let compressed = compress_to_vec_zlib(raw, CompressionLevel::DefaultLevel as u8);
    pdf.stream(content_id, &compressed)
        .filter(Filter::FlateDecode);
}
