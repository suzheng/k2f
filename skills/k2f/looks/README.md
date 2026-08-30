# K2F looks

Example design skins for posters, slide decks, and reports. Each look is a **complete, packable `theme.json` plus a composition guide (`look.md`)**. They are **not** content templates and **not** drop-in files for the user to copy unchanged.

`init_package.py` always copies [`starter/`](../starter/) (core theme). Looks stay in this folder as **agent reference**.

## When to use (and when not)

| User | What to do |
|------|------------|
| Named a style, mood, brand, or design | **Ignore bundled looks.** Author `styles/theme.json` (and layout) to match what they asked. |
| Did **not** specify a design | Pick the **one** look that fits this content, then **rewrite** the package theme from that example. If none fits, `quiet-light`. |

Do not load all three `look.md` files in the same task.

| Look | When this example fits (user gave no style) |
|------|---------------------------------------------|
| [`quiet-light/`](quiet-light/) | Calm, near-monochrome, generous whitespace. Reports and understated posters. Clean PDF (no shadows/gradients). Fallback if nothing else fits. |
| [`vivid-blocks/`](vivid-blocks/) | Bold, colorful, energetic. Saturated color-block partitions. |
| [`night-wash/`](night-wash/) | Dark, cinematic, launch-style. Deep background with a single linear wash. Gradients may trigger PDF stamp path. |

## How an agent uses a look

1. `init_package.py` as usual — starter theme is fine to start from.
2. Read **only** `looks/<name>/look.md` and that look's `theme.json`.
3. **Rewrite** `styles/theme.json` in the author package: copy the example as a starting point, then change palette, type, and variants to fit this document. Do not treat the bundled file as finished output.
4. If you use extension roles (`display`, `kicker`, `caption`, `metric`, `shell`), they **must** exist in the package theme (starter does not define them) — copy those role blocks from the example, then adapt.
5. Author `content/root.json` from catalog shapes + that look's composition recipes. Posters/slides: start from [`catalog/content/ex_poster_shell.json`](../catalog/content/ex_poster_shell.json).
6. `pack_verify.py --render preview.png` and open the PNG; check the look's **Don'ts**.

A whole-file overwrite of `styles/theme.json` is allowed when the rewrite is complete. Do **not** merge a few fields into starter — partial themes cause `UNKNOWN_ROLE`.

## Shared role vocabulary

All example looks define the **same role names**. If you rewrite from a look, keep these names so recipes and variants stay consistent.

### Standard roles (starter + catalog)

`default`, `document`, `section`, `h1`, `h2`, `h3`, `h4`, `body`, `warning`, `card`, `table`, `table_header_cell`, `table_row_cell`, `list_item`, `code`, `quote`, `rule`, `math`, `running_header`, `running_footer`, `signature_block`, `image`

### Look extension roles (semantic, not layout names)

| Role | Use for |
|------|---------|
| `display` | One dominant headline per page/slide/poster. Use variant `poster` on posters for larger type. |
| `kicker` | Short label above a headline (category, date, series name). |
| `caption` | Image credit, footnote, source line, slide footer note. |
| `metric` | Prominent number or stat. Variant `poster` for hero-scale metrics. |
| `shell` | Page/slide inner frame — safe-area padding via role `box_decoration.padding_pt`. |

Chapter titles in long reports stay **`h1`**, not `display`. Slide/poster heroes use **`display`**.

### Variants (examples)

| Role | Variant | When |
|------|---------|------|
| `display` | `poster` | Single-page poster or flyer hero title |
| `metric` | `poster` | Hero-scale stat on a poster |
| `kicker` | `poster` | Slightly larger kicker on posters |
| `card` | look-specific | e.g. `raised` (quiet-light), `block_a`/`block_b`/`block_c` (vivid-blocks), `glass` (night-wash) |

## Legal note

Looks are original palettes and layout recipes inspired by common modern document aesthetics. They do not reproduce trademarked brand names, logos, or proprietary color systems.
