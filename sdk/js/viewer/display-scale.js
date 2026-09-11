/** Display paint LOD: continuous UI zoom, quantized render_page scale. */

export const DISPLAY_PAINT_DEBOUNCE_MS = 120;

/** Paint-scale buckets (pixels per document pt). Max matches PDF stamp cap. */
export const PAINT_BUCKETS = [1, 1.25, 1.5, 2, 2.5, 3, 4];

const OFFICIAL = 2;

/** Web: CSS size is page_pt × zoom; device pixels need zoom × dpr. */
export function neededPaintScale(uiZoom, dpr = 1) {
  const z = Number(uiZoom);
  const d = Number(dpr);
  if (!Number.isFinite(z) || z <= 0) return OFFICIAL;
  if (!Number.isFinite(d) || d <= 0) return OFFICIAL;
  return z * d;
}

/** Smallest bucket >= needed, clamped to the last bucket. */
export function quantizePaintScale(needed) {
  const n = Number(needed);
  if (!Number.isFinite(n) || n <= 0) return OFFICIAL;
  const max = PAINT_BUCKETS[PAINT_BUCKETS.length - 1];
  if (n >= max) return max;
  for (const b of PAINT_BUCKETS) {
    if (b + Number.EPSILON >= n) return b;
  }
  return max;
}
