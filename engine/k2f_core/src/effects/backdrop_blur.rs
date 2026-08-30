use super::box_blur::box_blur_premul_rgba;

/// Apply a deterministic backdrop blur to a rectangular region of a premultiplied RGBA8 framebuffer.
///
/// Contract:
/// - `data` is premultiplied RGBA8 (R,G,B already multiplied by A).
/// - Edge handling for sampling: clamp-to-edge (replicate).
/// - Blur implementation: separable integer box blur (via `box_blur_premul_rgba`).
/// - Rounding in blur: nearest integer (as defined by the blur helper).
/// - Optional rounded-rect clipping is applied when writing blurred pixels back.
pub fn apply_backdrop_blur_premul_rgba(
    data: &mut [u8],
    width: usize,
    height: usize,
    rect_x: usize,
    rect_y: usize,
    rect_w: usize,
    rect_h: usize,
    radius: usize,
    passes: usize,
    corner_radius_px: Option<usize>,
) {
    if width == 0 || height == 0 || rect_w == 0 || rect_h == 0 || radius == 0 || passes == 0 {
        return;
    }

    let expected_len = width
        .checked_mul(height)
        .and_then(|v| v.checked_mul(4))
        .expect("buffer size overflow");
    assert_eq!(
        data.len(),
        expected_len,
        "buffer length mismatch for backdrop blur"
    );

    // Clamp rect bounds to the framebuffer.
    if rect_x >= width || rect_y >= height {
        return;
    }
    let rect_w = rect_w.min(width - rect_x);
    let rect_h = rect_h.min(height - rect_y);
    if rect_w == 0 || rect_h == 0 {
        return;
    }

    // Approximate a Gaussian-ish blur by doing multiple box-blur passes.
    // We sample an expanded region so the blur within the rect is correct.
    let margin = radius.saturating_mul(passes);
    let tmp_w = rect_w.saturating_add(margin.saturating_mul(2));
    let tmp_h = rect_h.saturating_add(margin.saturating_mul(2));
    if tmp_w == 0 || tmp_h == 0 {
        return;
    }

    let mut tmp = vec![0u8; tmp_w * tmp_h * 4];

    // Copy from framebuffer into tmp with clamp-to-edge sampling.
    // tmp pixel (tx,ty) samples from framebuffer at:
    //   sx = clamp(rect_x - margin + tx, 0..width-1)
    //   sy = clamp(rect_y - margin + ty, 0..height-1)
    for ty in 0..tmp_h {
        let sy = clamp_i32(
            rect_y as i32 - margin as i32 + ty as i32,
            0,
            (height - 1) as i32,
        ) as usize;
        for tx in 0..tmp_w {
            let sx = clamp_i32(
                rect_x as i32 - margin as i32 + tx as i32,
                0,
                (width - 1) as i32,
            ) as usize;
            let s_idx = (sy * width + sx) * 4;
            let t_idx = (ty * tmp_w + tx) * 4;
            tmp[t_idx..t_idx + 4].copy_from_slice(&data[s_idx..s_idx + 4]);
        }
    }

    // Blur in-place.
    box_blur_premul_rgba(&mut tmp, tmp_w, tmp_h, radius, passes);

    // Write the blurred pixels back into the rect region, optionally clipped to a rounded rect.
    let clip_r = corner_radius_px
        .unwrap_or(0)
        .min(rect_w / 2)
        .min(rect_h / 2);
    for y in 0..rect_h {
        for x in 0..rect_w {
            if clip_r > 0 && !inside_rounded_rect(x, y, rect_w, rect_h, clip_r) {
                continue;
            }
            let dst_x = rect_x + x;
            let dst_y = rect_y + y;
            let d_idx = (dst_y * width + dst_x) * 4;

            let src_tx = x + margin;
            let src_ty = y + margin;
            let t_idx = (src_ty * tmp_w + src_tx) * 4;

            data[d_idx..d_idx + 4].copy_from_slice(&tmp[t_idx..t_idx + 4]);
        }
    }
}

fn clamp_i32(v: i32, lo: i32, hi: i32) -> i32 {
    if v < lo {
        lo
    } else if v > hi {
        hi
    } else {
        v
    }
}

fn inside_rounded_rect(x: usize, y: usize, w: usize, h: usize, r: usize) -> bool {
    if r == 0 || w == 0 || h == 0 {
        return true;
    }
    let r = r.min(w / 2).min(h / 2);
    if r == 0 {
        return true;
    }

    // Use doubled coordinates to represent pixel centers exactly without floats:
    // center(x) = 2*x + 1
    let cx = (2 * x + 1) as i64;
    let cy = (2 * y + 1) as i64;
    let rr = (2 * r) as i64;
    let rr2 = rr * rr;

    // Top-left
    if x < r && y < r {
        return dist2(cx, cy, rr, rr) <= rr2;
    }
    // Top-right
    if x >= w - r && y < r {
        return dist2(cx, cy, (2 * (w - r)) as i64, rr) <= rr2;
    }
    // Bottom-left
    if x < r && y >= h - r {
        return dist2(cx, cy, rr, (2 * (h - r)) as i64) <= rr2;
    }
    // Bottom-right
    if x >= w - r && y >= h - r {
        return dist2(cx, cy, (2 * (w - r)) as i64, (2 * (h - r)) as i64) <= rr2;
    }

    true
}

fn dist2(ax: i64, ay: i64, bx: i64, by: i64) -> i64 {
    let dx = ax - bx;
    let dy = ay - by;
    dx * dx + dy * dy
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_test_buf(width: usize, height: usize) -> Vec<u8> {
        // A simple deterministic pattern with alpha variation.
        let mut out = vec![0u8; width * height * 4];
        for y in 0..height {
            for x in 0..width {
                let idx = (y * width + x) * 4;
                let r = (x * 40 + y * 5) as u8;
                let g = (x * 10 + y * 30) as u8;
                let a = 255u8;
                // Premultiplied = same as straight when alpha=255.
                out[idx] = r;
                out[idx + 1] = g;
                out[idx + 2] = 0;
                out[idx + 3] = a;
            }
        }
        out
    }

    #[test]
    fn backdrop_blur_is_deterministic_across_runs() {
        let mut a = make_test_buf(9, 7);
        let mut b = a.clone();

        apply_backdrop_blur_premul_rgba(&mut a, 9, 7, 2, 2, 5, 3, 2, 3, Some(2));
        apply_backdrop_blur_premul_rgba(&mut b, 9, 7, 2, 2, 5, 3, 2, 3, Some(2));

        assert_eq!(a, b);
    }

    #[test]
    fn backdrop_blur_changes_pixels_in_region() {
        let mut buf = make_test_buf(9, 7);
        let before = buf.clone();
        apply_backdrop_blur_premul_rgba(&mut buf, 9, 7, 2, 2, 5, 3, 1, 3, None);

        // At least one byte should differ.
        assert!(buf.iter().zip(before.iter()).any(|(a, b)| a != b));
    }
}
