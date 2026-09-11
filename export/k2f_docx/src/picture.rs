use crate::coord::pt_to_emu;
use crate::ir::PictureBox;
use crate::xml::escape_xml;
use crate::DocxError;
use k2f_core::Rect;
use k2f_paint::{decode_raster, letterbox_rect, lookup_image};
use std::collections::BTreeMap;
use std::io::Cursor;

pub(crate) fn picture_from_draw(
    node_id: &str,
    rect: &Rect,
    src: &str,
    assets: &BTreeMap<String, Vec<u8>>,
    media_index: u32,
    relative_height: u32,
) -> Result<PictureBox, DocxError> {
    let bytes = lookup_image(assets, src)
        .ok_or_else(|| DocxError::Write(format!("missing image '{src}'")))?;
    let dest = dest_rect_for_image(rect, bytes);
    let (ext, payload) = encode_media(bytes)?;
    Ok(PictureBox {
        node_id: node_id.to_string(),
        x_emu: pt_to_emu(dest.x),
        y_emu: pt_to_emu(dest.y),
        cx_emu: pt_to_emu(dest.width),
        cy_emu: pt_to_emu(dest.height),
        media_name: format!("image{media_index}.{ext}"),
        bytes: payload,
        relative_height,
        pin_empty_txbox: false,
    })
}

fn dest_rect_for_image(rect: &Rect, bytes: &[u8]) -> Rect {
    decode_raster(bytes)
        .ok()
        .and_then(|img| letterbox_rect(img.width(), img.height(), rect))
        .unwrap_or_else(|| rect.clone())
}

pub(crate) fn encode_media(bytes: &[u8]) -> Result<(&'static str, Vec<u8>), DocxError> {
    if is_png(bytes) {
        return Ok(("png", bytes.to_vec()));
    }
    if is_jpeg(bytes) {
        return Ok(("jpg", bytes.to_vec()));
    }
    // SVG (and WebP / other decode_raster formats) → PNG. Office hosts often
    // leave raw `image/svg+xml` blank (LibreOffice Writer; uneven Word support),
    // while paint/PDF already rasterize via resvg. Same path for both exporters.
    if looks_like_svg(bytes) || is_webp(bytes) {
        return raster_to_png(bytes);
    }
    match raster_to_png(bytes) {
        Ok(png) => Ok(png),
        Err(_) => Err(DocxError::Write(
            "unsupported image bytes (need png/jpg/webp/svg)".into(),
        )),
    }
}

fn raster_to_png(bytes: &[u8]) -> Result<(&'static str, Vec<u8>), DocxError> {
    let img = decode_raster(bytes).map_err(|e| DocxError::Write(e.to_string()))?;
    let mut out = Cursor::new(Vec::new());
    img.write_to(&mut out, image::ImageFormat::Png)
        .map_err(|e| DocxError::Write(format!("png encode: {e}")))?;
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

pub(crate) fn pic_xml(pic: &PictureBox, embed_rid: &str, cnv_id: u32) -> String {
    pic_xml_named(pic, embed_rid, &pic.node_id, cnv_id)
}

/// Effect slices (gradient / glass / shadow) as a DrawingML shape with a blip
/// fill. LibreOffice Writer paints `pic:pic` above every `wps:wsp` regardless of
/// `behindDoc` / `relativeHeight`, so a full-page gradient raster would cover
/// later text. A `wps:wsp` joins the shape z-order stack.
///
/// Large lock images under later text omit the empty txBox: Writer paints a
/// page-sized empty text frame over later labels (same as a full-page
/// gradient). Effect slices keep the empty txBox only when later paint does
/// not overlap — a shadowed card shell would otherwise hide every label.
pub(crate) fn raster_wsp_xml(pic: &PictureBox, embed_rid: &str) -> String {
    let (cnv, tail) = if pic.pin_empty_txbox {
        (
            "                  <wps:cNvSpPr txBox=\"1\"/>\n",
            "                  <wps:txbx>\n                    <w:txbxContent>\n                      <w:p/>\n                    </w:txbxContent>\n                  </wps:txbx>\n                  <wps:bodyPr wrap=\"square\" lIns=\"0\" tIns=\"0\" rIns=\"0\" bIns=\"0\" anchor=\"t\">\n                    <a:noAutofit/>\n                  </wps:bodyPr>\n",
        )
    } else {
        (
            "                  <wps:cNvSpPr/>\n",
            "                  <wps:bodyPr/>\n",
        )
    };
    format!(
        r#"                <wps:wsp>
{cnv}                  <wps:spPr>
                    <a:xfrm>
                      <a:off x="0" y="0"/>
                      <a:ext cx="{cx}" cy="{cy}"/>
                    </a:xfrm>
                    <a:prstGeom prst="rect">
                      <a:avLst/>
                    </a:prstGeom>
                    <a:blipFill>
                      <a:blip r:embed="{rid}"/>
                      <a:stretch>
                        <a:fillRect/>
                      </a:stretch>
                    </a:blipFill>
                    <a:ln>
                      <a:noFill/>
                    </a:ln>
                  </wps:spPr>
{tail}                </wps:wsp>
"#,
        cx = pic.cx_emu,
        cy = pic.cy_emu,
        rid = embed_rid,
    )
}

fn pic_xml_named(pic: &PictureBox, embed_rid: &str, name: &str, cnv_id: u32) -> String {
    let name = escape_xml(name);
    format!(
        r#"                <pic:pic>
                  <pic:nvPicPr>
                    <pic:cNvPr id="{cnv_id}" name="{name}"/>
                    <pic:cNvPicPr>
                      <a:picLocks noChangeAspect="0"/>
                    </pic:cNvPicPr>
                  </pic:nvPicPr>
                  <pic:blipFill>
                    <a:blip r:embed="{rid}"/>
                    <a:stretch>
                      <a:fillRect/>
                    </a:stretch>
                  </pic:blipFill>
                  <pic:spPr>
                    <a:xfrm>
                      <a:off x="0" y="0"/>
                      <a:ext cx="{cx}" cy="{cy}"/>
                    </a:xfrm>
                    <a:prstGeom prst="rect">
                      <a:avLst/>
                    </a:prstGeom>
                  </pic:spPr>
                </pic:pic>
"#,
        rid = embed_rid,
        cx = pic.cx_emu,
        cy = pic.cy_emu,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use k2f_core::Pt;

    #[test]
    fn missing_image_returns_write_error() {
        let rect = Rect {
            x: Pt(0),
            y: Pt(0),
            width: Pt(10_000),
            height: Pt(10_000),
        };
        let err = picture_from_draw("pic", &rect, "nope.png", &BTreeMap::new(), 1, 0).unwrap_err();
        match err {
            DocxError::Write(msg) => assert!(msg.contains("missing image")),
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
    fn png_keeps_original_bytes() {
        let png = [0x89, b'P', b'N', b'G', 0, 0, 0, 0];
        let (ext, payload) = encode_media(&png).unwrap();
        assert_eq!(ext, "png");
        assert_eq!(payload, png);
    }

    #[test]
    fn webp_encodes_as_png() {
        // 1×1 red VP8 WebP from Pillow.
        let webp: &[u8] = &[
            0x52, 0x49, 0x46, 0x46, 0x3c, 0x00, 0x00, 0x00, 0x57, 0x45, 0x42, 0x50, 0x56, 0x50,
            0x38, 0x20, 0x30, 0x00, 0x00, 0x00, 0xd0, 0x01, 0x00, 0x9d, 0x01, 0x2a, 0x01, 0x00,
            0x01, 0x00, 0x01, 0x40, 0x26, 0x25, 0xa0, 0x02, 0x74, 0xba, 0x01, 0xf8, 0x00, 0x03,
            0xb0, 0x00, 0xfe, 0xf2, 0xeb, 0x7f, 0xfc, 0xd8, 0x15, 0xcd, 0x73, 0xef, 0xf7, 0xff,
            0xd2, 0xe0, 0xfd, 0x2e, 0x0f, 0xd2, 0xe0, 0xff, 0xd2, 0x90, 0x00, 0x00,
        ];
        assert!(is_webp(webp));
        let (ext, payload) = encode_media(webp).expect("webp must decode");
        assert_eq!(ext, "png");
        assert!(is_png(&payload), "webp must become PNG bytes");
    }

    #[test]
    fn dest_rect_letterboxes_tall_png_into_wide_box() {
        use image::codecs::png::PngEncoder;
        use image::{ExtendedColorType, ImageEncoder};
        let pixels = [0u8; 6];
        let mut out = Cursor::new(Vec::new());
        PngEncoder::new(&mut out)
            .write_image(&pixels, 1, 2, ExtendedColorType::Rgb8)
            .unwrap();
        let bytes = out.into_inner();
        let rect = Rect {
            x: Pt(0),
            y: Pt(0),
            width: Pt(200_000),
            height: Pt(100_000),
        };
        let dest = dest_rect_for_image(&rect, &bytes);
        assert_eq!(dest.width, Pt(50_000));
        assert_eq!(dest.x, Pt(75_000));
    }
}
