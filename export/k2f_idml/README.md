# k2f_idml

Experimental K2F lock → Adobe InDesign `.idml` exporter. Isolated crate; **not** part of `k2f export-pdf`. The official entry is `k2f export-idml`. The crate binary `k2f-idml` remains for converter development.

IDML is a one-way dump of an already-locked package. It is not a K2F source (`IDML_IS_NOT_A_SOURCE`). Geometry comes from the published lock; this is **not a second layout engine**.

Do **not** stamp a full-page PNG and overlay invisible text. That is the PDF-bridge stamp path and would destroy native text, tables, and pictures.

## Capability table

| Input | InDesign object | Editable |
|---|---|---|
| Ordinary text | TextFrame + Story | yes; slight reflow is allowed |
| Plain-text table | Table (inside the positioning TextFrame’s Story) | yes |
| Bitmap / SVG illustration | embedded Rectangle / Image | replaceable (bytes live in the package) |
| Opaque solid box | Rectangle | fill can be changed |
| Engine shadow-only (no blur) | native Rectangle; glow dropped | yes |
| Opaque linear gradient | Rectangle + `Gradient` fill | yes (live corners) |
| Translucent solid / stroke (`#RRGGBBAA`) | Rectangle + fill/stroke opacity | yes |
| blur / math | `k2f-raster:` Image | no |

## Coordinates

K2F lock: origin at the **page top-left**, **Y down**, millipt (`Pt.0 / 1000` = pt).

InDesign pasteboard: origin at the **spread center**, **Y down** (negative Y is toward the page top; verified against InDesign 2026 PDF export and the IDML cookbook `ty = -pageHeight/2` page-top rule). `PathPointType/@Anchor` is `"x y"`. `GeometricBounds` is `"top left bottom right"` (y x y x).

```text
idml_x = k2f_x_pt - page_w_pt / 2
idml_y = k2f_y_pt - page_h_pt / 2
```

Page items are children of `Spread`, not nested in `Page`. Each lock page is one single-page spread (`FacingPages=false`). `DocumentPreference/@Intent` is `WebIntent` so process RGB swatches stay RGB (PrintIntent converted them to CMYK on PDF). Linear gradients set `GradientFillStart` (page coordinates, Properties list) and `GradientFillLength` to the same AABB corner span as `k2f_paint`. `GradientFillAngle` is negated (K2F 90°=down, InDesign 90°=up).

## Limits (v1)

- Slight text reflow vs K2F is expected. Glyphs are not absolutely positioned.
- **Fonts:** the IDML ZIP still lists family names only (`Fonts.xml`, no TTF inside). Official export writes a sibling `Document Fonts/` folder (or a zip of that package) so InDesign can open without substituting. `--idml-only` skips the faces.
- Glass / blur slices sample only the chrome lock (page background plus the effect ops). They do **not** blur native card shapes sitting behind the glass. Translucent solids without blur are native opacity, so a hero scrim still shows the picture underneath.
- Engine shadows on otherwise-native boxes are dropped. An expanded opaque PNG of the glow halo covers earlier labels (invoice totals, raised plaques).
- Nested / image / still-Asset table cells are not native `Table` (they stay box+text+pic). Rows whose lock boxes do not share y/height (status pills, vertically centered day cards) or that combine rounded fills with column gaps also stay box+text: InDesign `Cell` fill cannot leave air. Invoice-style grids with a 4pt `gap` and square fills still harvest.
- SVG assets are rasterized to PNG at export. No IDML → K2F import. No `.indd`. No MathML.
- Tracking is omitted when advance cannot be measured; the writer does not fake `Tracking="0"`.

## Official CLI

From the `k2f/` workspace root:

```bash
cargo run -p k2f -- export-idml examples/published/invoice.K2F -o /tmp/invoice
```

```text
k2f export-idml <in.K2F> -o <out-dir>
k2f export-idml <in.K2F> -o <out.zip>
k2f export-idml <in.K2F> --idml-only -o <out.idml>
```

No `--scale`. No `--trust-pack`. Failures print the error (including `IDML_IS_NOT_A_SOURCE`) to stderr and exit `1` without writing output.

## Crate binary

`k2f-idml` is for crate development only. Same conversion as the official command:

```bash
cargo run -p k2f_idml -- export examples/published/invoice.K2F -o /tmp/invoice
```

```text
k2f-idml export <in.K2F> -o <out-dir>
k2f-idml export <in.K2F> --idml-only -o <out.idml>
```

`--help` states this is experimental, does not modify the source package, and is not a second layout engine.

## Test

From the `k2f/` workspace root:

```bash
cargo test -p k2f_idml
cargo run -p k2f -- export-idml examples/published/invoice.K2F -o /tmp/invoice
```

Do not use bare `cargo test` for this crate; it is not in `default-members`. OSS CI checks ZIP/XML only. Visual QA against Adobe InDesign is private (LibreOffice cannot open IDML).

## Delete this module

1. Remove the directory `k2f/export/k2f_idml/`
2. Remove `"export/k2f_idml"` from the `members` list in `k2f/Cargo.toml` (do not touch `default-members`)
3. Drop the `k2f_cli` path dependency, `ExportIdml` command, `tests/export_idml.rs`, and skill/changelog mentions of `export-idml`. SDK / WASM / desktop wiring is later; do not leave a dangling CLI command.
