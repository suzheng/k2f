import assert from "node:assert/strict";
import { boxToCss } from "../viewer/coords.js";

const css = boxToCss({ x: 72000, y: 36000, width: 10000, height: 5000 }, 2);
assert.equal(css.left, 144);
assert.equal(css.top, 72);
assert.equal(css.width, 20);
assert.equal(css.height, 10);

console.log("ok coords boxToCss");
