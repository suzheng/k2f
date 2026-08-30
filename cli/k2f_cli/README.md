# k2f

Command-line tool for packing, compiling, verifying, signing, rendering, and converting K2F documents.

**License:** Apache-2.0

## Install

```bash
pip install k2f    # preferred: CLI on PATH + Python SDK
# cargo install k2f   # optional: CLI without Python
```

## Commands

| Command | Description |
|---------|-------------|
| `k2f pack <source> -o <output.K2F>` | Pack a source directory into a `.K2F` ZIP |
| `k2f compile <package.K2F>` | Compile semantic tree → `document.K2F.lock` in place |
| `k2f verify <package.K2F>` | Inspect integrity banners and hashes |
| `k2f keygen -o <secret.key>` | Generate Ed25519 signing key (hex) |
| `k2f sign <package> --key <secret.key>` | Sign package (`--signed-by`, `--signed-at` optional) |
| `k2f render <package> -o <page.png> [--page N] [--scale N]` | Rasterize one lock page to PNG |
| `k2f export-pdf <package> -o <out.pdf> [--scale N] [--trust-pack]` | Draw lock into PDF |
| `k2f hit-test <package> --page N --x X --y Y` | Hit-test lock geometry (pt, not px) |
| `k2f markdown <source.md> -o <out.K2F> [--theme report]` | Compile Markdown → `.K2F` |

Coordinates for `hit-test` are in **points** on the page, matching the lock executor.

## Examples

```bash
k2f compile examples/published/invoice.K2F
k2f verify examples/published/invoice.K2F
k2f export-pdf examples/published/invoice.K2F -o /tmp/invoice.pdf
k2f markdown README.md -o /tmp/readme.K2F --theme report
```

See the [repository README](https://github.com/suzheng/k2f#readme) and [Getting Started](https://github.com/suzheng/k2f/blob/main/docs/guide/getting-started.md).
