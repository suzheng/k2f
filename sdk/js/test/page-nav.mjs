import assert from "node:assert/strict";
import { PAGE_GAP, pageAtScroll } from "../viewer/page-nav.js";

assert.equal(PAGE_GAP, 24);

const h = 100;
const tops = [0, h + PAGE_GAP, 2 * (h + PAGE_GAP)];
const view = h;

assert.equal(pageAtScroll(0, view, tops), 0, "top of stack is page 0");
assert.equal(pageAtScroll(1, view, tops), 0, "small scroll stays on page 0");

const intoPage1 = tops[1] - view * 0.25;
assert.equal(pageAtScroll(intoPage1, view, tops), 1, "probe at page 1 top is page 1");

const intoPage2 = tops[2] - view * 0.25;
assert.equal(pageAtScroll(intoPage2, view, tops), 2, "probe at page 2 top is page 2");

assert.equal(pageAtScroll(0, view, []), 0, "empty stack stays 0");
assert.equal(pageAtScroll(-10, view, tops), 0, "negative scroll clamps to page 0");

console.log("ok page-nav pageAtScroll");
