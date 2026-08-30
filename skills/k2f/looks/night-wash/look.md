# night-wash

This file plus `theme.json` is an **agent example**, not a drop-in package. If the user named a style, ignore this look. Otherwise rewrite the author `styles/theme.json` from this theme (adapt; do not ship it unchanged).

Dark, cinematic layout: deep charcoal base with a **single linear wash** (`canvas_wash` gradient). Type is sparse and large; secondary text stays muted gray-green. Best for launch slides, keynote one-pagers, and premium posters. **Gradients trigger the PDF stamp path** — note this when exporting PDF.

## 1. Character

Stage lighting, not office document. Background carries mood; foreground carries only what matters. Few words, strong hierarchy. Accent green (`#7EE787`) marks kickers, links, and metrics — use once per view.

## 2. Surfaces and color

| Token | Hex | Use |
|-------|-----|-----|
| `paper` / gradient base | `#0D1117` → `#1F2937` | `document` + `shell` via `canvas_wash` |
| `ink` | `#E6EDF3` | Primary headings, display |
| `ink_soft` | `#8B949E` | Body on dark |
| `muted` | `#8B949E` | Captions, running text |
| `accent` | `#7EE787` | Kicker, metric, `h2`, links |
| `glass_fill` | `#FFFFFF14` | Card fill |
| `signature` | `#161B22` | Code, inset panels |

Cards use translucent fill + `border: glass` — not heavy shadows. Keep body text in `ink_soft`, not pure white, for long lines.

**PDF note:** Linear gradient backgrounds require the PDF export stamp path (see [exporting-pdf.md](../../references/exporting-pdf.md)). Expect slightly larger PDFs. For print-first jobs with no gradients, prefer [`quiet-light`](../quiet-light/look.md).

## 3. Type ramp (millipt = 1/1000 pt)

### Report (A4) — use sparingly

Dark long-form is hard to read. If the user insists:

| Role | Size | Color |
|------|------|-------|
| `h1` | 24000 | `ink` |
| `body` | 11000 | `ink_soft` |
| `caption` | 9000 | `muted` |

Prefer switching to `quiet-light` for multi-page reports unless the brief says dark.

### Slide deck (16:9)

| Role | Size | Notes |
|------|------|-------|
| `display` | 36000 | White, one line |
| `kicker` | 10000 | Accent green |
| `body` | 12000 | `ink_soft`, max 4 lines |
| `metric` | 52000 | Accent green |

### Poster

| Role | Variant | Size |
|------|---------|------|
| `display` | `poster` | 60000 |
| `metric` | `poster` | 78000 |
| `kicker` | `poster` | 12000 |

## 4. Space

| Context | `shell` padding | Stack `gap` |
|---------|-----------------|-------------|
| Poster | 44000–52000 | 28000 (more air than quiet-light) |
| Slide | 44000 | 20000 |
| Report | 48000 | 12000 |

Dark layouts need **extra vertical gap** — crowding looks muddy on gradient backgrounds.

## 5. Composition recipes

### Poster

```bash
python scripts/init_package.py --dir ./out/poster --title "…" --page a4 --margin 0
# rewrite styles/theme.json from this look's theme.json (adapt — do not ship unchanged)
```

1. Page background comes from `document`/`shell` `canvas_wash` — no extra background node unless layering a photo.
2. Copy [`ex_poster_shell.json`](../../catalog/content/ex_poster_shell.json): single `shell`, fixed height, `break_inside: "avoid"`, `{fr:1}` grower. Do not use a vertical stack as the page shell.
3. Header row: `kicker` (accent) → `display` variant `poster`. Grower: optional one `body` line (`ink_soft`) and/or `metric` variant `poster`.
4. Supporting points in the grower: max 3 `card` variant `glass` — 3-column grid, or two cells for 1–2 items.
5. Footer `{auto:true}` row: `caption` in `muted`.
6. Photo hero: wrap the shell in `overlay` with full-bleed `image`; keep text high contrast (`card` variant `glass` behind text if needed).

### Slide deck

```bash
python scripts/init_package.py --dir ./out/deck --title "…" --page widescreen --margin 0
```

1. Each slide: copy [`ex_poster_shell.json`](../../catalog/content/ex_poster_shell.json), `shell` `960000×540000`, gradient inherited. Main idea in `{fr:1}`.
2. **Title slide:** `kicker`, `display` in the header row; `caption` date in the footer row.
3. **Statement slide:** one `display` line in the grower (or header); keep copy short.
4. **Detail slide:** `display` in the header row + 3 `list_item` in `ink_soft` in the grower — no walls of text.
5. **Metric slide:** `metric` + `caption` in the grower.
6. Max ~12 words of `body` per slide besides bullets.

### Report (short)

Only for briefs ≤5 pages. Same as quiet-light flow but:

- `h1` in `ink`, `body` in `ink_soft`
- Tables use dark `header_bg`
- No light-paper assumptions in running headers

## 6. Images

| Intent | Pattern |
|--------|---------|
| Cinematic hero | Full-bleed `image` under `overlay`; dim with nested `card` variant `glass` behind text |
| Product shot | Centered `image` between `display` and `caption` |
| Logo | Monochrome/light SVG preferred; place in `kicker` row |

Avoid busy photos behind small body text without a glass panel.

## 7. Don'ts

- No white/off-white page background — wrong look; use [`quiet-light`](../quiet-light/look.md).
- No rainbow color-block grid — use [`vivid-blocks`](../vivid-blocks/look.md).
- No long gray paragraphs on dark (use `ink_soft`, keep ≤3 sentences per node).
- No multiple accent colors — one green accent only.
- No second gradient direction on the same page.
- Do not use shadows as the main depth cue — use glass border/fill.
- Do not assume PDF exports without stamp when gradient is present.

## 8. Mapping table

| User says | Role | Notes |
|-----------|------|-------|
| Keynote title | `display` | variant `poster` on posters |
| Event tag / "Introducing" | `kicker` | Accent green |
| Supporting sentence | `body` | `ink_soft`, one line preferred |
| Bullet | `list_item` | Short phrases |
| Big number | `metric` | Accent, variant `poster` on posters |
| Pull quote | `quote` | Muted, left border padding |
| Floating panel | `card` | variant `glass` |
| Legal / credit line | `caption` | Bottom |
| Slide/page frame | `shell` | Carries gradient padding |

Night-wash is for **sparse** content. If the user provides essay-length copy, suggest `quiet-light` or split across more slides.
