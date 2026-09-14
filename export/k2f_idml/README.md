# k2f_idml

Experimental K2F lock → Adobe InDesign `.idml` exporter. Isolated crate; **not** part of `k2f export-pdf` or the official `k2f` CLI yet.

IDML is a one-way dump of an already-locked package. It is not a K2F source (`IDML_IS_NOT_A_SOURCE`). Geometry comes from the published lock; this is not a second layout engine.

Step 3 writes opaque solid `DrawBox` as native `Rectangle` and `DrawImage` as embedded `Image` (base64 Contents, no `file:` links). Text frames from step 2 stay. Tables, blur, gradients, translucent fills, engine shadows, and formulas are still skipped.

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
