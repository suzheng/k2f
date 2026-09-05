# Export surfaces

Load when the default `k2f export-pptx` path is not the one in use. All draw the published lock.

## CLI (default)

```bash
k2f export-pptx file.K2F -o out.pptx
```

Requires `pip install k2f` (or `cargo install k2f`). Refuses PPTX input (`PPTX_IS_NOT_A_SOURCE`). Draws the published lock; does not compile. Not a second layout engine; slight text reflow is allowed. No `--scale` / `--trust-pack`.
