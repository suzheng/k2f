# k2f_pptx

K2F lock → PPTX exporter. Official entry is `k2f export-pptx`. The crate binary `k2f-pptx` remains for crate development.

PPTX is a one-way dump of an already-locked package. It is not a K2F source (`PPTX_IS_NOT_A_SOURCE`). Coordinates come from the published lock; this is not a second layout engine. Slight text reflow vs K2F is expected.

## Capability table

| Input | PPT object | Editable |
|---|---|---|
| Ordinary text | text box | yes; slight reflow is allowed |
| Plain-text table | `a:tbl` | yes |
| Bitmap / SVG illustration | pic | replaceable |
| Opaque solid box | shape | fill can be changed |
| Translucent solid (`#RRGGBBAA`) | shape `a:solidFill` + `a:alpha` | fill can be changed |
| Shadowed opaque box | shape (fill + stroke; glow is a v1 gap) | fill can be changed |
| blur / glass / gradient / math | `k2f-raster` pic | no |

Do **not** stamp a full-slide PNG and overlay invisible text. That is the PDF-bridge stamp path and would destroy native text, tables, and pictures.

## Limits (v1)

- Slight text reflow vs K2F is expected. Glyphs are not absolutely positioned.
- Glass / blur slices sample only the chrome lock (page background plus the effect ops). They do **not** blur native card shapes sitting behind the glass.
- Nested / image / still-Asset table cells are not native `a:tbl` (they stay box+text+pic).
- Partial borders are drawn on all four sides. SVG assets are rasterized to PNG at export (same `decode_raster` / resvg path as paint).
- No PPT master, animation, OMML, or pptx → K2F import.

## Official CLI

From the `k2f/` workspace root (or any `k2f` on PATH):

```bash
k2f export-pptx examples/published/invoice.K2F -o /tmp/invoice.pptx
```

No `--scale` / `--trust-pack`. Failures print to stderr and exit `1` without writing output.

## Crate binary (development)

```bash
cargo run -p k2f_pptx -- export examples/published/invoice.K2F -o /tmp/invoice.pptx
```

```text
k2f-pptx export <in.K2F> -o <out.pptx>
```

`--help` states this is experimental, does not modify the source package, and is not a second layout engine.

## Test

From the `k2f/` workspace root:

```bash
cargo test -p k2f_pptx
cargo test -p k2f --test export_pptx
```

Do not use bare `cargo test` for this crate; it is not in `default-members`.

## Delete this module

1. Remove the directory `k2f/export/k2f_pptx/`
2. Remove `"export/k2f_pptx"` from the `members` list in `k2f/Cargo.toml` (do not touch `default-members`)
3. Revert `k2f export-pptx` wiring in `cli/k2f_cli`, this README, `CHANGELOG.md`, and `skills/k2f` (`export-pptx` / `exporting-pptx.md`)
