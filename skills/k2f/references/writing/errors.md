# Writing error codes

Fix the **semantic tree** or **theme**. Shapes: [`catalog/content/ex_*.json`](../../catalog/content/).

## AgentError `code` values

| Code | Fix |
|------|-----|
| `UNKNOWN_ROLE` | Add role (and variant if used) to `styles/theme.json` |
| `DUPLICATE_ID` | Rename; ids must be globally unique |
| `INVALID_ID` | Pattern `segment.segment` — alphanumeric + underscore |
| `TABLE_ROW_MISMATCH` | Every row same length as columns; header_rows ≤ rows; table `column_widths` cannot be `{auto:true}` |
| `IMAGE_SIZE` | Provide image bytes + declared width/height millipt. Formats: PNG, JPEG, WebP, SVG — not GIF. SVG `<text>` / `<tspan>` / `<textPath>` / `<foreignObject>` fail at **compile** (paint still fail-closed) — convert labels to `<path>`. Mentions inside XML comments or CDATA do not count. |
| `FONT_MISSING` | Embed TTF/OTF under `assets/fonts/`; map role `font_family` via **`styles/theme.json` `font_aliases`** (not manifest) to the file **stem** (e.g. `"DejaVuSans"`). One **face** auto-registers as `"default"`; license/sidecar `.txt` under `fonts/` is packed but ignored at load (not a second face). With two+ faces you must alias **every** stem and retarget role `font_family` (see `catalog/styles/theme.json`). `--add-font` adds the alias only. No CSS generic families. Invalid `.ttf`/`.otf` → `FONT_INVALID` (path in the message), not a parser dump. |
| `INVALID_MODIFIER` | Range on UTF-8 **byte** boundaries (`python scripts/modifier_range.py --text … --find …`); max 50 modifiers. For multi-line titles, pass the full `value` including `\n` (`\n` = 1 byte). Missing `intent` is `SCHEMA_INVALID`, not this code. |
| `SCHEMA_INVALID` | Open [`schema/`](../../schema/) for the failing file. Key in schema but rejected → `pip install -U k2f`. Key **not** in schema → remove it (`colspan`, `space-between`, node-level `box_decoration`/`padding`/`text_align` are not keys). Modifier objects need `range`, `type`, and `intent`. Optional layout keys (including grid `rows`) may be omitted or `null`. |
| `UNKNOWN_ID` | Id not in tree — search outline; do not guess |
| `WRONG_CONTENT` | Wrong content type for the edit (e.g. text API on a table) |
| `UNEXPECTED_PATH` | Extra ZIP path (e.g. `schema/agent_v0.schema.json`) — remove it; assets must live only under `assets/fonts/`, `assets/images/`, or `assets/data/` |
| `UNKNOWN_PRIMITIVE` | `box_decoration` must name an existing primitive |
| `FONT_MISSING_GLYPH` | Glyph not in **any** embedded face — add a covering font (`init_package.py --add-font`), or change that character. No OS fallback. Roboto lacks arrows, dingbats, math (`∈` `∑`), and **CJK/kana**. Formulas (`role: "math"`, inline math modifiers, or math glyphs in body) need **NotoSansMath** from [`catalog/assets/fonts/`](../../catalog/assets/fonts/) plus `font_aliases` (starter is Roboto only). NotoSansSC ≠ Japanese. Do not rewrite the user's language to English. |
| `INVALID_ARGUMENT` | Catch-all for many compile failures — read **message** |
| `PDF_IS_NOT_A_SOURCE` | Do not import PDF as K2F source |
| `UNSPLITTABLE_OVERFLOW` | Unsplittable node (e.g. `break_inside: avoid`, overlay, grid, **padded columns**) taller than one page content box. Often `layout.height` shell + role padding + a child also sized to full page, or a page-height `avoid` **stack** with large `padding_pt` — copy `ex_poster_shell.json` (`page_shell`); `inner_h = height − pad_t − pad_b`; do not re-declare page height on children. Article columns: unpadded `columns` + `column_span: "all"`, not a padded role |

## Compile stderr (not AgentError)

`k2f compile` may print `LAYOUT_SLACK id=… unused_below=… (%) … after=…`. Warning only (exit 0). A large stretched box is empty at the bottom — usually a grower, not the shell. `{fr:1}` grows the **box**; text stays top-packed. Put leftover on a figure or dense siblings ([`ex_poster_growers.json`](../../catalog/content/ex_poster_growers.json)), not a 2-line card. Silence does not mean interiors are filled — open the PNG. Not stored in the lock.

## Math messages (not separate codes)

Compile may include `MATH_UNSUPPORTED:`, `MATH_PARSE:`, or `MATH_MISSING_GLYPH:` in the **message**. The mapped `code` is usually `INVALID_ARGUMENT`.

Display math requires `role: "math"` — custom roles with `content.type = "math"` fail compile. Use `variant` on the `math` role for visual skins. Example: `{ "id": "eq.1", "role": "math", "content": { "type": "math", "value": "\\frac{a}{b}" } }`.

Inline math in author JSON: U+FFFC placeholder + `{ "type": "math", "intent": "<tex>" }` on the text node — not `$...$` in JSON. See `catalog/content/ex_modifiers.json`.

### TeX subset (unknown command → `MATH_UNSUPPORTED`)

This is a **whitelist**. Commands not listed fail; there is no full-LaTeX mode.

- **Structure:** `\frac`, `\sqrt` (not `\sqrt[n]`), sub/sup scripts, `\left`/`\right`, `matrix` / `pmatrix` / `bmatrix` / `align` / `cases`
- **Greek:** `\alpha` … `\omega`, `\Gamma` … `\Omega` (incl. `\zeta`, `\varepsilon`, `\varphi`, etc.)
- **Ops:** `\sum`, `\prod`, `\int`; `\sin`, `\cos`, `\tan`, `\log`, `\ln`, `\exp`, `\lim`, `\max`, `\min`
- **Symbols:** `\infty`, `\partial`, `\nabla`, `\hbar`, `\ell`, `\emptyset`, `\forall`, `\exists`, `\prime`, `\pm`, `\times`, `\cdot`, `\leq`, `\geq`, `\neq`, `\approx`, `\equiv`, `\in`, `\rightarrow`, `\Rightarrow`, `\ldots`, `\cdots`, `\langle`/`\rangle` (also as `\left\langle`), `\lfloor`/`\rfloor`, etc.
- **Alphabet:** `\mathbb{R}` / `\mathcal{L}` map letters to Unicode double-struck / script (NotoSansMath required)
- **Upright words:** `\text{...}`, `\mathrm{...}`
- **Spacing:** `\,` `;` `\quad` `\qquad`
- **Not in subset:** `\color` / `\textcolor` (use the `math` role color); `\tag` / `\label` / `\ref` (copy [`ex_math_numbered.json`](../../catalog/content/ex_math_numbered.json) — 2-col `{fr:1}` + `{auto:true}`); `\mathbf` / `\mathit` / `\textbf`; `\dfrac` / `\displaystyle`; `\sqrt[n]`; `\hat` / `\vec` / `\overline` / `\underline` / `\over`; `\def` / `\newcommand`. Rewrite the formula to the whitelist; do not add engine commands.

Display math also needs NotoSansMath embedded (see `FONT_MISSING_GLYPH` above). Starter is Roboto only.

Grid `fr` rows in unbounded height fail with `Cannot resolve fr tracks with infinite available size` — set the grid's own `layout.height`, use `pt`/`auto` rows, or nest under a fixed-height stack/overlay. `fr` is **not** content-auto sizing (`auto` is). See `ex_grid.json` / `ex_poster_shell.json`. Table `column_widths` cannot use `auto`.

## Integrity after pack

`k2f compile` / `verify` stderr include `pages=N` (multi-page is legal). For single-page posters use `pack_verify.py --expect-pages 1`. `k2f verify` may report `UNSIGNED` (normal for agent output), `VALID`, or broken-hash banners. Relock with `save` / `compile` after content or theme changes.
