# Getting started

Create a packed `.K2F`. Two paths: ask an agent, or write the JSON and pack it yourself.

To look at a file first, open the [Playground](https://k2f.dev/playground) or a sample in the [Gallery](https://k2f.dev/gallery).

The native [desktop/k2f_reader in development](../../desktop/k2f_reader/README.md) opens locked packages offline from a clone:

```bash
cargo run -p k2f_reader -- examples/published/invoice.K2F
```

## Use an agent

```bash
npx skills add suzheng/k2f --skill k2f
pip install k2f
```

Works with Cursor, Claude Code, Codex, and other agents that support [`npx skills`](https://github.com/vercel-labs/skills). `pip install k2f` puts the CLI on PATH; the skill scripts call it. If `npx skills` is not available, copy [`skills/k2f/`](https://github.com/suzheng/k2f/tree/main/skills/k2f) into your agent skills path ([skills/README.md](../../skills/README.md)).

Then ask for a document, for example *Write a one-page monthly report.* The agent follows [K2F Skill](../../skills/k2f/SKILL.md).

## Write JSON yourself

### 1. Install the CLI

```bash
pip install k2f
```

Python 3.9+. This installs the `k2f` CLI and the Python SDK. Optional: `cargo install k2f` for a CLI without Python. Install `@openk2f/k2f` only when you [embed the web viewer](web-viewer.md).

### 2. Get the skill folder

Starter package, catalog shapes, and helper scripts live in [`skills/k2f/`](https://github.com/suzheng/k2f/tree/main/skills/k2f). Clone or copy that folder. Run the Python scripts from it (`k2f` must be on `PATH`).

### 3. Create a package

```bash
python scripts/init_package.py --workspace ./out/doc --title "Hello K2F" --page a4
```

Writes `./out/doc/source/` (empty `content/root.json`, starter theme, Roboto) and `./out/doc/tmp/` for preview PNGs.

### 4. Add content

Edit `source/content/root.json`. Keep `id: "root"` and paste catalog nodes into `children`. Do not replace `root.json` with a fragment. Start with [`ex_heading.json`](../../skills/k2f/catalog/content/ex_heading.json).

### 5. Pack and preview

```bash
python scripts/pack_verify.py ./out/doc/source -o ./out/doc/doc.K2F --render preview.png
```

Expect `UNSIGNED`. Open `./out/doc/tmp/preview.png`. If the bottom third of the PNG is blank paper, you are not done — see [visual check](../../skills/k2f/references/writing.md#visual-check).

Export from the lock (optional):

```bash
k2f export-pdf  ./out/doc/doc.K2F -o ./out/doc/doc.pdf
k2f export-pptx ./out/doc/doc.K2F -o ./out/doc/doc.pptx
k2f export-docx ./out/doc/doc.K2F -o ./out/doc/doc.docx
k2f export-idml ./out/doc/doc.K2F -o ./out/doc/doc
```

Guides: [Export](exporting.md) — [PDF](exporting-pdf.md) (fillable AcroForm), [PowerPoint](exporting-pptx.md), [Word](exporting-docx.md), [InDesign](exporting-idml.md).

### Existing file

```bash
k2f unpack existing.K2F -o ./out/doc/source
```

Do not pass `--include-lock`. Then the same edit / pack loop.

## Next

- Authoring: [Text](../authoring/text.md), [Images](../authoring/images.md), [Tables](../authoring/tables.md), [Layout](../authoring/layout.md), [Theme and fonts](../authoring/theme.md)
- Look up: [Catalog](../reference/catalog.md) · [Allowed keys](../reference/keys.md) · [Format spec](../spec/k2f-v0.3.md)
- [Markdown conversion](markdown.md) · [Export](exporting.md) · [Web viewer](web-viewer.md)
- Build from a clone: [Contributing](../../CONTRIBUTING.md)
