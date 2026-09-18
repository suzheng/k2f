use crate::coord::{millipt_to_emu, pt_to_emu};
use crate::ir::PictureBox;
use crate::PptxError;
use k2f_core::{ImageFit, Rect};
use k2f_paint::{cover_src_rect_100000, decode_raster, letterbox_rect, lookup_image};
use std::collections::BTreeMap;
use std::io::Cursor;

pub(crate) fn picture_from_draw(
    node_id: &str,
    rect: &Rect,
    src: &str,
    assets: &BTreeMap<String, Vec<u8>>,
    media_index: u32,
    fit: ImageFit,
    corner_radius_pt: Option<i64>,
) -> Result<PictureBox, PptxError> {
    let bytes = lookup_image(assets, src)
        .ok_or_else(|| PptxError::Write(format!("missing image '{src}'")))?;
    let (dest, src_crop) = dest_and_crop(rect, bytes, fit);
    let (ext, payload) = encode_media(bytes)?;
    Ok(PictureBox {
        node_id: node_id.to_string(),
        x_emu: pt_to_emu(dest.x),
        y_emu: pt_to_emu(dest.y),
        cx_emu: pt_to_emu(dest.width),
        cy_emu: pt_to_emu(dest.height),
        media_name: format!("image{media_index}.{ext}"),
        bytes: payload,
        src_l: src_crop.0,
        src_t: src_crop.1,
        src_r: src_crop.2,
        src_b: src_crop.3,
        corner_emu: millipt_to_emu(corner_radius_pt.unwrap_or(0).max(0)),
    })
}

fn dest_and_crop(rect: &Rect, bytes: &[u8], fit: ImageFit) -> (Rect, (i64, i64, i64, i64)) {
    let Some(img) = decode_raster(bytes).ok() else {
        return (rect.clone(), (0, 0, 0, 0));
    };
    match fit {
        ImageFit::Cover => {
            let bw = rect.width.0.max(1) as u32;
            let bh = rect.height.0.max(1) as u32;
            let crop =
                cover_src_rect_100000(img.width(), img.height(), bw, bh).unwrap_or((0, 0, 0, 0));
            (rect.clone(), crop)
        }
        ImageFit::Contain => (
            letterbox_rect(img.width(), img.height(), rect).unwrap_or_else(|| rect.clone()),
            (0, 0, 0, 0),
        ),
    }
}

fn encode_media(bytes: &[u8]) -> Result<(&'static str, Vec<u8>), PptxError> {
    if is_png(bytes) {
        return Ok(("png", bytes.to_vec()));
    }
    if is_jpeg(bytes) {
        return Ok(("jpg", bytes.to_vec()));
    }
    // SVG (and WebP / other decode_raster formats) → PNG. Office hosts often
    // leave raw `image/svg+xml` blank; paint/PDF already rasterize via resvg.
    if looks_like_svg(bytes) || is_webp(bytes) {
        return raster_to_png(bytes);
    }
    match raster_to_png(bytes) {
        Ok(png) => Ok(png),
        Err(_) => Err(PptxError::Write(
            "unsupported image bytes (need png/jpg/webp/svg)".into(),
        )),
    }
}

fn raster_to_png(bytes: &[u8]) -> Result<(&'static str, Vec<u8>), PptxError> {
    let img = decode_raster(bytes).map_err(|e| PptxError::Write(e.to_string()))?;
    let mut out = Cursor::new(Vec::new());
    img.write_to(&mut out, image::ImageFormat::Png)
        .map_err(|e| PptxError::Write(format!("png encode: {e}")))?;
    Ok(("png", out.into_inner()))
}

fn is_png(bytes: &[u8]) -> bool {
    bytes.starts_with(&[0x89, b'P', b'N', b'G'])
}

fn is_jpeg(bytes: &[u8]) -> bool {
    bytes.len() >= 3 && bytes[0] == 0xFF && bytes[1] == 0xD8 && bytes[2] == 0xFF
}

fn is_webp(bytes: &[u8]) -> bool {
    bytes.len() >= 12 && &bytes[0..4] == b"RIFF" && &bytes[8..12] == b"WEBP"
}

fn looks_like_svg(bytes: &[u8]) -> bool {
    let s = String::from_utf8_lossy(bytes);
    let trimmed = s.trim_start().to_ascii_lowercase();
    trimmed.starts_with("<svg") || (trimmed.starts_with("<?xml") && trimmed.contains("<svg"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::codecs::png::PngEncoder;
    use image::{ExtendedColorType, ImageEncoder};
    use k2f_core::Pt;

    fn rgb_png(w: u32, h: u32) -> Vec<u8> {
        let pixels = vec![0u8; (w * h * 3) as usize];
        let mut out = Cursor::new(Vec::new());
        PngEncoder::new(&mut out)
            .write_image(&pixels, w, h, ExtendedColorType::Rgb8)
            .unwrap();
        out.into_inner()
    }

    #[test]
    fn missing_image_returns_write_error() {
        let rect = Rect {
            x: Pt(0),
            y: Pt(0),
            width: Pt(10_000),
            height: Pt(10_000),
        };
        let assets = BTreeMap::new();
        let err = picture_from_draw(
            "pic",
            &rect,
            "nope.png",
            &assets,
            1,
            ImageFit::Contain,
            None,
        )
        .unwrap_err();
        match err {
            PptxError::Write(msg) => assert!(msg.contains("missing image")),
            other => panic!("expected Write, got {other:?}"),
        }
    }

    #[test]
    fn svg_encodes_as_png() {
        let svg = br##"<svg xmlns="http://www.w3.org/2000/svg" width="4" height="4"><rect width="4" height="4" fill="#00f"/></svg>"##;
        let (ext, payload) = encode_media(svg).expect("svg must rasterize");
        assert_eq!(ext, "png");
        assert!(is_png(&payload), "svg must become PNG bytes");
    }

    #[test]
    fn dest_rect_letterboxes_tall_png_into_wide_box() {
        let bytes = rgb_png(1, 2);
        let rect = Rect {
            x: Pt(0),
            y: Pt(0),
            width: Pt(200_000),
            height: Pt(100_000),
        };
        let dest = dest_and_crop(&rect, &bytes, ImageFit::Contain).0;
        assert_eq!(dest.height, rect.height);
        assert_eq!(dest.width, Pt(50_000));
        assert_eq!(dest.x, Pt(75_000));
        assert_eq!(dest.y, rect.y);
    }

    #[test]
    fn dest_and_crop_cover_fills_box_and_sets_src_rect() {
        let bytes = rgb_png(20, 10);
        let rect = Rect {
            x: Pt(0),
            y: Pt(0),
            width: Pt(100_000),
            height: Pt(100_000),
        };
        let (dest, crop) = dest_and_crop(&rect, &bytes, ImageFit::Cover);
        assert_eq!(dest, rect);
        assert_eq!(crop, (25_000, 0, 25_000, 0));
    }
}
