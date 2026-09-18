use crate::ir::PictureBox;
use crate::IdmlError;
use k2f_core::{ImageFit, Rect};
use k2f_paint::{decode_raster, letterbox_rect, lookup_image};
use std::collections::BTreeMap;
use std::io::Cursor;

pub fn picture_from_draw(
    node_id: &str,
    rect: &Rect,
    src: &str,
    assets: &BTreeMap<String, Vec<u8>>,
    media_index: u32,
    fit: ImageFit,
    corner_radius_pt: Option<i64>,
) -> Result<PictureBox, IdmlError> {
    let bytes = lookup_image(assets, src)
        .ok_or_else(|| IdmlError::Write(format!("missing image '{src}'")))?;
    let dest = match fit {
        ImageFit::Cover => rect.clone(),
        ImageFit::Contain => dest_rect_for_image(rect, bytes),
    };
    let (ext, payload) = encode_media(bytes)?;
    Ok(PictureBox {
        node_id: node_id.to_string(),
        rect: dest,
        raster_name: format!("image{media_index}.{ext}"),
        bytes: payload,
        ext: ext.to_string(),
        corner_pt: corner_radius_pt.unwrap_or(0).max(0) as f64 / 1000.0,
        fill_proportionally: matches!(fit, ImageFit::Cover),
    })
}

fn dest_rect_for_image(rect: &Rect, bytes: &[u8]) -> Rect {
    decode_raster(bytes)
        .ok()
        .and_then(|img| letterbox_rect(img.width(), img.height(), rect))
        .unwrap_or_else(|| rect.clone())
}

fn encode_media(bytes: &[u8]) -> Result<(&'static str, Vec<u8>), IdmlError> {
    if is_png(bytes) {
        return Ok(("png", bytes.to_vec()));
    }
    if is_jpeg(bytes) {
        return Ok(("jpg", bytes.to_vec()));
    }
    if looks_like_svg(bytes) || is_webp(bytes) {
        return raster_to_png(bytes);
    }
    match raster_to_png(bytes) {
        Ok(png) => Ok(png),
        Err(_) => Err(IdmlError::Write(
            "unsupported image bytes (need png/jpg/webp/svg)".into(),
        )),
    }
}

fn raster_to_png(bytes: &[u8]) -> Result<(&'static str, Vec<u8>), IdmlError> {
    let img = decode_raster(bytes).map_err(|e| IdmlError::Write(e.to_string()))?;
    let mut out = Cursor::new(Vec::new());
    img.write_to(&mut out, image::ImageFormat::Png)
        .map_err(|e| IdmlError::Write(format!("png encode: {e}")))?;
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
    fn cover_uses_full_box_and_fill_proportionally() {
        let bytes = rgb_png(20, 10);
        let mut assets = BTreeMap::new();
        assets.insert("wide.png".into(), bytes);
        let rect = Rect {
            x: Pt(0),
            y: Pt(0),
            width: Pt(100_000),
            height: Pt(100_000),
        };
        let pic = picture_from_draw(
            "pic",
            &rect,
            "wide.png",
            &assets,
            1,
            ImageFit::Cover,
            Some(12_000),
        )
        .unwrap();
        assert_eq!(pic.rect, rect);
        assert!(pic.fill_proportionally);
        assert!((pic.corner_pt - 12.0).abs() < 0.001);
    }
}
