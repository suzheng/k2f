/** UI zoom steps and fit-to-width on open. */

export const ZOOM_STEPS = [0.5, 0.75, 1, 1.25, 1.5, 2];

const STAGE_PAD = 48;

/** Scrollport inner width (stage padding is 24px each side). */
export function stageInnerWidth(stage) {
  return Math.max(0, stage.clientWidth - STAGE_PAD);
}

/** Largest ZOOM_STEPS value <= raw (never below minimum step). */
export function snapToStep(raw) {
  let best = ZOOM_STEPS[0];
  for (const step of ZOOM_STEPS) {
    if (step <= raw) best = step;
    else break;
  }
  return best;
}

function maxPageWidthPt(viewer) {
  let max = 0;
  for (let p = 0; p < viewer.page_count(); p++) {
    max = Math.max(max, viewer.page_width_pt(p));
  }
  return max;
}

/** Initial zoom: fit the widest page into the stage, capped at 100%. */
export function fitZoom(viewer, stageWidth) {
  if (!viewer || viewer.page_count() === 0 || stageWidth <= 0) return 1;
  const pageW = maxPageWidthPt(viewer);
  if (pageW <= stageWidth) return 1;
  return snapToStep(stageWidth / pageW);
}
