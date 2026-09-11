import assert from "node:assert/strict";
import {
  DISPLAY_PAINT_DEBOUNCE_MS,
  neededPaintScale,
  quantizePaintScale,
  PAINT_BUCKETS,
} from "../viewer/display-scale.js";
import { cacheKey } from "../viewer/paint-page.js";

assert.equal(DISPLAY_PAINT_DEBOUNCE_MS, 120);
assert.deepEqual(PAINT_BUCKETS, [1, 1.25, 1.5, 2, 2.5, 3, 4]);

assert.equal(neededPaintScale(1, 2), 2);
assert.equal(neededPaintScale(1.5, 2), 3);
assert.equal(neededPaintScale(2, 2), 4);

assert.equal(quantizePaintScale(2), 2);
assert.equal(quantizePaintScale(2.01), 2.5);
assert.equal(quantizePaintScale(2.5), 2.5);
assert.equal(quantizePaintScale(2.51), 3);
assert.equal(quantizePaintScale(3.1), 4);
assert.equal(quantizePaintScale(9), 4);
assert.equal(quantizePaintScale(0.5), 1);

assert.equal(cacheKey(0, 2), "0:2");
assert.equal(cacheKey(3, 2.5), "3:2.5");

console.log("display-scale: ok");
