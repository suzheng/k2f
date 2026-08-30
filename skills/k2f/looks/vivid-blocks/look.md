# vivid-blocks

This file plus `theme.json` is an **agent example**, not a drop-in package. If the user named a style, ignore this look. Otherwise rewrite the author `styles/theme.json` from this theme (adapt; do not ship it unchanged).

Bold, energetic layout built from **2–3 saturated color blocks** that partition the page. Geometry and contrast do the work — not hairline borders or gray-on-white minimalism. Best for event posters, marketing one-pagers, and upbeat slide decks.

## 1. Character

Confident and readable. Each block is a solid color field with **white type on color** (`on_block`). The page background stays white; color lives inside cards. Limit to three block hues per page — rotate `card` variants `block_a`, `block_b`, `block_c` (teal, coral, navy).

## 2. Surfaces and color

| Token | Hex | Use |
|-------|-----|-----|
| `block_teal` | `#4ECDC4` | `card` variant `block_a` |
| `block_coral` | `#FF6B6B` | `card` variant `block_b`, kickers, metrics |
| `block_navy` | `#2D3047` | `card` variant `block_c`, display titles |
| `on_block` | `#FFFFFF` | Text inside color cards |
| `paper` | `#FFFFFF` | Page background |
| `ink` | `#2D3047` | Body text on white areas |

Display titles and metrics pick up **navy or coral** on white backgrounds. Inside color cards, child text inherits `on_block` — keep copy short.

Default `card` uses teal. Cycle variants for adjacent tiles: A → B → C → A.

## 3. Type ramp (millipt = 1/1000 pt)

### Report (A4)

| Role | Size | Color |
|------|------|-------|
| `h1` | 24000 | `block_navy` |
| `h2` | 18000 | `accent` (teal-green) |
| `body` | 11000 | `ink` |
| `caption` | 9000 | `muted` |

Use color blocks sparingly in long reports — one hero band at chapter open, otherwise white flow.

### Slide deck (16:9)

| Role | Size | Notes |
|------|------|-------|
| `display` | 36000 | Navy on white, or white on a full-width `block_c` bar |
| `kicker` | 10000 | Coral |
| `body` | 12000 | On white or inside cards |
| `metric` | 52000 | Coral accent for KPI slides |

### Poster

| Role | Variant | Size |
|------|---------|------|
| `display` | `poster` | 58000 |
| `metric` | `poster` | 76000 |
| `kicker` | `poster` | 12000 |

## 4. Space

| Context | `shell` padding | Grid `gap` | Card padding |
|---------|-----------------|------------|--------------|
| Poster | 40000 | 16000–20000 | 20000 |
| Slide | 40000 | 12000 | 20000 |
| Report | page margins | 12000 | 20000 |

Color blocks need **air between them** — never bleed edge-to-edge without `shell` inset unless doing an intentional full-bleed band inside a fixed-height row.

## 5. Composition recipes

### Poster

```bash
python scripts/init_package.py --dir ./out/poster --title "…" --page a4 --margin 0
# rewrite styles/theme.json from this look's theme.json (adapt — do not ship unchanged)
```

1. Copy [`ex_poster_shell.json`](../../catalog/content/ex_poster_shell.json): `shell` + page height + `break_inside: "avoid"` + `{fr:1}` grower. Do not stack the whole page.
2. Header `pt` row: optional 2-col `kicker` / `caption`; or `display` variant `poster` (white, or inside `card` variant `block_c`).
3. **Feature grid in the `{fr:1}` row:** 3 or 6 items in 3-column grid. Each cell: `card` with rotating `block_a` / `block_b` / `block_c`, one bold `body` line (≤8 words).
4. **Stat row:** 2–3 `metric` variant `poster` in that grower (or a nested row), or one coral metric.
5. Footer `pt` row: `caption` on white — never on coral without checking contrast.

For 4 equal tiles: 2×2 grid, variants A/B/C/A.

### Slide deck

```bash
python scripts/init_package.py --dir ./out/deck --title "…" --page widescreen --margin 0
```

1. **Title / content / KPI slides:** copy [`ex_poster_shell.json`](../../catalog/content/ex_poster_shell.json) per slide (`960000×540000`). Put bands in `pt` rows and the main cards/metric in `{fr:1}`.
2. **Title slide:** navy full-width top `pt` row — `card` variant `block_c` with white `display` + `kicker`; subtitle `body` in the grower.
3. **Content slide:** 2-column grid of `block_a` / `block_b` cards **in the grower**, with bullet `list_item` children.
4. **KPI slide:** `metric` + `caption` in the grower.
5. Avoid more than two block colors on one slide.

### Report

```bash
python scripts/init_package.py --dir ./out/report --title "…" --page a4
```

1. Cover: large `display` + colored band — horizontal stack with `card` variant `block_c` behind title using `overlay`.
2. Interior: mostly normal `h1`/`body` on white. Use color cards only for **key takeaways** (1–2 per chapter).
3. Tables: navy header row (`table_header_cell` theme) — already styled.
4. Do not wrap every paragraph in a card.

## 6. Images

| Intent | Pattern |
|--------|---------|
| Hero photo | Full-width top: `image` in overlay under a bottom-aligned `card` variant `block_c` strip with title |
| Icon tile | Small image centered above `body` inside a color card |
| Logo | Top-left on white margin, outside color blocks |

Photos on color: prefer white text on a solid block overlay, not text directly on busy photos.

## 7. Don'ts

- No all-gray minimal cards — that's [`quiet-light`](../quiet-light/look.md).
- No dark gradient washes — that's [`night-wash`](../night-wash/look.md).
- No fourth block color on one page — stick to teal/coral/navy.
- No long paragraphs inside color cards (≤2 lines).
- No hairline-border-only layout — blocks should read as **color fields**.
- No white `body` text on white `paper` — check `on_block` vs `ink`.
- Do not use shadow elevation as the primary card treatment.

## 8. Mapping table

| User says | Role | Variant |
|-----------|------|---------|
| Hero headline | `display` | `poster` on posters |
| Section label | `kicker` | Coral |
| Feature / benefit tile | `card` | `block_a` / `block_b` / `block_c` rotate |
| Plain paragraph | `body` | On white, not in a card |
| KPI / stat | `metric` | `poster` on posters |
| Chapter title (report) | `h1` | Navy |
| Warning | `warning` | Yellow panel — distinct from brand blocks |
| Footer credit | `caption` | On white |
| Slide frame | `shell` | |

If content is narrative prose, use `body` on white — **not** a color card per paragraph.
