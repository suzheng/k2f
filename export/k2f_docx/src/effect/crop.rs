use crate::coord::pt_to_emu;
use crate::ir::PictureBox;
use crate::DocxError;
use image::codecs::png::PngEncoder;
use image::{ExtendedColorType, ImageEncoder};
use k2f_core::{BoxDecoration, Pt, Rect, ShadowRef};
use k2f_paint::OFFICIAL_PNG_SCALE;
use std::io::Cursor;

pub fn millipt_to_px(millipt: i64, scale: f32) -> i32 {
    let pt = millipt as f64 / 1000.0;
    (pt * f64::from(scale)).round() as i32
}

fn i64_pt(pt: Pt) -> i64 {
    i64::try_from(pt.0).unwrap_or(i64::MAX)
}

pub fn union_rect(a: Rect, b: Rect) -> Rect {
    let x1 = a.x.0.min(b.x.0);
    let y1 = a.y.0.min(b.y.0);
    let x2 = (a.x.0 + a.width.0).max(b.x.0 + b.width.0);
    let y2 = (a.y.0 + a.height.0).max(b.y.0 + b.height.0);
    Rect {
        x: Pt(x1),
        y: Pt(y1),
        width: Pt((x2 - x1).max(1)),
        height: Pt((y2 - y1).max(1)),
    }
}

pub fn clamp_rect_to_page(rect: Rect, page_w: Pt, page_h: Pt) -> Rect {
    let x = rect.x.0.max(0);
    let y = rect.y.0.max(0);
    let x2 = (rect.x.0 + rect.width.0).min(page_w.0).max(x + 1);
    let y2 = (rect.y.0 + rect.height.0).min(page_h.0).max(y + 1);
    Rect {
        x: Pt(x),
        y: Pt(y),
        width: Pt(x2 - x),
        height: Pt(y2 - y),
    }
}

pub fn expand_rect_for_shadow(rect: Rect, decoration: &BoxDecoration) -> Result<Rect, DocxError> {
    let Some(shadow) = decoration.shadow.as_ref() else {
        return Ok(rect);
    };
    let layers = match shadow {
        ShadowRef::Inline(s) => &s.layers,
        ShadowRef::Ref(name) => {
            return Err(DocxError::Write(format!("unresolved shadow ref '{name}'")));
        }
    };
    if layers.is_empty() {
        return Ok(rect);
    }
    let mut pad_l = 0i64;
    let mut pad_t = 0i64;
    let mut pad_r = 0i64;
    let mut pad_b = 0i64;
    for layer in layers {
        let extra = layer.blur_radius_pt.max(0) + layer.spread_radius_pt.max(0);
        pad_l = pad_l.max(extra + (-layer.offset_x_pt).max(0));
        pad_r = pad_r.max(extra + layer.offset_x_pt.max(0));
        pad_t = pad_t.max(extra + (-layer.offset_y_pt).max(0));
        pad_b = pad_b.max(extra + layer.offset_y_pt.max(0));
    }
    let min_pad = ((1000.0 / f64::from(OFFICIAL_PNG_SCALE)).ceil() as i64).max(1);
    if pad_l + pad_r + pad_t + pad_b == 0 {
        pad_l = min_pad;
        pad_r = min_pad;
        pad_t = min_pad;
        pad_b = min_pad;
    }
    Ok(Rect {
        x: Pt(rect.x.0 - i128::from(pad_l)),
        y: Pt(rect.y.0 - i128::from(pad_t)),
        width: Pt(rect.width.0 + i128::from(pad_l + pad_r)),
        height: Pt(rect.height.0 + i128::from(pad_t + pad_b)),
    })
}

pub fn crop_rgb_to_png(
    rgb: &[u8],
    img_w: u32,
    img_h: u32,
    rect: &Rect,
    scale: f32,
) -> Result<Vec<u8>, DocxError> {
    let mut left = millipt_to_px(i64_pt(rect.x), scale);
    let mut top = millipt_to_px(i64_pt(rect.y), scale);
    let mut right = millipt_to_px(i64_pt(rect.x) + i64_pt(rect.width), scale);
    let mut bottom = millipt_to_px(i64_pt(rect.y) + i64_pt(rect.height), scale);
    let w = img_w as i32;
    let h = img_h as i32;
    if w <= 0 || h <= 0 {
        return Err(DocxError::Write("empty page bitmap".into()));
    }
    left = left.clamp(0, w.saturating_sub(1));
    top = top.clamp(0, h.saturating_sub(1));
    right = right.clamp(left + 1, w);
    bottom = bottom.clamp(top + 1, h);
    let cw = (right - left) as u32;
    let ch = (bottom - top) as u32;
    let mut cropped = Vec::with_capacity((cw * ch * 3) as usize);
    for y in top..bottom {
        let row = ((y as u32) * img_w + left as u32) as usize * 3;
        cropped.extend_from_slice(&rgb[row..row + (cw as usize) * 3]);
    }
    encode_png(cw, ch, &cropped)
}

fn encode_png(w: u32, h: u32, rgb: &[u8]) -> Result<Vec<u8>, DocxError> {
    let mut out = Cursor::new(Vec::new());
    PngEncoder::new(&mut out)
        .write_image(rgb, w, h, ExtendedColorType::Rgb8)
        .map_err(|e| DocxError::Write(format!("png encode: {e}")))?;
    Ok(out.into_inner())
}

pub fn picture_from_crop(
    node_id: &str,
    crop: &Rect,
    png: Vec<u8>,
    raster_index: u32,
    relative_height: u32,
) -> PictureBox {
    PictureBox {
        node_id: node_id.to_string(),
        x_emu: pt_to_emu(crop.x),
        y_emu: pt_to_emu(crop.y),
        cx_emu: pt_to_emu(crop.width),
        cy_emu: pt_to_emu(crop.height),
        media_name: format!("raster{raster_index}.png"),
        bytes: png,
        relative_height,
        pin_empty_txbox: true,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn millipt_to_px_matches_official_scale() {
        assert_eq!(millipt_to_px(960_000, 2.0), 1920);
        assert_eq!(millipt_to_px(0, 2.0), 0);
    }
}
