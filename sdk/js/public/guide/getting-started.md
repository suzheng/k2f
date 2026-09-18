# First document

Install the toolchain, create an author directory, edit JSON like source, and pack a `.K2F`.

## Install

```bash
pip install k2f
```

Python 3.9+. Optional: `npm i @openk2f/k2f` for the [web viewer](../../skills/k2f/references/embedding-viewer.md); `cargo install k2f` for a CLI without Python.

You also need the **skill folder** (starter package, catalog shapes, `init_package.py` / `pack_verify.py`):

```bash
npx skills add suzheng/k2f --skill k2f
```

Or clone [`skills/k2f/`](https://github.com/suzheng/k2f/tree/main/skills/k2f) from the repository. Run the Python scripts from that directory (`k2f` must be on `PATH`).

## Create a package

```bash
python scripts/init_package.py --workspace ./out/doc --title "Hello K2F" --page a4
```

That writes `./out/doc/source/` (empty `content/root.json`, starter theme, Roboto) and `./out/doc/tmp/` for preview PNGs.

Edit `source/content/root.json`. Keep `id: "root"` and paste catalog nodes into `children` — do not replace `root.json` with a fragment. Start with [`ex_heading.json`](../../skills/k2f/catalog/content/ex_heading.json).

```bash
python scripts/pack_verify.py ./out/doc/source -o ./out/doc/doc.K2F --render preview.png
```

Expect `UNSIGNED`. Open `./out/doc/tmp/preview.png`. Empty paper in the lower third of a designed sheet means you are not done — see the skill [writing loop](../../skills/k2f/references/writing.md).

Existing file: `k2f unpack existing.K2F -o ./out/doc/source` (do not pass `--include-lock`), then the same edit / pack loop.

## Next

- [Text](../authoring/text.md), [Images](../authoring/images.md), [Tables](../authoring/tables.md), [Layout](../authoring/layout.md), [Theme and fonts](../authoring/theme.md)
- [Catalog](../../skills/k2f/catalog/README.md) · [Allowed keys](../../skills/k2f/references/writing/fields.md) · [Format spec](../spec/k2f-v0.1.md)
- [Markdown conversion](../../skills/k2f/references/converting-markdown.md)

## Open a document

Serve files over **HTTP** (WASM does not load from `file://`). Embed with the [web viewer](../../skills/k2f/references/embedding-viewer.md).

**Native reader** ([k2f_reader](../../desktop/k2f_reader/README.md), in development) — same lock-executor rules as the web viewer. Packaged builds: website `/download`. From a clone:

```bash
cargo run -p k2f_reader -- examples/published/invoice.K2F
```

## Build from this repository

For contributors, golden tests, and the smoke script.

```bash
git clone https://github.com/suzheng/k2f.git
cd k2f
bash scripts/dev-install.sh
bash scripts/test-getting-started.sh
```

`dev-install.sh` creates a Python venv under `sdk/python/.venv`, builds the `k2f` Python package with maturin, and compiles the JS WASM bindings. The smoke script writes a package, exports PDF, confirms `PDF_IS_NOT_A_SOURCE`, and runs `k2f verify`.

On Windows, run the bash scripts in Git Bash or WSL. Rust is only required for this path.

See [Contributing](../../CONTRIBUTING.md) and [project status](status.md).
