# PDF output contract

Load when the exported PDF looks wrong or you are checking bytes.

## Pages

By default the PDF has the same page count as the lock — only the document pages, no extra material.

With `--trust-pack` (CLI) or `PdfExportOptions::with_trust_pack()` (Rust), page count = lock pages **+ 1**. The last page is **integrity verification** (status, `status_code`, `content_hash`, `appearance_hash`, fingerprint / signed_by / generated_by when present).

Page size comes from the lock `page_config`, not the viewer viewport.

## Source caption (trust pack only)

When trust pack is enabled, each content page draws a small caption:

`Official source is K2F. appearance_hash={hash}`

The PDF Info subject uses the same line. `scripts/check-pdf.py` accepts the default (clean) export via uncompressed `/Producer` (`K2F PDF bridge`). Trust-pack PDFs also pass (caption / Info subject). Do not require `--trust-pack` to satisfy the checker. Content streams are Flate-compressed — do **not** assert `% k2f.verify` in the raw file.

## Selectable text

Pages without effect slices: visible glyph outlines plus an invisible text layer (`3 Tr`).

**Effect pages** (see below): blur, shadow, gradient, and translucent boxes are cropped RGBA images with `/SMask`, placed in lock order. Text on those effects is visible glyph outlines, plus the same invisible `3 Tr` layer for copy and search. Do not overlay DOM text, html2pdf, or a second PDF library.

Text that sits *under* a backdrop blur and intersects the slice may be baked into that image so the blur matches the lock. Text drawn after the effect stays vector.

## Paint ops that slice (raster at export scale)

Per-op detection, in render-plan order. A slice is:

- `backdrop_blur` (plus the following `DrawBox` with the same node id, when present)
- `DrawBox` with `shadow`, `blur`, a `linear_gradient` fill, or a fill/stroke whose alpha is below 255

The exporter runs `k2f_paint` at scale **2**, **3**, or **4** (default **4**; `--scale` / `export_pdf_at`) for those ops only, embeds each crop as DeviceRGB plus `/SMask`, and paints the other ops as vectors. This is **not** a second layout pass — geometry stays in the lock. `--scale` changes slice pixels, not page count.

`::rule_` boxes stay vector. Opaque solid boxes, images, and tables stay vector.

`RASTER_PAINT_OP` should not appear for supported ops. If it does, the slice classifier missed an op — file a bug; do not fake effects in jsPDF.

`UNKNOWN_PAINT_OP`: lock contains an op this engine cannot execute — engine/package version mismatch. Do not skip ops.

## Broken and unlocked

| Banner | Export |
|--------|--------|
| `UNLOCKED` (no `document.K2F.lock`) | Fails |
| `BROKEN_INTEGRITY` / `SIGNED_BUT_BROKEN` | Still draws the **published** lock. Warn the user; do not treat the PDF as a valid signed original |
| Missing embedded fonts | Fails (“package has no embedded font”) |
