# Export surfaces

Load when the default `k2f export-docx` path is not the one in use. All draw the published lock.

## CLI (default)

```bash
k2f export-docx file.K2F -o out.docx
```

Requires `pip install k2f` (or `cargo install k2f`). Refuses DOCX input (`DOCX_IS_NOT_A_SOURCE`). Draws the published lock; does not compile. Not a flowing “Save as Word” layout engine; slight text reflow is allowed. No `--scale` / `--trust-pack`.

## Desktop reader (not the agent default)

`k2f-reader --export-docx out.docx file.K2F` — prefer `k2f export-docx`. HUD format cycle includes **DOCX**; Export writes the same lock bytes.
