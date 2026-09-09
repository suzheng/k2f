# k2f_docx

K2F lock → DOCX exporter. Official entry is `k2f export-docx`. The crate binary `k2f-docx` remains for crate development.

DOCX is a one-way dump of an already-locked package. It is not a K2F source (`DOCX_IS_NOT_A_SOURCE`). Paper size and page breaks come from the published lock. This is not a flowing “Save as Word” layout engine and not “generate a .docx from an outline”: objects are placed with lock coordinates (absolute DrawingML), not Word’s automatic reflow.

If Word’s “Show editing marks” is on, the file looks like many floating text boxes. That is expected.

## Capability table

| Input | Word object | Editable |
|---|---|---|
| Ordinary text | `wps:txbx` text box | yes; slight reflow is allowed |
| Running header/footer | header/footer part + `PAGE` / `NUMPAGES` fields | yes (edit the part, every page changes) |
| Plain-text table | `w:tbl` inside a positioned text box | yes |
| Bitmap / SVG illustration | `pic:pic` | replaceable |
| Opaque solid box | `wps:wsp` | fill can be changed |
| Linear gradient / translucent solid | `wps:wsp` `a:gradFill` / `a:alpha` | fill can be changed |
| blur / shadow / math | `k2f-raster:` `wps:wsp` + `a:blipFill` | no |

Do **not** stamp a full-page PNG and overlay invisible text. That is the PDF-bridge stamp path and would destroy native text, tables, and pictures.

## Limits (v1)

- Slight text reflow vs K2F is expected. Glyphs are not absolutely positioned.
- Glass / blur slices sample only the chrome lock (page background plus the effect ops). They do **not** blur native card shapes sitting behind the glass.
- Nested / image / still-Asset table cells are not native `w:tbl` (they stay box+text+pic).
- Do not restyle with Word’s Heading 1 style gallery: that would reflow the absolutely positioned contract.
- SVG embeds as `image/svg+xml`. Word’s SVG support is weaker than PowerPoint; CI only asserts media + `a:blip`.
- No DOCX → K2F import. No OMML.

## Official CLI

From the `k2f/` workspace root (or any `k2f` on PATH):

```bash
k2f export-docx examples/published/invoice.K2F -o /tmp/invoice.docx
```

No `--scale` / `--trust-pack`. Failures print to stderr and exit `1`. Slight text reflow vs K2F is legal.

## Crate binary (development)

```bash
cargo run -p k2f_docx -- export examples/published/invoice.K2F -o /tmp/invoice.docx
```

```text
k2f-docx export <in.K2F> -o <out.docx>
```

`--help` states this is experimental, does not modify the source package, and is not a second layout engine. Failures print to stderr and exit `1`.

## Test

From the `k2f/` workspace root:

```bash
cargo test -p k2f_docx
cargo test -p k2f --test export_docx
```

Do not use bare `cargo test` for this crate; it is not in `default-members`.

## Relationship with `k2f_pptx`

Independent crate. Do not `use k2f_pptx`. Do not extract a shared `k2f_office` crate. Deleting Word export must not touch `export/k2f_pptx`.

## Delete this module

1. Remove the directory `k2f/export/k2f_docx/`
2. Remove `"export/k2f_docx"` from the `members` list in `k2f/Cargo.toml` (do not touch `default-members`)
3. Revert `k2f export-docx` wiring in `cli/k2f_cli`, this README, `CHANGELOG.md`, and `skills/k2f` (`export-docx` / `exporting-docx.md`)
