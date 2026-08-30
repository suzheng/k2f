import assert from "node:assert/strict";
import { issuerLabel, KNOWN_ISSUERS, paintBanner } from "../viewer/banner.js";

const fp = "ab".repeat(32);

assert.equal(issuerLabel(fp), "issuer unknown");
assert.equal(issuerLabel(undefined), "issuer unknown");

KNOWN_ISSUERS[fp] = "Acme Legal";
assert.equal(issuerLabel(fp), "Acme Legal");
delete KNOWN_ISSUERS[fp];

function el() {
  return {
    classList: {
      _c: new Set(),
      remove(...names) {
        for (const n of names) this._c.delete(n);
      },
      add(name) {
        this._c.add(name);
      },
    },
    dataset: {},
    textContent: "",
  };
}

const signed = el();
paintBanner(signed, {
  banner: "SIGNED",
  fingerprint: fp,
  signedBy: "Eve the Forger",
  generatedBy: "agent.invoice-bot",
  signedAt: 1704067200,
});
assert.match(signed.textContent, /SIGNED · issuer unknown · /);
assert.match(signed.textContent, /generated_by=agent.invoice-bot/);
assert.match(signed.textContent, /claimed signed_by=Eve the Forger/);
assert.match(signed.textContent, /signed_at=1704067200/);
assert.ok(
  !signed.textContent.startsWith("SIGNED · Eve"),
  "file signed_by must not look like a verified issuer",
);

KNOWN_ISSUERS[fp] = "Acme Legal";
const known = el();
paintBanner(known, {
  banner: "SIGNED",
  fingerprint: fp,
  signedBy: "Eve the Forger",
});
assert.match(known.textContent, /SIGNED · Acme Legal · /);
assert.doesNotMatch(known.textContent, /claimed signed_by/);
delete KNOWN_ISSUERS[fp];

console.log("ok banner identity");
