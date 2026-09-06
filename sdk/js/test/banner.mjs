import assert from "node:assert/strict";
import {
  bannerTier,
  issuerLabel,
  KNOWN_ISSUERS,
  paintBanner,
  resolveBannerMode,
} from "../viewer/banner.js";

const fp = "ab".repeat(32);

assert.equal(resolveBannerMode(undefined), "auto");
assert.equal(resolveBannerMode("auto"), "auto");
assert.equal(resolveBannerMode(true), "full");
assert.equal(resolveBannerMode("full"), "full");
assert.equal(resolveBannerMode(false), "off");
assert.equal(resolveBannerMode("off"), "off");

assert.equal(bannerTier("UNSIGNED"), "quiet");
assert.equal(bannerTier("SIGNED"), "positive");
assert.equal(bannerTier("UNLOCKED"), "caution");
assert.equal(bannerTier("BROKEN_INTEGRITY"), "alert");

assert.equal(issuerLabel(fp), "issuer unknown");
assert.equal(issuerLabel(undefined), "issuer unknown");

KNOWN_ISSUERS[fp] = "Acme Legal";
assert.equal(issuerLabel(fp), "Acme Legal");
delete KNOWN_ISSUERS[fp];

function el() {
  return {
    hidden: false,
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

const unsignedAuto = el();
paintBanner(unsignedAuto, { banner: "UNSIGNED" }, "auto");
assert.equal(unsignedAuto.hidden, true);
assert.equal(unsignedAuto.dataset.state, "UNSIGNED");
assert.equal(unsignedAuto.textContent, "");

const signedAuto = el();
paintBanner(
  signedAuto,
  { banner: "SIGNED", fingerprint: fp, generatedBy: "agent.invoice-bot" },
  "auto",
);
assert.match(signedAuto.textContent, /^Signed · issuer unknown/);
assert.match(signedAuto.textContent, /generated_by=agent.invoice-bot/);
assert.ok(signedAuto.classList._c.has("banner-compact"));

const brokenAuto = el();
paintBanner(brokenAuto, { banner: "BROKEN_INTEGRITY", statusCode: "ENGINE_MISMATCH" }, "auto");
assert.equal(brokenAuto.hidden, false);
assert.match(brokenAuto.textContent, /Content and lock do not match/);
assert.match(brokenAuto.textContent, /ENGINE_MISMATCH/);

const signed = el();
paintBanner(signed, {
  banner: "SIGNED",
  fingerprint: fp,
  signedBy: "Eve the Forger",
  generatedBy: "agent.invoice-bot",
  signedAt: 1704067200,
}, "full");
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
}, "full");
assert.match(known.textContent, /SIGNED · Acme Legal · /);
assert.doesNotMatch(known.textContent, /claimed signed_by/);
delete KNOWN_ISSUERS[fp];

console.log("ok banner identity");
