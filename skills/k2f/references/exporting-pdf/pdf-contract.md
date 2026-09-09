# PDF output contract

Load when the exported PDF looks wrong or you are checking bytes.

## Pages

By default the PDF has the same page count as the lock — only the document pages, no extra material.

With `--trust-pack` (CLI) or `PdfExportOptions::with_trust_pack()` (Rust), page count = lock pages **+ 1**. The last page is **integrity verification** (status, `status_code`, `content_hash`, `appearance_hash`, fingerprint / signed_by / generated_by when present).

Page size comes from the lock `page_config`, not the viewer viewport.

## Source caption (trust pack only)

When trust pack is enabled, each content page draws a small caption:

`Official source is K2F. appearance_hash={hash}`

The PDF Info subject uses the same line. `scripts/check-pdf.py` is for **trust-pack** PDFs: it looks for `appearance_hash=` or `Official source is K2F` in the file bytes (Info / uncompressed strings). Default export has neither — do not run the checker on it. Content streams are Flate-compressed — do **not** assert `% k2f.verify` in the raw file.

## Selectable text

Vector pages: visible glyph outlines plus an invisible text layer (`3 Tr`).

**Stamp pages** (see below): the lock is painted via `k2f_paint` at the export scale, then a full-page image is drawn. Body text is **not** duplicated as visible outlines — only the invisible selectable layer remains. Do not overlay DOM text, html2pdf, or a second PDF library.

## Paint ops that stamp (raster at export scale)

Per-page detection. If the render plan includes any of:

- `backdrop_blur`
- `DrawBox` with `shadow`
- `DrawBox` with `linear_gradient` fill

…the exporter runs `k2f_paint` at scale **2**, **3**, or **4** (default **4**; `--scale` / `export_pdf_at`), embeds the page as one RGB image, and keeps invisible selectable text. This is **not** a second layout pass — geometry stays in the lock.

Pages without those ops stay on the vector path (visible outlines + invisible layer).

`RASTER_PAINT_OP` should not appear for supported ops. If it does, the stamp detector missed an op — file a bug; do not fake effects in jsPDF.

`UNKNOWN_PAINT_OP`: lock contains an op this engine cannot execute — engine/package version mismatch. Do not skip ops.

## Broken and unlocked

| Banner | Export |
|--------|--------|
| `UNLOCKED` (no `document.K2F.lock`) | Fails |
| `BROKEN_INTEGRITY` / `SIGNED_BUT_BROKEN` | Still draws the **published** lock. Warn the user; do not treat the PDF as a valid signed original |
| Missing embedded fonts | Fails (“package has no embedded font”) |
