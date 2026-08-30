# Official templates

Agents start from an **official template directory** at [`k2f/templates/`](../../templates/). Each template is a complete author source: `manifest.json`, `content/root.json`, `styles/theme.json`, embedded fonts, and optional starter content. Templates are not hand-written at runtime — copy or open them, then edit by node id.

| Template | Directory | Font | Typical use |
|----------|-----------|------|-------------|
| Blank | `blank` | Roboto | Empty starting point |
| Legal | `legal` | Noto Sans SC | Contracts, agreements (CN/EN) |
| Invoice | `invoice` | Roboto | Statements, invoices |
| Clinical summary | `clinical_summary` | Noto Sans SC | Medical / clinical reports |
| Report | `report` | Noto Sans SC | General formal documents |

```python
import json
import k2f

k2f.copy_template("invoice", "./my-invoice")
ed = k2f.Editor.open_dir("./my-invoice")

ed = k2f.Editor.open_template("legal")
ed.insert_node("root", 0, json.dumps({
    "id": "contract.title",
    "role": "h1",
    "content": {"type": "text", "value": "Independent Contractor Agreement"},
}))
blob = bytes(ed.save_bytes())
```

```javascript
const k2f = await createK2f();
await k2f.copyTemplate("report", "./my-report");
const ed = k2f.Editor.openTemplate("invoice");
const templates = k2f.officialTemplates(); // ["blank", "invoice", ...]
const bytes = k2f.markdownToK2f(md, { title: "Notes", template: "report" });
```

List templates with `official_templates()` (Python) or `officialTemplates()` (JS). Resolve a bundled path with `resolve_template("invoice")` / `resolveTemplate("invoice")`.

Fonts are embedded in every template. The committed Noto Sans SC file is a **demo subset** (Basic Latin plus the CJK/punctuation needed by in-repo fixtures). K2F never uses system fonts: missing glyphs fail closed (`FONT_MISSING_GLYPH`). When the primary face lacks a glyph, the layout engine tries other embedded package fonts before failing.

To convert Markdown that needs other scripts:

1. `python3 scripts/md-glyph-report.py path/to/docs` — list missing `U+XXXX` vs SC ∪ emoji.
2. `python3 scripts/md-to-k2f.py path/to/docs -o target/md-k2f` — compile when SC ∪ emoji covers the file; otherwise subset the first covering font in [`scripts/font-catalog.json`](../../scripts/font-catalog.json) and pass it to `k2f markdown --template report --font …`.
3. Add a catalog row when a real document needs another script.

```bash
k2f markdown README.md -o readme.K2F --template report
k2f markdown docs/guide -o target/md-k2f/guide --template report --font path/to/subset.otf
```

Custom roles, grid/stack, or fixed visual layout (CV, flyer, slide decks) use the agent skill: copy [`skills/k2f/starter/`](../../skills/k2f/starter/) with `init_package.py`, copy shapes from [`skills/k2f/catalog/content/ex_*.json`](../../skills/k2f/catalog/content/), look up allowed keys in [`skills/k2f/schema/`](../../skills/k2f/schema/), then pack. See [writing/package.md](../../skills/k2f/references/writing/package.md).

## Not supported

Private theme-sketch JSON pairs (kept outside this public repo) are visual explorations only. They are **not** supported templates and must not be referenced in agent instructions or public docs.
