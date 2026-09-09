# Author sources and themes

Published SDKs do **not** bundle official templates. Create an author directory with the agent skill (`init_package.py` + [`skills/k2f/starter/`](../../skills/k2f/starter/)), unpack an existing `.K2F`, or download a Gallery package. Then edit JSON and pack — or open that directory / those bytes in the SDK.

The `templates/` tree in this repository is a **contributor fixture** only (invoice, legal, report, …). It is not on PyPI, npm, or crates.io.

## SDK

```python
import json
import k2f

ed = k2f.Editor.open_dir("./source")  # unpack or init_package.py output
# or: ed = k2f.Editor.open_bytes(open("doc.K2F", "rb").read())

ed.insert_node("root", 0, json.dumps({
    "id": "doc.title",
    "role": "h1",
    "content": {"type": "text", "value": "Independent Contractor Agreement"},
}))
blob = bytes(ed.save_bytes())
```

```javascript
const k2f = await createK2f();
const ed = k2f.Editor.open(packageBytes); // packed .K2F — no named templates
const bytes = k2f.markdownToK2f(md, { title: "Notes", templateBytes: packageBytes });
```

Python/CLI Markdown takes an **author directory** (fonts + `styles/theme.json`):

```bash
python3 skills/k2f/scripts/init_package.py --workspace ./out/doc --title "From Markdown" --page a4
k2f markdown README.md -o ./out/doc/doc.K2F --template ./out/doc/source
k2f markdown docs/guide -o target/md-k2f/guide --template ./out/doc/source --font path/to/subset.otf
```

Custom roles, grid/stack, or fixed visual layout (CV, flyer, slide decks) use the agent skill: copy [`skills/k2f/starter/`](../../skills/k2f/starter/) with `init_package.py`, copy shapes from [`skills/k2f/catalog/content/ex_*.json`](../../skills/k2f/catalog/content/), look up allowed keys in [`skills/k2f/schema/`](../../skills/k2f/schema/), then pack. See [writing/package.md](../../skills/k2f/references/writing/package.md).

K2F never uses system fonts: missing glyphs fail closed (`FONT_MISSING_GLYPH`). When the primary face lacks a glyph, the layout engine tries other embedded package fonts before failing.

`k2f pack`, `Editor.save_bytes`, and `k2f compile` coverage-subset large CJK faces in the **package** (not the author directory): CJK ideographs shrink to GB2312 ∪ Big5 level 1 ∪ JIS X 0208, and every non-Han glyph (Latin, kana, hangul, punctuation) is kept. Faces that would not drop any Han stay byte-identical. A Han outside that union still fails closed. Author dirs may keep a full face for rare-character work; the `.K2F` ZIP does not.

To convert Markdown that needs other scripts:

1. `python3 scripts/md-glyph-report.py path/to/docs` — list missing `U+XXXX` vs SC ∪ emoji.
2. `python3 scripts/md-to-k2f.py path/to/docs -o target/md-k2f` — compile when SC ∪ emoji covers the file; otherwise subset the first covering font in [`scripts/font-catalog.json`](../../scripts/font-catalog.json) and pass it with `--font`.
3. Add a catalog row when a real document needs another script.

## Not supported

Private theme-sketch JSON pairs (kept outside this public repo) are visual explorations only. They are **not** supported templates and must not be referenced in agent instructions or public docs.
