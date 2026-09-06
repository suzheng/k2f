const FULL_COPY = {
  SIGNED: "SIGNED",
  UNSIGNED: "UNSIGNED — hashes match. No signature. Do not treat this as a signed original.",
  SIGNED_BUT_BROKEN:
    "SIGNED-BUT-BROKEN — a signature is present but the lock no longer matches. Do not trust the green check. The page below is the old published lock, not a reflow.",
  UNLOCKED: "UNLOCKED — this is a draft. There is no published lock to sign.",
  BROKEN_INTEGRITY:
    "BROKEN INTEGRITY — content and lock do not match. Do not sign. The page below is the old published lock, not a reflow.",
};

const AUTO_COPY = {
  SIGNED_BUT_BROKEN:
    "Signature invalid — the lock no longer matches. The page below is the old published lock.",
  UNLOCKED: "Draft — not published yet",
  BROKEN_INTEGRITY:
    "Content and lock do not match. Do not sign. The page below is the old published lock.",
};

const BANNER_CLASSES = [
  "banner-idle",
  "banner-valid",
  "banner-signed",
  "banner-unsigned",
  "banner-unlocked",
  "banner-broken",
  "banner-signed-broken",
  "banner-compact",
];

/** Org config, not part of the file format. fingerprint → display name. */
export const KNOWN_ISSUERS = {};

/** Trusted name comes only from KNOWN_ISSUERS. signed_by on the file is an unbound claim. */
export function issuerLabel(fingerprint) {
  if (fingerprint && KNOWN_ISSUERS[fingerprint]) {
    return KNOWN_ISSUERS[fingerprint];
  }
  return "issuer unknown";
}

/** @param {boolean | "auto" | "full" | "off" | undefined} banner */
export function resolveBannerMode(banner) {
  if (banner === false || banner === "off") return "off";
  if (banner === true || banner === "full") return "full";
  return "auto";
}

/** @param {string} banner */
export function bannerTier(banner) {
  if (banner === "UNSIGNED") return "quiet";
  if (banner === "SIGNED") return "positive";
  if (banner === "UNLOCKED") return "caution";
  return "alert";
}

function clearBannerClasses(el) {
  el.classList.remove(...BANNER_CLASSES);
}

function paintQuiet(el, banner) {
  clearBannerClasses(el);
  el.hidden = true;
  el.dataset.state = banner;
  el.textContent = "";
}

function signedAutoText(info) {
  const label = issuerLabel(info.fingerprint);
  const parts = [`Signed · ${label}`];
  if (info.generatedBy) parts.push(`generated_by=${info.generatedBy}`);
  return parts.join(" · ");
}

function signedFullText(info) {
  const parts = [FULL_COPY.SIGNED, issuerLabel(info.fingerprint), info.fingerprint];
  if (info.generatedBy) parts.push(`generated_by=${info.generatedBy}`);
  if (info.signedBy && issuerLabel(info.fingerprint) === "issuer unknown") {
    parts.push(`claimed signed_by=${info.signedBy}`);
  }
  if (info.signedAt != null && info.signedAt !== "") {
    parts.push(`signed_at=${info.signedAt}`);
  }
  return parts.filter(Boolean).join(" · ");
}

/**
 * @param {HTMLElement} el
 * @param {object} info
 * @param {"auto" | "full" | "off"} [mode]
 */
export function paintBanner(el, info, mode = "auto") {
  if (mode === "off") {
    el.hidden = true;
    return;
  }

  const banner = info.banner;
  el.dataset.state = banner;

  if (mode === "auto" && bannerTier(banner) === "quiet") {
    paintQuiet(el, banner);
    return;
  }

  el.hidden = false;
  clearBannerClasses(el);

  const compact = mode === "auto";

  if (banner === "SIGNED") {
    el.classList.add("banner-signed");
    if (compact) el.classList.add("banner-compact");
    el.textContent = compact ? signedAutoText(info) : signedFullText(info);
  } else if (banner === "UNSIGNED") {
    el.classList.add("banner-unsigned");
    el.textContent = info.generatedBy
      ? `${FULL_COPY.UNSIGNED} generated_by=${info.generatedBy}`
      : FULL_COPY.UNSIGNED;
  } else if (banner === "SIGNED_BUT_BROKEN") {
    el.classList.add("banner-signed-broken");
    const fp = info.fingerprint ? ` ${info.fingerprint}` : "";
    const lead = compact ? AUTO_COPY.SIGNED_BUT_BROKEN : FULL_COPY.SIGNED_BUT_BROKEN;
    el.textContent = `${lead} (${info.hashCode || info.statusCode})${fp}`;
  } else if (banner === "UNLOCKED") {
    el.classList.add("banner-unlocked");
    if (compact) el.classList.add("banner-compact");
    el.textContent = compact ? AUTO_COPY.UNLOCKED : FULL_COPY.UNLOCKED;
  } else {
    el.classList.add("banner-broken");
    const lead = compact ? AUTO_COPY.BROKEN_INTEGRITY : FULL_COPY.BROKEN_INTEGRITY;
    el.textContent = `${lead} (${info.statusCode})`;
  }
}

/**
 * @param {HTMLElement} el
 * @param {string} message
 * @param {"auto" | "full" | "off"} [mode]
 */
export function paintOpenError(el, message, mode = "auto") {
  if (mode === "off") {
    el.hidden = true;
    return;
  }
  clearBannerClasses(el);
  el.hidden = false;
  el.classList.add("banner-broken");
  el.dataset.state = "ERROR";
  el.textContent = `Cannot open this file. ${message}`;
}
