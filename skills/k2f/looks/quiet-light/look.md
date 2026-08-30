# quiet-light

This file plus `theme.json` is an **agent example**, not a drop-in package. If the user named a style, ignore this look. Otherwise rewrite the author `styles/theme.json` from this theme (adapt; do not ship it unchanged).

Calm, near-monochrome design with generous whitespace. One accent hue at most. Cards are outlined, not elevated. Best default for reports, formal one-pagers, and understated posters. **No shadows or gradients** — PDF export stays clean.

## 1. Character

Whisper, don't shout. Let type size and spacing carry hierarchy. Backgrounds stay warm off-white; ink stays soft black. Avoid decorative color blocks — structure comes from alignment and breathing room.

## 2. Surfaces and color

| Token | Role |
|-------|------|
| `paper` `#FAFAFA` | Page background, card fill |
| `ink` `#1C1C1C` | Primary text, display titles |
| `muted` `#6B6B6B` | Kickers, captions, running headers |
| `signature` `#F3F3F3` | Code blocks, subtle panels |
| `rule` `#D8D8D8` | Dividers |
| `header_bg` / `header_ink` | Table headers (light gray bar) |

Cards use `border: hairline`, not shadow. Default card variant is flat; use `raised` only when a second outline weight helps separation.

## 3. Type ramp (millipt = 1/1000 pt)

### Report (A4, default margins)

| Role | Size | Notes |
|------|------|-------|
| `h1` | 24000 | Chapter openers only |
| `h2` | 18000 | Section heads |
| `h3` | 14000 | Subsections |
| `body` | 11000 | Line height 1.5× |
| `caption` | 9000 | Footnotes, figure credits |
| `running_*` | 9000 | Muted |

### Slide deck (16:9, `--margin 0`)

| Role | Size | Notes |
|------|------|-------|
| `display` | 34000 | One per slide — the slide title |
| `kicker` | 9000 | Optional series label above display |
| `body` | 12000 | Short bullets only; max ~5 lines |
| `caption` | 9000 | Source line at bottom |

### Poster (A4 or custom, `--margin 0`)

| Role | Variant | Size |
|------|---------|------|
| `display` | `poster` | 56000 |
| `kicker` | `poster` | 11000 |
| `metric` | `poster` | 72000 |
| `body` | — | 12000–14000 for supporting lines |
| `caption` | — | 9000 bottom bar |

Use **`display` + variant `poster`**, not oversized `h1`, for the hero headline.

## 4. Space

| Context | `shell` padding | Stack `gap` | Card padding |
|---------|-----------------|-------------|--------------|
| Report page | — (use page margins) | 8000–12000 between blocks | 20000 |
| Slide | 48000 inner inset | 16000 title → body | 20000 |
| Poster | 56000–64000 | 24000 between hero and grid | 20000 |

Root node: **no** `padding_pt` on `document`/`root` — safe area lives in `shell` or page margins.

## 5. Composition recipes

Copy layout **shapes** from [`catalog/content/`](../../catalog/content/). Do not ship catalog files as the deliverable.

### Poster (single page)

```bash
python scripts/init_package.py --dir ./out/poster --title "…" --page a4 --margin 0
# rewrite styles/theme.json from this look's theme.json (adapt — do not ship unchanged)
```

1. Under `root`, **one** child: `role: "shell"`, `break_inside: "avoid"`, fixed `layout.height` = page height (842000 for A4).
2. Inside `shell`, vertical `stack` with `gap` 24000.
3. **Hero block:** `kicker` (optional) → `display` variant `poster` → one `body` subtitle line max.
4. **Points grid** (pick by count):
   - 1–2 points: vertical stack, `align_items: "center"`, `text_align: "center"` on text roles via centered parent stack — see [`ex_stack.json`](../../catalog/content/ex_stack.json).
   - 3 or 6 points: 3-column grid — [`ex_grid.json`](../../catalog/content/ex_grid.json).
   - 4 points: 2×2 grid.
5. Each point: optional `card` variant `flat` wrapping a short `body` line, or plain `body` with a bold lead word via emphasis modifier.
6. **Footer:** `caption` for date, URL, or credit.
7. Full-bleed photo: wrap `shell` content in `overlay` — background `image` child first, then a semi-opaque `card` variant `flat` if text must sit on photo — [`ex_overlay.json`](../../catalog/content/ex_overlay.json).
8. Pack with `--expect-pages 1 --render preview.png`.

### Slide deck (multi-page)

```bash
python scripts/init_package.py --dir ./out/deck --title "…" --page widescreen --margin 0
```

1. Each top-level child under `root` = one slide.
2. Slide container: `role: "shell"`, `break_inside: "avoid"`, `layout: { "type": "stack", "width": 960000, "height": 540000, "direction": "vertical", "gap": 16000 }`.
3. Slide anatomy: optional `kicker` → `display` → 3–5 `list_item` or short `body` nodes. One idea per slide.
4. Closing slide: centered `display` + `caption` only.
5. Render page 0..N with `k2f render … --page N`.

### Report (multi-page article)

```bash
python scripts/init_package.py --dir ./out/report --title "…" --page a4
```

1. Keep default page margins (56pt). Do **not** zero margins.
2. Cover: optional centered stack — `kicker`, `display`, `body` subtitle, `caption` date — inside one `shell`-less section with vertical centering via fixed-height stack + `justify_content: "center"`.
3. Body flow: `h1` opens each chapter; `body` paragraphs as **separate text nodes**; `gap` between siblings for paragraph spacing.
4. Tables: copy [`ex_table.json`](../../catalog/content/ex_table.json). Header uses light `header_bg`, not dark bars.
5. Warnings and quotes: native `warning` / `quote` roles — no extra decoration.
6. Optional `manifest.running_blocks` for header/footer — keep text short; see [`catalog/manifest.json`](../../catalog/manifest.json).

## 6. Images

| Intent | Pattern |
|--------|---------|
| Full-bleed background | `overlay`: image child (declared width/height = canvas), content stack on top |
| Inline figure | `image` role between `body` paragraphs; follow with `caption` |
| Logo mark | Small `image`, `self_align: "start"` in theme; place in slide/poster header row via horizontal stack |

SVG labels must be `<path>`, not `<text>`. PNG/WebP only under `assets/images/`.

## 7. Don'ts

- No saturated color-block grids — use [`vivid-blocks`](../vivid-blocks/look.md) instead.
- No dark full-page backgrounds — use [`night-wash`](../night-wash/look.md).
- No shadows, blur, or gradients in this look.
- No more than one `display` per page/slide.
- Do not use `h1` for poster heroes — use `display` variant `poster`.
- Do not cram long report paragraphs onto slides.
- Do not invent spacer nodes — use `gap` and role padding only.
- Do not merge this theme with starter — replace the whole `theme.json`.

## 8. Mapping table

| User says | Role | Notes |
|-----------|------|-------|
| Title / headline / hero | `display` | variant `poster` on posters |
| Chapter / section title (report) | `h1` / `h2` | Not `display` |
| Subtitle / tag / category | `kicker` | Above display |
| Body / paragraph | `body` | One node per paragraph |
| Bullet | `list_item` | With `list_id` |
| Stat / big number | `metric` | variant `poster` on posters |
| Callout / note | `warning` or `quote` | Pick by tone |
| Feature box / tile | `card` variant `flat` | Only if content is truly boxed |
| Photo | `image` | Millipt width required |
| Credit / source / date line | `caption` | |
| Page/slide safe frame | `shell` | Container role with padding |
| Divider | `rule` | Small height container |

If the user's content has no discrete "cards," use `body` or `list_item` — do not force `card`.
