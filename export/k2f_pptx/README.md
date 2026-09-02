# k2f_pptx (experimental)

Isolated K2F lock → PPTX exporter. This crate is **not** part of `k2f export-pdf` and is **not** wired into `k2f_cli`.

PPTX is a one-way dump of an already-locked package. It is not a K2F source (`PPTX_IS_NOT_A_SOURCE`). Coordinates come from the published lock; this is not a second layout engine.

## Capability table

| Input | PPT object | Editable |
|---|---|---|
| Ordinary text | text box | yes; slight reflow is allowed |
| Plain-text table | `a:tbl` | yes |
| Bitmap / SVG illustration | pic | replaceable |
| Opaque solid box | shape | fill can be changed |
| blur / glass / shadow / gradient / translucency / math | `k2f-raster` pic | no |

Do **not** stamp a full-slide PNG and overlay invisible text. That is the PDF-bridge stamp path and would destroy native text, tables, and pictures.

## Limits (v1)

- Slight text reflow vs K2F is expected. Glyphs are not absolutely positioned.
- Glass / blur slices sample only the chrome lock (page background plus the effect ops). They do **not** blur native card shapes sitting behind the glass.
- Nested / image / still-Asset table cells are not native `a:tbl` (they stay box+text+pic).
- Partial borders are drawn on all four sides. SVG embeds as `image/svg+xml`.
- No PPT master, animation, OMML, or pptx → K2F import.
- No `k2f export-pptx` CLI flag. Use this crate's binary only.

## Test

From the `k2f/` workspace root:

```bash
cargo test -p k2f_pptx
```

Do not use bare `cargo test`; this crate is not in `default-members`.

## CLI

```bash
cargo run -p k2f_pptx -- export examples/published/invoice.K2F -o /tmp/invoice.pptx
```

```text
k2f-pptx export <in.K2F> -o <out.pptx>
```

`--help` states this is experimental, does not modify the source package, and is not a second layout engine. Failures print to stderr and exit `1`.

## Delete this module

1. Remove the directory `k2f/export/k2f_pptx/`
2. Remove `"export/k2f_pptx"` from the `members` list in `k2f/Cargo.toml` (do not touch `default-members`)
3. No other crate should mention `k2f_pptx`
