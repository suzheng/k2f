const COPY = {
  SIGNED: "SIGNED",
  UNSIGNED: "UNSIGNED — hashes match. No signature. Do not treat this as a signed original.",
  SIGNED_BUT_BROKEN:
    "SIGNED-BUT-BROKEN — a signature is present but the lock no longer matches. Do not trust the green check. The page below is the old published lock, not a reflow.",
  UNLOCKED: "UNLOCKED — this is a draft. There is no published lock to sign.",
  BROKEN_INTEGRITY:
    "BROKEN INTEGRITY — content and lock do not match. Do not sign. The page below is the old published lock, not a reflow.",
};

/** Org config, not part of the file format. fingerprint → display name. */
export const KNOWN_ISSUERS = {};

/** Trusted name comes only from KNOWN_ISSUERS. signed_by on the file is an unbound claim. */
export function issuerLabel(fingerprint) {
  if (fingerprint && KNOWN_ISSUERS[fingerprint]) {
    return KNOWN_ISSUERS[fingerprint];
  }
  return "issuer unknown";
}

export function paintBanner(el, info) {
  el.classList.remove(
    "banner-idle",
    "banner-valid",
    "banner-signed",
    "banner-unsigned",
    "banner-unlocked",
    "banner-broken",
    "banner-signed-broken"
  );
  const banner = info.banner;
  el.dataset.state = banner;
  if (banner === "SIGNED") {
    el.classList.add("banner-signed");
    const parts = [COPY.SIGNED, issuerLabel(info.fingerprint), info.fingerprint];
    if (info.generatedBy) parts.push(`generated_by=${info.generatedBy}`);
    if (info.signedBy && issuerLabel(info.fingerprint) === "issuer unknown") {
      parts.push(`claimed signed_by=${info.signedBy}`);
    }
    if (info.signedAt != null && info.signedAt !== "") {
      parts.push(`signed_at=${info.signedAt}`);
    }
    el.textContent = parts.filter(Boolean).join(" · ");
  } else if (banner === "UNSIGNED") {
    el.classList.add("banner-unsigned");
    el.textContent = info.generatedBy
      ? `${COPY.UNSIGNED} generated_by=${info.generatedBy}`
      : COPY.UNSIGNED;
  } else if (banner === "SIGNED_BUT_BROKEN") {
    el.classList.add("banner-signed-broken");
    const fp = info.fingerprint ? ` ${info.fingerprint}` : "";
    el.textContent = `${COPY.SIGNED_BUT_BROKEN} (${info.hashCode || info.statusCode})${fp}`;
  } else if (banner === "UNLOCKED") {
    el.classList.add("banner-unlocked");
    el.textContent = COPY.UNLOCKED;
  } else {
    el.classList.add("banner-broken");
    el.textContent = `${COPY.BROKEN_INTEGRITY} (${info.statusCode})`;
  }
}

export function paintOpenError(el, message) {
  el.classList.remove(
    "banner-idle",
    "banner-valid",
    "banner-signed",
    "banner-unsigned",
    "banner-unlocked",
    "banner-broken",
    "banner-signed-broken"
  );
  el.classList.add("banner-broken");
  el.dataset.state = "ERROR";
  el.textContent = `Cannot open this file. ${message}`;
}
