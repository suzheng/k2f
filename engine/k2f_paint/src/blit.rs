use tiny_skia::Pixmap;

pub(crate) fn blit_premul(dst: &mut Pixmap, dst_x: i32, dst_y: i32, src: &Pixmap) {
    let dst_w = dst.width() as i32;
    let dst_h = dst.height() as i32;
    let src_w = src.width() as i32;
    let src_h = src.height() as i32;
    let dst_data = dst.data_mut();
    let src_data = src.data();

    for sy in 0..src_h {
        let dy = sy + dst_y;
        if dy < 0 || dy >= dst_h {
            continue;
        }
        for sx in 0..src_w {
            let dx = sx + dst_x;
            if dx < 0 || dx >= dst_w {
                continue;
            }

            let s_idx = ((sy * src_w + sx) * 4) as usize;
            let d_idx = ((dy * dst_w + dx) * 4) as usize;

            let sr = src_data[s_idx] as u32;
            let sg = src_data[s_idx + 1] as u32;
            let sb = src_data[s_idx + 2] as u32;
            let sa = src_data[s_idx + 3] as u32;
            if sa == 0 {
                continue;
            }

            let dr = dst_data[d_idx] as u32;
            let dg = dst_data[d_idx + 1] as u32;
            let db = dst_data[d_idx + 2] as u32;
            let da = dst_data[d_idx + 3] as u32;

            let inv = 255 - sa;
            let out_r = sr + (dr * inv + 127) / 255;
            let out_g = sg + (dg * inv + 127) / 255;
            let out_b = sb + (db * inv + 127) / 255;
            let out_a = sa + (da * inv + 127) / 255;

            dst_data[d_idx] = out_r.min(255) as u8;
            dst_data[d_idx + 1] = out_g.min(255) as u8;
            dst_data[d_idx + 2] = out_b.min(255) as u8;
            dst_data[d_idx + 3] = out_a.min(255) as u8;
        }
    }
}
