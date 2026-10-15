use image::{DynamicImage, ImageBuffer, ImageFormat, RgbaImage};
use k2f_core::{
    looks_like_svg, svg_bytes_contain_text_element, ImageFit, Rect, SVG_TEXT_FORBIDDEN_MSG,
};
use tiny_skia::Pixmap;

use super::letterbox::{cover_src, letterbox_dest};
use crate::error::PaintError;

/// Widest or tallest SVG raster we will allocate. A landscape page at the viewer
/// cap (4 px/pt) stays under this; larger requests fail instead of exhausting WASM.
pub const SVG_RASTER_MAX_PX: u32 = 8192;

/// Pixels per document point when an SVG is embedded in PDF, DOCX, PPTX, or IDML.
/// Matches the viewer paint-scale cap so export hairlines sit in the same class
/// as on-screen paint. Official PNG comparison stays at scale 2; that gate is
/// not the export resolution.
pub const SVG_EXPORT_PX_PER_PT: f32 = 4.0;

pub fn decode_raster(bytes: &[u8]) -> Result<DynamicImage, PaintError> {
    match image::guess_format(bytes) {
        Ok(ImageFormat::Png) | Ok(ImageFormat::WebP) | Ok(ImageFormat::Jpeg) => {
            image::load_from_memory(bytes).map_err(|e| PaintError::Image(e.to_string()))
        }
        Ok(other) => Err(PaintError::Image(format!(
            "unsupported image format {other:?}; embed PNG, JPEG, WebP, or SVG"
        ))),
        Err(_) if looks_like_svg(bytes) => decode_svg(bytes),
        Err(e) => Err(PaintError::Image(e.to_string())),
    }
}

/// Rasterize SVG at its own pixel size with a fixed-version resvg.
/// Built without the `text` feature so host fonts never affect canonical output.
///
/// Paint and export do not use this size as the final bitmap. Fit uses
/// [`svg_intrinsic_px`]; the draw uses a destination-sized render. This decode
/// is the 1:1 file-header preview, and it refuses a side above [`SVG_RASTER_MAX_PX`].
pub fn decode_svg(bytes: &[u8]) -> Result<DynamicImage, PaintError> {
    let (w, h) = svg_intrinsic_px(bytes)?;
    ensure_svg_px_limit(w, h)?;
    let tree = parse_svg(bytes)?;
    render_tree(&tree, w, h, resvg::tiny_skia::Transform::default())
}

/// Viewport size in pixels: the SVG `width`/`height` rounded up, matching
/// [`decode_svg`]. Fit math uses this so a scaled raster keeps the same aspect
/// as the intrinsic decode.
pub fn svg_intrinsic_px(bytes: &[u8]) -> Result<(u32, u32), PaintError> {
    let tree = parse_svg(bytes)?;
    let size = tree.size();
    Ok((
        size.width().ceil().max(1.0) as u32,
        size.height().ceil().max(1.0) as u32,
    ))
}

/// Rasterize the whole SVG into `dest_w`×`dest_h` with one scale transform.
/// Callers pass a size that preserves aspect (the contain letterbox). Does not
/// nearest-neighbor upscale a smaller bitmap.
pub fn render_svg_uniform(
    bytes: &[u8],
    dest_w: u32,
    dest_h: u32,
) -> Result<DynamicImage, PaintError> {
    premul_pixmap_to_dynamic(&render_svg_uniform_pixmap(bytes, dest_w, dest_h)?)
}

pub(crate) fn render_svg_uniform_pixmap(
    bytes: &[u8],
    dest_w: u32,
    dest_h: u32,
) -> Result<Pixmap, PaintError> {
    ensure_svg_px_limit(dest_w, dest_h)?;
    let tree = parse_svg(bytes)?;
    let vw = tree.size().width();
    let vh = tree.size().height();
    let transform = resvg::tiny_skia::Transform::from_scale(dest_w as f32 / vw, dest_h as f32 / vh);
    render_tree_pixmap(&tree, dest_w, dest_h, transform)
}

/// Cover window: render only the center-crop that fills `box_w`×`box_h`.
pub fn render_svg_cover(bytes: &[u8], box_w: u32, box_h: u32) -> Result<DynamicImage, PaintError> {
    premul_pixmap_to_dynamic(&render_svg_cover_pixmap(bytes, box_w, box_h)?)
}

pub(crate) fn render_svg_cover_pixmap(
    bytes: &[u8],
    box_w: u32,
    box_h: u32,
) -> Result<Pixmap, PaintError> {
    ensure_svg_px_limit(box_w, box_h)?;
    let (iw, ih) = svg_intrinsic_px(bytes)?;
    let (sx, sy, sw, sh) = cover_src(iw, ih, box_w, box_h).ok_or(PaintError::Pixmap)?;
    let scale_x = box_w as f32 / sw as f32;
    let scale_y = box_h as f32 / sh as f32;
    let transform = resvg::tiny_skia::Transform::from_scale(scale_x, scale_y)
        .post_translate(-(sx as f32) * scale_x, -(sy as f32) * scale_y);
    let tree = parse_svg(bytes)?;
    render_tree_pixmap(&tree, box_w, box_h, transform)
}

/// Pixel size of an SVG embedded in PDF / DOCX / PPTX / IDML.
///
/// Contain returns the letterboxed image (the whole SVG, scaled so the longer
/// side fits the box at [`SVG_EXPORT_PX_PER_PT`]). Cover returns the box itself:
/// the PNG is already the center crop that fills that box.
pub fn svg_export_raster_px(
    bytes: &[u8],
    rect: &Rect,
    fit: ImageFit,
) -> Result<(u32, u32), PaintError> {
    let box_w = pt_to_export_px(rect.width);
    let box_h = pt_to_export_px(rect.height);
    let (w, h) = match fit {
        ImageFit::Contain => {
            let (iw, ih) = svg_intrinsic_px(bytes)?;
            let (_, _, dw, dh) = letterbox_dest(iw, ih, box_w, box_h).ok_or(PaintError::Pixmap)?;
            (dw, dh)
        }
        ImageFit::Cover => (box_w, box_h),
    };
    ensure_svg_px_limit(w, h)?;
    Ok((w, h))
}

/// SVG → PNG at the export pixel size for `rect` and `fit`. Bitmaps do not use this.
pub fn svg_png_for_export(bytes: &[u8], rect: &Rect, fit: ImageFit) -> Result<Vec<u8>, PaintError> {
    let img = raster_svg_for_export(bytes, rect, fit)?;
    let mut out = std::io::Cursor::new(Vec::new());
    img.write_to(&mut out, ImageFormat::Png)
        .map_err(|e| PaintError::Image(format!("png encode: {e}")))?;
    Ok(out.into_inner())
}

/// Decode for PDF embedding. SVG is rasterized at export density; PNG/JPEG/WebP
/// stay at their file dimensions (`rect` is ignored for those).
pub fn raster_for_embed(
    bytes: &[u8],
    rect: &Rect,
    fit: ImageFit,
) -> Result<DynamicImage, PaintError> {
    if looks_like_svg(bytes) {
        raster_svg_for_export(bytes, rect, fit)
    } else {
        decode_raster(bytes)
    }
}

fn raster_svg_for_export(
    bytes: &[u8],
    rect: &Rect,
    fit: ImageFit,
) -> Result<DynamicImage, PaintError> {
    let (w, h) = svg_export_raster_px(bytes, rect, fit)?;
    match fit {
        ImageFit::Cover => render_svg_cover(bytes, w, h),
        ImageFit::Contain => render_svg_uniform(bytes, w, h),
    }
}

fn pt_to_export_px(pt: k2f_core::Pt) -> u32 {
    let v = pt.as_f64_pt() * (SVG_EXPORT_PX_PER_PT as f64);
    v.round().max(1.0) as u32
}

fn ensure_svg_px_limit(w: u32, h: u32) -> Result<(), PaintError> {
    if w == 0 || h == 0 || w > SVG_RASTER_MAX_PX || h > SVG_RASTER_MAX_PX {
        return Err(PaintError::Image(format!(
            "SVG raster {w}x{h} exceeds the {SVG_RASTER_MAX_PX}px limit on a side"
        )));
    }
    Ok(())
}

fn parse_svg(bytes: &[u8]) -> Result<resvg::usvg::Tree, PaintError> {
    reject_svg_text(bytes)?;
    let opt = resvg::usvg::Options::default();
    resvg::usvg::Tree::from_data(bytes, &opt)
        .map_err(|e| PaintError::Image(format!("SVG parse failed: {e}")))
}

fn render_tree(
    tree: &resvg::usvg::Tree,
    w: u32,
    h: u32,
    transform: resvg::tiny_skia::Transform,
) -> Result<DynamicImage, PaintError> {
    premul_pixmap_to_dynamic(&render_tree_pixmap(tree, w, h, transform)?)
}

fn render_tree_pixmap(
    tree: &resvg::usvg::Tree,
    w: u32,
    h: u32,
    transform: resvg::tiny_skia::Transform,
) -> Result<Pixmap, PaintError> {
    let mut pixmap = Pixmap::new(w, h).ok_or(PaintError::Pixmap)?;
    resvg::render(tree, transform, &mut pixmap.as_mut());
    Ok(pixmap)
}

/// Text-bearing SVG elements need host fonts. Fail closed instead of dropping glyphs.
pub(crate) fn reject_svg_text(bytes: &[u8]) -> Result<(), PaintError> {
    if svg_bytes_contain_text_element(bytes) {
        return Err(PaintError::Image(format!(
            "SVG_TEXT: {SVG_TEXT_FORBIDDEN_MSG}"
        )));
    }
    Ok(())
}

fn premul_pixmap_to_dynamic(pixmap: &Pixmap) -> Result<DynamicImage, PaintError> {
    let w = pixmap.width();
    let h = pixmap.height();
    let mut rgba = Vec::with_capacity((w * h * 4) as usize);
    for px in pixmap.data().chunks_exact(4) {
        let a = px[3] as u16;
        if a == 0 {
            rgba.extend_from_slice(&[0, 0, 0, 0]);
        } else if a == 255 {
            rgba.extend_from_slice(px);
        } else {
            // Un-premultiply for image crate Rgba.
            let r = ((px[0] as u16 * 255 + a / 2) / a) as u8;
            let g = ((px[1] as u16 * 255 + a / 2) / a) as u8;
            let b = ((px[2] as u16 * 255 + a / 2) / a) as u8;
            rgba.extend_from_slice(&[r, g, b, a as u8]);
        }
    }
    let img: RgbaImage = ImageBuffer::from_raw(w, h, rgba).ok_or(PaintError::Pixmap)?;
    Ok(DynamicImage::ImageRgba8(img))
}

pub fn raster_to_pixmap(
    img: &DynamicImage,
    dest_w: u32,
    dest_h: u32,
) -> Result<Pixmap, PaintError> {
    let resized = img.resize_exact(dest_w, dest_h, image::imageops::FilterType::Nearest);
    let rgba = resized.to_rgba8();
    let mut premul = rgba.into_raw();
    for px in premul.chunks_exact_mut(4) {
        let a = px[3] as u16;
        px[0] = ((px[0] as u16 * a + 127) / 255) as u8;
        px[1] = ((px[1] as u16 * a + 127) / 255) as u8;
        px[2] = ((px[2] as u16 * a + 127) / 255) as u8;
    }
    Pixmap::from_vec(
        premul,
        tiny_skia::IntSize::from_wh(dest_w, dest_h).ok_or(PaintError::Pixmap)?,
    )
    .ok_or(PaintError::Pixmap)
}

pub fn letterbox_pixmap(
    img: &DynamicImage,
    box_w: u32,
    box_h: u32,
) -> Result<(u32, u32, Pixmap), PaintError> {
    let (x, y, w, h) =
        letterbox_dest(img.width(), img.height(), box_w, box_h).ok_or(PaintError::Pixmap)?;
    let pixmap = raster_to_pixmap(img, w, h)?;
    Ok((x, y, pixmap))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn svg_without_text_decodes() {
        let svg = br##"<svg xmlns="http://www.w3.org/2000/svg" width="4" height="4"><rect width="4" height="4" fill="#00f"/></svg>"##;
        decode_svg(svg).unwrap();
    }

    #[test]
    fn svg_with_text_fails_closed() {
        let svg = br#"<svg xmlns="http://www.w3.org/2000/svg" width="4" height="4"><text x="1" y="2">A</text></svg>"#;
        let err = decode_svg(svg).unwrap_err().to_string();
        assert!(err.contains("SVG_TEXT"), "{err}");
        assert!(err.contains("<text>"), "{err}");
        assert!(err.contains("<path>"), "{err}");
    }

    #[test]
    fn svg_comment_mentioning_text_decodes() {
        let svg = br##"<svg xmlns="http://www.w3.org/2000/svg" width="4" height="4"><!-- <text> converted to path --><rect width="4" height="4" fill="#00f"/></svg>"##;
        decode_svg(svg).unwrap();
    }

    #[test]
    fn svg_with_textpath_fails_closed() {
        let svg = br##"<svg xmlns="http://www.w3.org/2000/svg" width="4" height="4"><textPath href="#p">A</textPath></svg>"##;
        let err = decode_svg(svg).unwrap_err().to_string();
        assert!(err.contains("SVG_TEXT"), "{err}");
        assert!(err.contains("<text>"), "{err}");
        assert!(err.contains("<path>"), "{err}");
    }

    /// Nearest-neighbor upscale of a 1px antialiased stroke replicates that soft
    /// column. Destination-resolution rendering must not do this: a 1-user-unit
    /// stroke at 4× is about four device pixels of solid ink.
    #[test]
    fn nearest_upscale_replicates_soft_stroke_but_dest_render_does_not() {
        let svg = br##"<svg xmlns="http://www.w3.org/2000/svg" width="20" height="20" viewBox="0 0 20 20"><line x1="10" y1="0" x2="10" y2="20" stroke="#000" stroke-width="1"/></svg>"##;
        let low = decode_svg(svg).unwrap();
        assert_eq!((low.width(), low.height()), (20, 20));
        let soft = opaque_run_at_mid_row(&low, 200);
        assert!(
            soft <= 2,
            "intrinsic 1px stroke should be about one device pixel, got {soft}"
        );
        let nearest = low.resize_exact(80, 80, image::imageops::FilterType::Nearest);
        let copied = opaque_run_at_mid_row(&nearest, 200);
        let sharp = render_svg_uniform(svg, 80, 80).unwrap();
        let solid = opaque_run_at_mid_row(&sharp, 200);
        assert!(
            solid + 1 >= 4 && solid <= 6,
            "4× dest stroke should be about 4px of solid ink, got {solid}"
        );
        assert!(
            solid > copied.max(soft.saturating_mul(4).saturating_sub(1)) || sharp.to_rgba8() != nearest.to_rgba8(),
            "dest render must not be a nearest copy of the intrinsic raster (solid={solid}, copied={copied})"
        );
        assert_ne!(
            sharp.to_rgba8().as_raw(),
            nearest.to_rgba8().as_raw(),
            "4× vector raster must differ from nearest-neighbor upscale"
        );
    }

    #[test]
    fn export_px_is_box_pt_times_four_not_svg_header() {
        use k2f_core::Pt;
        let svg = br##"<svg xmlns="http://www.w3.org/2000/svg" width="10" height="10"><rect width="10" height="10" fill="#000"/></svg>"##;
        let rect = Rect {
            x: Pt(0),
            y: Pt(0),
            width: Pt(10_000),
            height: Pt(10_000),
        };
        assert_eq!(
            svg_export_raster_px(svg, &rect, ImageFit::Contain).unwrap(),
            (40, 40)
        );
        let png = svg_png_for_export(svg, &rect, ImageFit::Contain).unwrap();
        let img = image::load_from_memory(&png).unwrap();
        assert_eq!((img.width(), img.height()), (40, 40));
    }

    #[test]
    fn decode_svg_over_limit_fails() {
        let svg = br##"<svg xmlns="http://www.w3.org/2000/svg" width="9000" height="10"><rect width="9000" height="10" fill="#000"/></svg>"##;
        let err = decode_svg(svg).unwrap_err().to_string();
        assert!(err.contains("8192"), "{err}");
    }

    #[test]
    fn cover_export_is_the_box_not_the_full_svg() {
        use k2f_core::Pt;
        let svg = br##"<svg xmlns="http://www.w3.org/2000/svg" width="20" height="10"><rect width="20" height="10" fill="#000"/></svg>"##;
        let rect = Rect {
            x: Pt(0),
            y: Pt(0),
            width: Pt(10_000),
            height: Pt(10_000),
        };
        assert_eq!(
            svg_export_raster_px(svg, &rect, ImageFit::Cover).unwrap(),
            (40, 40)
        );
        let png = svg_png_for_export(svg, &rect, ImageFit::Cover).unwrap();
        let img = image::load_from_memory(&png).unwrap();
        assert_eq!((img.width(), img.height()), (40, 40));
    }

    #[test]
    fn real_mark_svg_exports_at_box_times_four() {
        use k2f_core::Pt;
        let path = concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../tests/fixtures/cases/image_svg_basic/assets/images/mark.svg"
        );
        let bytes = std::fs::read(path).expect("image_svg_basic mark.svg");
        let rect = Rect {
            x: Pt(0),
            y: Pt(0),
            width: Pt(40_000),
            height: Pt(40_000),
        };
        assert_eq!(
            svg_export_raster_px(&bytes, &rect, ImageFit::Contain).unwrap(),
            (160, 160)
        );
        let png = svg_png_for_export(&bytes, &rect, ImageFit::Contain).unwrap();
        let img = image::load_from_memory(&png).unwrap();
        assert_eq!((img.width(), img.height()), (160, 160));
    }

    #[test]
    fn export_raster_over_limit_fails() {
        use k2f_core::Pt;
        let svg = br##"<svg xmlns="http://www.w3.org/2000/svg" width="10" height="10"><rect width="10" height="10" fill="#000"/></svg>"##;
        let rect = Rect {
            x: Pt(0),
            y: Pt(0),
            width: Pt(3_000_000),
            height: Pt(3_000_000),
        };
        let err = svg_export_raster_px(svg, &rect, ImageFit::Contain)
            .unwrap_err()
            .to_string();
        assert!(err.contains("8192"), "{err}");
    }

    #[test]
    fn svg_with_foreign_object_fails_closed() {
        let svg = br##"<svg xmlns="http://www.w3.org/2000/svg"><foreignObject width="4" height="4">A</foreignObject></svg>"##;
        let err = decode_svg(svg).unwrap_err().to_string();
        assert!(err.contains("SVG_TEXT"), "{err}");
        assert!(err.contains("foreignObject"), "{err}");
    }

    fn opaque_run_at_mid_row(img: &DynamicImage, min_alpha: u8) -> u32 {
        let rgba = img.to_rgba8();
        let y = rgba.height() / 2;
        let mut best = 0u32;
        let mut run = 0u32;
        for x in 0..rgba.width() {
            if rgba.get_pixel(x, y)[3] > min_alpha {
                run += 1;
                best = best.max(run);
            } else {
                run = 0;
            }
        }
        best
    }
}
