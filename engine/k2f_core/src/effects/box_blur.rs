/// Deterministic, separable box blur for premultiplied RGBA buffers.
///
/// This is intended as a shared building block for soft shadows and other effects where
/// the engine must not rely on platform-specific blur implementations.
///
/// Contract:
/// - Input is **premultiplied** RGBA8 (R,G,B already multiplied by A).
/// - Edge handling: **clamp-to-edge** (replicate).
/// - Rounding: **nearest integer** by adding `div/2` before division.
/// - No SIMD / floats; pure integer math for cross-platform stability.
pub fn box_blur_premul_rgba(
    data: &mut [u8],
    width: usize,
    height: usize,
    radius: usize,
    passes: usize,
) {
    if radius == 0 || passes == 0 || width == 0 || height == 0 {
        return;
    }

    let expected_len = width
        .checked_mul(height)
        .and_then(|v| v.checked_mul(4))
        .expect("buffer size overflow");
    assert_eq!(
        data.len(),
        expected_len,
        "buffer length mismatch for box blur"
    );

    let mut tmp = vec![0u8; expected_len];

    for _ in 0..passes {
        blur_h(data, &mut tmp, width, height, radius);
        blur_v(&tmp, data, width, height, radius);
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

fn blur_h(src: &[u8], dst: &mut [u8], width: usize, height: usize, radius: usize) {
    let w_i32 = width as i32;
    let r_i32 = radius as i32;
    let div = (2 * radius + 1) as u32;
    let half = div / 2;

    for y in 0..height {
        let row_base = y * width * 4;

        // Initialize window for x=0 centered at 0 with clamp-to-edge.
        let mut sum_r: u32 = 0;
        let mut sum_g: u32 = 0;
        let mut sum_b: u32 = 0;
        let mut sum_a: u32 = 0;
        for dx in -r_i32..=r_i32 {
            let sx = clamp_i32(dx, 0, w_i32 - 1) as usize;
            let idx = row_base + sx * 4;
            sum_r += src[idx] as u32;
            sum_g += src[idx + 1] as u32;
            sum_b += src[idx + 2] as u32;
            sum_a += src[idx + 3] as u32;
        }

        for x in 0..width {
            let out = row_base + x * 4;
            dst[out] = ((sum_r + half) / div) as u8;
            dst[out + 1] = ((sum_g + half) / div) as u8;
            dst[out + 2] = ((sum_b + half) / div) as u8;
            dst[out + 3] = ((sum_a + half) / div) as u8;

            // Slide window from x -> x+1
            let x_i32 = x as i32;
            let remove_x = clamp_i32(x_i32 - r_i32, 0, w_i32 - 1) as usize;
            let add_x = clamp_i32(x_i32 + r_i32 + 1, 0, w_i32 - 1) as usize;

            let remove_idx = row_base + remove_x * 4;
            let add_idx = row_base + add_x * 4;

            sum_r = sum_r + src[add_idx] as u32 - src[remove_idx] as u32;
            sum_g = sum_g + src[add_idx + 1] as u32 - src[remove_idx + 1] as u32;
            sum_b = sum_b + src[add_idx + 2] as u32 - src[remove_idx + 2] as u32;
            sum_a = sum_a + src[add_idx + 3] as u32 - src[remove_idx + 3] as u32;
        }
    }
}

fn blur_v(src: &[u8], dst: &mut [u8], width: usize, height: usize, radius: usize) {
    let h_i32 = height as i32;
    let r_i32 = radius as i32;
    let div = (2 * radius + 1) as u32;
    let half = div / 2;

    for x in 0..width {
        // Initialize window for y=0 centered at 0 with clamp-to-edge.
        let mut sum_r: u32 = 0;
        let mut sum_g: u32 = 0;
        let mut sum_b: u32 = 0;
        let mut sum_a: u32 = 0;
        for dy in -r_i32..=r_i32 {
            let sy = clamp_i32(dy, 0, h_i32 - 1) as usize;
            let idx = (sy * width + x) * 4;
            sum_r += src[idx] as u32;
            sum_g += src[idx + 1] as u32;
            sum_b += src[idx + 2] as u32;
            sum_a += src[idx + 3] as u32;
        }

        for y in 0..height {
            let out = (y * width + x) * 4;
            dst[out] = ((sum_r + half) / div) as u8;
            dst[out + 1] = ((sum_g + half) / div) as u8;
            dst[out + 2] = ((sum_b + half) / div) as u8;
            dst[out + 3] = ((sum_a + half) / div) as u8;

            // Slide window from y -> y+1
            let y_i32 = y as i32;
            let remove_y = clamp_i32(y_i32 - r_i32, 0, h_i32 - 1) as usize;
            let add_y = clamp_i32(y_i32 + r_i32 + 1, 0, h_i32 - 1) as usize;

            let remove_idx = (remove_y * width + x) * 4;
            let add_idx = (add_y * width + x) * 4;

            sum_r = sum_r + src[add_idx] as u32 - src[remove_idx] as u32;
            sum_g = sum_g + src[add_idx + 1] as u32 - src[remove_idx + 1] as u32;
            sum_b = sum_b + src[add_idx + 2] as u32 - src[remove_idx + 2] as u32;
            sum_a = sum_a + src[add_idx + 3] as u32 - src[remove_idx + 3] as u32;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn box_blur_is_deterministic_for_tiny_buffer() {
        // 3x1 pixels: [black, white, black] with full alpha.
        let mut buf = vec![
            0, 0, 0, 255, //
            255, 255, 255, 255, //
            0, 0, 0, 255, //
        ];
        let mut buf2 = buf.clone();

        box_blur_premul_rgba(&mut buf, 3, 1, 1, 2);
        box_blur_premul_rgba(&mut buf2, 3, 1, 1, 2);
        assert_eq!(buf, buf2);
    }
}
