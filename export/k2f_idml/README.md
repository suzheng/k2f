# k2f_idml

Experimental K2F lock → Adobe InDesign `.idml` exporter. Isolated crate; **not** part of `k2f export-pdf` or the official `k2f` CLI yet.

IDML is a one-way dump of an already-locked package. It is not a K2F source (`IDML_IS_NOT_A_SOURCE`). Geometry comes from the published lock; this is not a second layout engine.

Step 4 writes harvestable inline text tables as native InDesign `Table`. Step 5 slices blur / glass / engine shadow / linear gradient / translucent solid / math into `k2f-raster:` PNG images (chrome lock + `render_lockfile_page_rgb`, no `k2f_pdf` stamp). Body text, tables, and content pictures stay native.

**Glass limitation (v1):** frost PNGs only blur chrome kept in the filtered lock (page background + that effect). They do not blur native cards that sit behind the glass in InDesign.

Fonts are **not** embedded: `Fonts.xml` lists ttf-parser family names only. If the target machine lacks that face, InDesign substitutes and visual QA will drift.

## Test

From the `k2f/` workspace root:

```bash
cargo test -p k2f_idml
cargo test -p k2f_idml --test step1_skeleton
cargo run -p k2f_idml -- export examples/published/invoice.K2F -o /tmp/invoice-step1.idml
```

Do not use bare `cargo test` for this crate; it is not in `default-members`.

## Crate binary

```text
k2f-idml export <in.K2F> -o <out.idml>
```

`--help` states this is experimental, does not modify the source package, and is not a second layout engine.

## Delete this module

1. Remove the directory `k2f/export/k2f_idml/`
2. Remove `"export/k2f_idml"` from the `members` list in `k2f/Cargo.toml` (do not touch `default-members`)
3. No other files should mention this crate
