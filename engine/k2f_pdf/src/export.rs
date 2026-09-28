use k2f_paint::{load_faces, OpenedDocument};
use miniz_oxide::deflate::{compress_to_vec_zlib, CompressionLevel};
use pdf_writer::{Filter, Finish, Name, Pdf, Rect, Ref, TextStr};
use std::collections::{BTreeMap, BTreeSet, HashSet};

use crate::acroform;
use crate::error::PdfError;
use crate::ids::Alloc;
use crate::image::{collect_image_srcs, embed_images, ImageRes};
use crate::ops::paint_page;
use crate::select::{self, FontSet};
use crate::source::{draw_source_caption, source_line};
use crate::stamp::{compose_page, slices_for_page, PdfScale};
use crate::verify_page;

/// PDF export settings. By default exports only the lock pages (no captions or verify page).
#[derive(Clone, Copy, Debug)]
pub struct PdfExportOptions {
    pub scale: PdfScale,
    /// When true, each page gets a source caption and a final integrity verification page.
    pub trust_pack: bool,
    /// When true, paint field glyphs into page content and omit AcroForm widgets.
    pub flatten: bool,
}

impl PdfExportOptions {
    pub const fn new(scale: PdfScale) -> Self {
        Self {
            scale,
            trust_pack: false,
            flatten: false,
        }
    }

    pub const fn with_trust_pack(mut self) -> Self {
        self.trust_pack = true;
        self
    }

    pub const fn with_flatten(mut self) -> Self {
        self.flatten = true;
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
    export_bytes_with(package_bytes, PdfExportOptions::new(scale))
}

pub fn export_bytes_with(
    package_bytes: &[u8],
    options: PdfExportOptions,
) -> Result<Vec<u8>, PdfError> {
    export_opened(&OpenedDocument::open(package_bytes)?, options)
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
    let fields = doc.form_fields();
    if options.trust_pack && !options.flatten && !fields.is_empty() {
        return Err(PdfError::FillableExclusive);
    }
    let fillable = !options.flatten && !fields.is_empty();
    let skip_text: HashSet<String> = if fillable {
        fields.iter().map(|f| f.id.clone()).collect()
    } else {
        HashSet::new()
    };

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
    let acro = fillable.then(|| acroform::alloc(&mut alloc, &fields));

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

    {
        let mut cat = pdf.catalog(catalog_id);
        cat.pages(pages_id);
        if let Some(acro) = acro.as_ref() {
            acroform::write_catalog_form(&mut cat, acro);
        }
    }
    pdf.pages(pages_id)
        .kids(page_ids.iter().copied())
        .count(total as i32);

    let paint_scale = options.scale.as_f32();
    let mut gid_maps = vec![BTreeMap::new(); fonts.slots.len()];
    let mut pages: Vec<BuiltPage> = Vec::with_capacity(total);
    for i in 0..n {
        let page = &lock.geometry.pages[i];
        let plan = lock
            .render_plan
            .pages
            .iter()
            .find(|p| p.index == page.index)
            .ok_or(k2f_paint::PaintError::MissingRenderPlan(page.index))?;
        let slices = slices_for_page(&plan.ops, page.width, page.height)?;
        let (mut page_draw, page_stamps) = if slices.is_empty() {
            (
                paint_page(lock, i, &faces, &images, &skip_text)?,
                Vec::new(),
            )
        } else {
            compose_page(
                doc,
                i,
                &faces,
                &images,
                &skip_text,
                &mut pdf,
                &mut alloc,
                paint_scale,
                &slices,
            )?
        };
        if options.trust_pack {
            draw_source_caption(&mut page_draw, &faces, hash);
        }
        let mut glyphs = select::collect_page(doc, i, &faces, &skip_text)?;
        if options.trust_pack {
            glyphs.extend(select::collect_caption(page_draw.page_h, &faces, hash));
        }
        select::merge_gid_maps(&fonts, &glyphs, &mut gid_maps);
        select::emit_page(&mut page_draw, &glyphs, &fonts);
        let fill_alphas = page_draw.fill_alphas.clone();
        pages.push(BuiltPage {
            w: page_draw.page_w,
            h: page_draw.page_h,
            raw: page_draw.finish(),
            stamps: page_stamps,
            fill_alphas,
        });
    }
    if options.trust_pack {
        let verify = verify_page::build_page(doc, &faces, &fonts, &mut gid_maps)?;
        pages.push(BuiltPage {
            w: verify.0,
            h: verify.1,
            raw: verify.2,
            stamps: Vec::new(),
            fill_alphas: BTreeSet::new(),
        });
    }

    select::write(&mut pdf, &fonts, doc.fonts(), &faces, &gid_maps);

    let page_heights: Vec<f64> = lock
        .geometry
        .pages
        .iter()
        .map(|p| p.height.as_f64_pt())
        .collect();
    let annots = match acro.as_ref() {
        Some(acro) => acroform::write_widgets(&mut pdf, acro, &fields, &page_ids, &page_heights),
        None => vec![Vec::new(); total],
    };

    for (i, page) in pages.into_iter().enumerate() {
        write_page(
            &mut pdf,
            &mut alloc,
            pages_id,
            page_ids[i],
            content_ids[i],
            page.w,
            page.h,
            &page.raw,
            &images,
            &page.stamps,
            &fonts,
            annots.get(i).map(Vec::as_slice).unwrap_or(&[]),
            &page.fill_alphas,
        );
    }

    Ok(pdf.finish())
}

struct BuiltPage {
    w: f64,
    h: f64,
    raw: Vec<u8>,
    stamps: Vec<ImageRes>,
    fill_alphas: BTreeSet<u8>,
}

#[allow(clippy::too_many_arguments)]
fn write_page(
    pdf: &mut Pdf,
    alloc: &mut Alloc,
    pages_id: Ref,
    page_id: Ref,
    content_id: Ref,
    width_pt: f64,
    height_pt: f64,
    raw: &[u8],
    images: &BTreeMap<String, ImageRes>,
    stamps: &[ImageRes],
    fonts: &FontSet,
    annots: &[Ref],
    fill_alphas: &BTreeSet<u8>,
) {
    let mut graphics = Vec::with_capacity(fill_alphas.len());
    for alpha in fill_alphas {
        let id = alloc.bump();
        let ca = f32::from(*alpha) / 255.0;
        pdf.ext_graphics(id)
            .non_stroking_alpha(ca)
            .stroking_alpha(ca);
        graphics.push((format!("ca{alpha}"), id));
    }
    {
        let mut p = pdf.page(page_id);
        p.parent(pages_id);
        p.media_box(Rect::new(0.0, 0.0, width_pt as f32, height_pt as f32));
        p.contents(content_id);
        {
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
            if !graphics.is_empty() {
                let mut gs = res.ext_g_states();
                for (name, id) in &graphics {
                    gs.pair(Name(name.as_bytes()), *id);
                }
            }
            if !fonts.slots.is_empty() {
                let mut fo = res.fonts();
                for slot in &fonts.slots {
                    fo.pair(Name(slot.name.as_bytes()), slot.type0);
                }
            }
        }
        if !annots.is_empty() {
            p.annotations(annots.iter().copied());
        }
    }
    let compressed = compress_to_vec_zlib(raw, CompressionLevel::DefaultLevel as u8);
    pdf.stream(content_id, &compressed)
        .filter(Filter::FlateDecode);
}
