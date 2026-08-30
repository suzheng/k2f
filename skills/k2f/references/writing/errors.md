# Writing error codes

Fix the **semantic tree** or **theme**. Shapes: [`catalog/content/ex_*.json`](../../catalog/content/).

## AgentError `code` values

| Code | Fix |
|------|-----|
| `UNKNOWN_ROLE` | Add role (and variant if used) to `styles/theme.json` |
| `DUPLICATE_ID` | Rename; ids must be globally unique |
| `INVALID_ID` | Pattern `segment.segment` — alphanumeric + underscore |
| `TABLE_ROW_MISMATCH` | Every row same length as columns; header_rows ≤ rows |
| `IMAGE_SIZE` | Provide image bytes + declared width/height millipt. Formats: PNG, WebP, SVG — not JPEG |
| `FONT_MISSING` | Embed TTF/OTF under `assets/fonts/`; map role `font_family` via `font_aliases` to the file **stem** (e.g. `"DejaVuSans"`). One font auto-registers as `"default"`; with two+ fonts you must alias explicitly. No CSS generic families. |
| `INVALID_MODIFIER` | Range on UTF-8 **byte** boundaries (`python scripts/modifier_range.py --text … --find …`); max 50 modifiers. For multi-line titles, pass the full `value` including `\n` (`\n` = 1 byte). |
| `SCHEMA_INVALID` | Open [`schema/`](../../schema/) for the failing file. Key in schema but rejected → `pip install -U k2f`. Key not in schema → remove it. Optional layout keys may be omitted or `null`. |
| `UNKNOWN_ID` | Id not in tree — search outline; do not guess |
| `WRONG_CONTENT` | Wrong content type for the edit (e.g. text API on a table) |
| `UNEXPECTED_PATH` | Extra ZIP path (e.g. `schema/agent_v0.schema.json`) — remove it; assets must live only under `assets/fonts/`, `assets/images/`, or `assets/data/` |
| `UNKNOWN_PRIMITIVE` | `box_decoration` must name an existing primitive |
| `FONT_MISSING_GLYPH` | Glyph not in embedded faces — change text, subset, or add a font; no OS font fallback. Bundled Roboto lacks arrows (`→` `↗` `↑`), dingbats (★ ◆), and **CJK** → ASCII/`->`, SVG, English rewrite, or covering `--font` |
| `INVALID_ARGUMENT` | Catch-all for many compile failures — read **message** |
| `PDF_IS_NOT_A_SOURCE` | Do not import PDF as K2F source |
| `UNSPLITTABLE_OVERFLOW` | Unsplittable node (e.g. `break_inside: avoid`, overlay, grid) taller than one page content box. Often `layout.height` shell + role padding + a child also sized to full page — shrink padding/gap or size children to the inner box |

## Math messages (not separate codes)

Compile may include `MATH_UNSUPPORTED:`, `MATH_PARSE:`, or `MATH_MISSING_GLYPH:` in the **message**. The mapped `code` is usually `INVALID_ARGUMENT`.

Display math requires `role: "math"` — custom roles with `content.type = "math"` fail compile. Use `variant` on the `math` role for visual skins. Example: `{ "id": "eq.1", "role": "math", "content": { "type": "math", "value": "\\frac{a}{b}" } }`.

Inline math in author JSON: U+FFFC placeholder + `{ "type": "math", "intent": "<tex>" }` on the text node — not `$...$` in JSON. See `catalog/content/ex_modifiers.json`.

### TeX subset (unknown command → `MATH_UNSUPPORTED`)

- **Structure:** `\frac`, `\sqrt`, sub/sup scripts, `\left`/`\right`, `matrix` / `pmatrix` / `bmatrix` / `align` / `cases`
- **Greek:** `\alpha` … `\omega`, `\Gamma` … `\Omega` (incl. `\zeta`, `\varepsilon`, `\varphi`, etc.)
- **Ops:** `\sum`, `\prod`, `\int`; `\sin`, `\cos`, `\tan`, `\log`, `\ln`, `\exp`, `\lim`, `\max`, `\min`
- **Symbols:** `\infty`, `\partial`, `\nabla`, `\pm`, `\times`, `\cdot`, `\leq`, `\geq`, `\neq`, `\approx`, `\equiv`, `\in`, `\rightarrow`, `\Rightarrow`, `\ldots`, `\cdots`, etc.
- **Spacing:** `\,` `;` `\quad` `\qquad`

Grid `fr` rows in unbounded height fail with `Cannot resolve fr tracks with infinite available size` — set the grid's own `layout.height`, use `pt` rows, or nest under a fixed-height stack/overlay. `fr` is **not** content-auto sizing. See `ex_grid.json`.

## Integrity after pack

`k2f compile` / `verify` stderr include `pages=N` (multi-page is legal). For single-page posters use `pack_verify.py --expect-pages 1`. `k2f verify` may report `UNSIGNED` (normal for agent output), `VALID`, or broken-hash banners. Relock with `save` / `compile` after content or theme changes.
