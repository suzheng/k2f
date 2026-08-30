import assert from "node:assert/strict";
import { linkUrlFromSelection } from "../viewer/links.js";

assert.equal(
  linkUrlFromSelection({
    node: { modifiers: [{ type: "link", intent: "https://example.com/k2f" }] },
  }),
  "https://example.com/k2f",
);
assert.equal(
  linkUrlFromSelection({
    node: { modifiers: [{ type: "emphasis", intent: "strong" }] },
  }),
  null,
);
assert.equal(
  linkUrlFromSelection({
    text: "See the K2F overview here",
    char_range: [0, 3],
    node: {
      modifiers: [
        { type: "link", intent: "https://example.com/k2f", range: [8, 20] },
      ],
    },
  }),
  null,
);
assert.equal(
  linkUrlFromSelection({
    text: "See the K2F overview here",
    char_range: [8, 12],
    node: {
      modifiers: [
        { type: "link", intent: "https://example.com/k2f", range: [8, 20] },
      ],
    },
  }),
  "https://example.com/k2f",
);

console.log("ok links linkUrlFromSelection");
