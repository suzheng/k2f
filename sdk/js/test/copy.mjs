import { clipboardFromSelection, collectK2fNodesFromSelection, layersIn } from "../viewer/copy.js";
import {
  COPY_FORMAT_KEY,
  normalizeCopyFormat,
  readStoredCopyFormat,
  writeStoredCopyFormat,
} from "../viewer/copy-format.js";

if (clipboardFromSelection(null) !== null) throw new Error("null selection");
if (clipboardFromSelection({ isCollapsed: true }) !== null) {
  throw new Error("collapsed selection");
}
if (collectK2fNodesFromSelection(null).length !== 0) {
  throw new Error("null range");
}

const layerA = { id: "a" };
const layerB = { id: "b" };
const stack = {
  querySelectorAll(sel) {
    return sel === ".k2f-text-layer" ? [layerA, layerB] : [];
  },
};
const both = {
  intersectsNode(n) {
    return n === layerA || n === layerB;
  },
};
const onlyA = {
  intersectsNode(n) {
    return n === layerA;
  },
};
if (layersIn(stack, both).length !== 2) {
  throw new Error("must walk every text layer in the range");
}
if (layersIn(stack, onlyA).length !== 1 || layersIn(stack, onlyA)[0] !== layerA) {
  throw new Error("must skip layers outside the range");
}
if (layersIn(null, both).length !== 0) throw new Error("null root");

if (normalizeCopyFormat("plain") !== "plain") throw new Error("plain");
if (normalizeCopyFormat("markdown") !== "markdown") throw new Error("markdown");
if (normalizeCopyFormat("other") !== "markdown") throw new Error("default markdown");
if (COPY_FORMAT_KEY !== "k2f.copyFormat") throw new Error("storage key");

const mem = new Map();
globalThis.localStorage = {
  getItem(k) {
    return mem.has(k) ? mem.get(k) : null;
  },
  setItem(k, v) {
    mem.set(k, String(v));
  },
};
writeStoredCopyFormat("plain");
if (readStoredCopyFormat() !== "plain") throw new Error("persist plain");
writeStoredCopyFormat("markdown");
if (readStoredCopyFormat() !== "markdown") throw new Error("persist markdown");

console.log("ok copy helpers");
