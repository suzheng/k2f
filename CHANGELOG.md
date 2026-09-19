# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/).

## [Unreleased]

## [0.3.1] - 2026-09-19

### Fixed

- `export-idml`: two-line author-`\n` titles beside a column (hero image) drop `NoBreak`. InDesign still hid the whole story when every paragraph was NoBreak+HeightOnly, even with lock-line slack — the designed newline stays, the lock column is not WidthOnly-grown into the gutter. Iceberg for any stacked display title next to a figure, not only this editorial hero. Invoice/text/corpus tests still pass.
- `export-idml`: native `Table` only when a row's lock boxes share y/height. Shorter rounded pills and vertically centered cards used to stretch `Cell` fill across the whole InDesign slot (status chips, day labels). Rounded fills with column gaps stay DrawBox + DrawText so lock rects and radii win. Invoice-style 4pt-gap square grids still harvest. Invoice/text/corpus tests still pass.
- `export-docx` / `export-pptx`: line-box `tIns` slack is only for shrink-wrapped headers (≤1.4× face). Padded heading rules (~1.5–1.7×, `padding_pt.bottom` + `::edge_bottom`) were dumping that gap into tIns, so glyphs sat on the underline. Folded compact top-aligned one-liners use face pitch and put leftover in `w:after` so Writer does not pack the line onto the bar. Wrap=none headings with a same-node bottom bar also drop native `w:u` / `a:rPr u` — Writer/Impress paint that thick and far; the lock bar is the designed rule. Links and wrapped bodies keep native underline. Invoice/text/corpus tests still pass.
- `export-docx`: folded wrapped list items keep lock leading and do not NBSP-pad the marker gutter. `(cy − face)/2` pill padding was applied as `w:before`/`w:after` on 2-line bullets, and leftover `lIns` was padded on top of the literal `•`, so card lists opened huge gaps and wrapped like a larger face. Iceberg for any wrapped `list_item` inside a folded card. Invoice/text/corpus tests still pass.
- `export-docx`: embed a Bold-only package font under its TTF family name (strip “Bold” from FULL_NAME when Windows FAMILY is missing) as both `embedBold` and `embedRegular`, and emit `w:b` on those runs. Missing unicode FAMILY used to register `K2F Font` while runs named the stem, so hosts substituted a default serif at the lock size (27pt Bradley Hand titles). Iceberg for any decorative Bold-only face. Invoice/text/corpus tests still pass.
- `export-docx` / `export-pptx`: one-line frames sized to the lock line box (`face × line_height_mult`) keep that leftover as `tIns`. Office `anchor=t` packed glyphs to the frame top, so running headers at y=0 sat on the page edge. Tall padded cards still use glyph `y_offset` only. Invoice/text/corpus tests still pass.
- `export-idml`: Regular-only lock `bold` keeps `FontStyle` on the embedded TTF subfamily and faux-bolds with a same-color 1/45 em character stroke (`StrokeWeight` + fill `StrokeColor` only). K2F paint centers 1/30 em on the glyph path; the same width in InDesign reads extra-black. Advertising `Bold` substitutes a system Bold cut or falls back to Regular. `StrokeType` / `StrokeAlignment` drop or outline the stroke; a default-Black stroke made type extra-black. Real Bold/Italic files still use those styles with no stroke (do not double-bold a Bold TTF). Synthetic italic is still `Skew`. Superscript-only glyph rows (™) are not a first-line indent or a bottom inset that clips the title. Invoice/text/corpus tests still pass.
- `export-docx` / `export-pptx`: tall white cells (table rows, stretched card bodies) keep lock first-line `tIns` instead of `anchor=ctr`. Writer ignores DrawingML `anchor=ctr` on `wps` boxes, so centered lock glyphs sat at the top. Compact pills still use `anchor=ctr`. Quote/callout cards that are top-padded in a taller shell stay top-aligned. Invoice/text/corpus tests still pass.
- `export-docx`: leftover nested widgets (empty checkboxes, small form squares) stay above a folded card fill. Stroke-only checkboxes were demoted as covering outlines, and leftover-width task rows beside a checkbox gutter were upgraded to Right. Iceberg for any checkbox/task card, not only this planner. Invoice/text/corpus tests still pass.
- `export-docx`: folded/absorbed square cards inset their fill by the `::edge_*` bar thickness. Writer paints the opaque text box over sibling rim bars that share the AABB, so white title frames and cream exhibition cards vanished (only the fill showed, or nothing on paper). Bars stay on the lock rim. Iceberg for any axis-aligned folded card frame, not only these gallery hits. Invoice/text/corpus tests still pass.
- `export-docx`: explicit `\n` labels infer per-paragraph `w:jc` from lock line gaps (e.g. full-width first line + centered second in invitation copy). Multi-line frames with balanced side insets keep `wrap=square` below `WIDE_CENTER_MIN` so Word/LibreOffice honor center alignment instead of painting at `lIns`. Invoice/text/corpus tests still pass.
- `export-docx`: folded callout/quote bodies drop lock-pinned `\n` as well as NBSP. Host-wider metrics were wrapping the last word of a pinned line onto its own row ("the" / "formal…") inside wrap=square cards. Iceberg for any long folded paragraph, not only this recital. Compact two-line titles stay pinned. Invoice/text/corpus tests still pass.
- `export-docx`: embed K2F package fonts in `word/fonts/*.odttf` with `fontTable.xml` and `w:embedRegular` on `w:rFonts`. Hosts without proprietary faces (e.g. Arial Unicode MS from embedded CJK TTFs) no longer render blank Chinese text.
- `export-docx` / `export-pptx`: list wrap NBSP pad matches the literal `•` / `{n}.` marker (~0.75em), not the lock marker box (12–16pt). Hang-by-gutter made continuation lines sit past line-1 body. Iceberg for any wrapped `list_item`, same as IDML marker-width hanging. Invoice/text/corpus tests still pass.
- `export-docx` / `export-pptx`: native table cells keep lock left/right glyph pad (`a:tcPr marL`/`marR` in PowerPoint, `w:tcMar` in Word). Cells had hardcoded zero insets, so first-column labels sat on the left rule and right-aligned numbers sat on the right rule. Table width is fixed, so the text-box ≥85%-full `rIns` skip does not apply. Invoice/text/corpus tests still pass.
- `export-docx`: folded quote/callout cards keep ordinary spaces, lock leading, and `anchor=t`. wrap=none labels were NBSP'd and single-label shells were vertically centered, so a wrapping paragraph stayed one overflowing line and hid the attribution. Compact pills still center + NBSP. Invoice/text/corpus tests still pass.
- `export-pptx`: multi-line text uses lock leading in Impress. DrawingML `spcPts` is ignored on one-line / wrapped runs, so titles and body packed to single spacing. Pin lock wraps, put lock−face on `spcAft` between stacked paragraphs, and emit `spcPct` for single-para wraps (lists). Iceberg for any wrapped PPTX text, not only this poster. Invoice/text/corpus tests still pass.
- `export-idml`: translucent solids and strokes (`#RRGGBBAA` scrims, glass cards) are native Rectangle fill/stroke + `FillTransparencySetting` / `StrokeTransparencySetting`, not opaque RGB `k2f-raster` slices. Rasterizing a frost plate with the page paper behind it hid the hero photo (gray overlay) and washed glass cards. Same iceberg as PPTX/DOCX `a:alpha`. Document `Intent` is `WebIntent` so InDesign keeps process RGB swatches (PrintIntent converted saturated gradients to CMYK on PDF). Linear `GradientFillStart`/`Length` use K2F paint’s AABB corner projection in page coordinates; `GradientFillAngle` is negated (K2F 90°=down, InDesign 90°=up). Blur/math still rasters. Invoice/text/corpus tests still pass.
- `export-idml`: linear gradients are native InDesign `Gradient` fills with live rounded corners, not opaque RGB `k2f-raster` slices. Rasterizing a rounded gradient with the page paper behind it put a white AABB around pills and clipped transparent SVG card corners (hero format chips / doc stack). Same iceberg as PPTX/DOCX `a:gradFill`. Blur/glass still rasters. Invoice/text/corpus tests still pass.
- `export-docx`: nested same-node chips/pills (text + `box_decoration`) keep their fill and lock AABB instead of folding into the parent card. Absorb already turned those chips into filled text boxes, so the card fold treated them as labels — the chip fill vanished and padding-inferred center alignment was applied to the card width (`self_align: start` tags and badges). Chips painted before body labels are raised above the folded card so Writer cannot hide them. Iceberg for any filled chip inside a card, not only these gallery hits. Invoice/text/corpus tests still pass.
- `export-docx`: leftover left-icon pills still trim side insets for wrap slack, but `lIns` never shrinks past the icon's right edge. Skipping the 25% trim entirely made the wrap column equal the lock text width, so host-wider "PPTX" / "Fillable PDF" mid-wrapped. Right-side photos still keep their wrap column. Invoice/text/corpus tests still pass.
- `export-docx` / `export-pptx`: compact one-line pills (~1.9× face) still count as vertically centered. The old 2×-face gate left Word `tIns` + `anchor=t` on same-node chips (hero right format labels), so the glyphs sat low. Tight boxes no taller than the face stay top-aligned. Invoice/text/corpus tests still pass.
- `export-docx`: leftover left icons keep full `lIns` when folding a banner. The 25% inset trim for host-wider pills was pulling titles under the icon. Right-side photos still keep their wrap column. Invoice/text/corpus tests still pass.
- `export-pptx`: linear gradients are native `a:gradFill` (same as Word), not opaque RGB `k2f-raster` slices. Rounded gradient pills no longer carry a page-background AABB. Shadows on those fills still drop (v1, same as Word). Invoice/text/corpus tests still pass.
- `export-docx` / `export-pptx`: picture `spPr` emits `a:noFill` so hosts do not paint a theme-white rectangle behind transparent SVG/PNG (hero card stacks, icons).
- `export-docx`: same-node DrawBox+DrawText absorb copies native `a:gradFill`, not only solid `fill_hex`. Gradient pills whose label shared the box id (hero format chips) lost their fill.
- `export-docx`: leftover icons inside a folded pill join the `wps` z-stack above the shell. Writer hid `pic:pic` under the expanded gradient text box. Iceberg for any icon+label chip, not only this hero.
- `export-docx`: single-label folded pills trim side insets 25% so host-wider "PPTX" / "Fillable PDF" do not mid-wrap.
- `export-docx`: axis-aligned dashed/dotted four-sided rims use `::edge_*` bars like solid rims. Folded opaque text boxes clipped centered `a:ln`, so navy-grid card frames vanished (dash pattern is lost on square frames; v1). Invoice/text/corpus tests still pass.
- `export-docx` / `export-pptx`: checked form-field checkboxes export the painted `X`, not the semantic value `true`. A 12pt square was clipping `true` to `tr` / garbled glyphs. Iceberg for any checked `form_field` checkbox — including table cells. Invoice/text/corpus tests still pass.
- `export-docx`: folding a banner with a left icon no longer treats the empty right of a glyph-tight title as a photo column. That `rIns` squeezed one-line section headers into mid-word wraps. Right-side hero photos still keep the wrap column left of the image. Invoice/text/corpus tests still pass.
- `export-docx`: folded card stacks of padded one-line cells (note-line / underline rows) use the lock box height as paragraph pitch, not face size. wrap=square leftover-width one-liners have no lock leading — extra cy is still padding. Only wrap=square bodies with a real inter-line `line_twips` keep lock leading (soft-wrapped quotes). Iceberg for any folded underline/journal table, not only this planner. Invoice/text/corpus tests still pass.
- `export-pptx`: harvested native tables still emit `::edge_*` bars from partial-edge cell DrawBoxes. LibreOffice Impress drops hairline/dashed `a:lnB`, so ruled note lines vanished. Four-sided strokes stay table `a:ln`. Iceberg for booktabs and any bottom-only cell rule. Invoice/text/corpus tests still pass.
- `export-docx`: overlay banners that paint a parent fill then a same-size child image no longer keep the leftover fill. Writer composites that in-front non-txBox bar above the folded hero raster, so the artwork (and titles folded into it) vanished. Drop the redundant same-AABB shell; smaller FIG photos still keep the card. Invoice/text/corpus tests still pass.
- `export-docx`: rounded paper-colored boxes with a four-sided stroke (empty form `box` fields) stay in front. Axis-aligned clones already split `::edge_*` bars in front of a behindDoc fill; a roundRect merges `a:ln` onto the fill, so behindDoc hid the rim under the page wash. Iceberg for any rounded paper/white card that cannot emit edge bars. Invoice/text/corpus tests still pass.
- `export-docx`: folded card stacks keep per-paragraph `w:jc` (centered title + left body). Align used to follow only the widest label, so notice titles in gray shells painted left.
- `export-docx`: a shrink-wrapped title centered by parent `align_items` infers Left inside a glyph-tight box. After fold, a narrower label inset equally from the content column keeps `w:jc=center` (header/notice banners). Iceberg for any stacked shell, not only this form.
- `export-docx` / `export-pptx`: left-aligned boxes always keep lock left pad as `lIns`. Dropping it only on ≥85%-full lines made short table cells indent further than filled neighbors in the same column.
- `export-docx` / `export-pptx`: list items stay Left. Marker glyphs are not source clusters, so gap inference (and card-fold leftover) treated short bullets as Right/Center — wrap sat under `•` and items zigzagged in folded cards. Iceberg for any `list_item` in a tight or mixed-width shell.
- `export-docx` / `export-pptx`: list wrap breaks are U+2028, not `\n`. Folding a card splits `\n` into a new paragraph, so the continuation started in the marker gutter. Continuation indent is NBSP pad only (no `w:ind` / `marL` hanging, which double-counted when the host honored it).

## [0.3.0] - 2026-09-18

### Added

- Pack/compile/raster reject SVG live text with `SVG_TEXT` (not `IMAGE_SIZE`). Still fail-closed; put labels in a K2F text node or convert to `<path>`.
- List markers and code blocks use the same package-font coverage fallback as body text.
- Skill catalog: OFL `NotoSerif-Regular.ttf` for serif headings or body (`--add-font catalog/assets/fonts/NotoSerif-Regular.ttf`, then retarget role `font_family`). Starter stays Roboto-only; no cross-family fallback.
- `k2f export-idml <package> -o <out-dir>` — InDesign package (`{stem}.idml` + `Document Fonts/`). `-o file.zip` writes the same tree zipped; `--idml-only` writes a lone `.idml`. The IDML ZIP still does not contain TTF bytes. IDML is not a K2F source (`IDML_IS_NOT_A_SOURCE`).
- Role/variant `image_fit` (`contain` default, or `cover` center-crop). An image node's own `corner_radius` clips that bitmap. PPTX/DOCX emit `a:srcRect` + `roundRect`; IDML uses FillProportionally + frame corners.

### Changed

- Root `layout` of `grid` / `overlay` / `columns` / horizontal `stack` is a compile error (was a silent vertical-flow fallback for horizontal). Nest under a child. Catalog `ex_*.json` are children — do not replace `content/root.json`.

### Fixed

- `export-docx` / `export-pptx`: lock paint already applied theme `emphasis` (including `emphasis` → italic) is not remapped to bold. Overlaying every non-`italic` intent as bold made true-italic runs export as bold-italic. Empty-paint fallback still bolds default emphasis.
- `list_item` with a partial `list_style` (for example only `bullet_glyph`) compiles by merging starter defaults. Explicit `0` still wins; schema fields stay optional.

## [0.2.4] - 2026-09-11

### Added

- Desktop reader: continuous pinch / Ctrl+wheel zoom with display LOD, copy-all-as-text, and macOS menu checkmarks.
- Web viewer: display-scale LOD helpers aligned with the desktop reader.

### Fixed

- `export-docx`: fold hero titles into the parent banner. Writer paints a sibling non-txBox fill/raster above txBox labels, so titles on a blur raster or a photo banner vanished. Blur chrome folds as `blipFill` + text; `pic:pic` side photos stay siblings (wrap keeps the text column). FIG cards with a wps raster still skip fold. Invoice/text/corpus tests still pass.
- `export-docx`: multi-label card folds restore per-paragraph line pitch and inter-label `after` from the lock (host auto had crushed body leading in callouts/metric stacks). Align follows the widest label so a leading centered pill/kicker no longer centers the whole card. Drop `bIns`, use `exact` pitch, and only treat short single-para boxes as one-line (a 2-line PASSBAND cy was mistaken for one line and doubled pitch, clipping the last glyphs). Iceberg: any folded shell with stacked labels—not only these two gallery hits. Invoice/text/corpus tests still pass.
- `export-docx`: axis-aligned solid four-sided rims emit four thin `::edge_*` bars instead of one full-AABB `{id}::stroke`; rounded solid rims keep `a:ln` on the fill (no sibling). Hosts still hit-tested closed noFill frames over nested labels even after z-order demote (full-page / plaque business-card shells). Dashed outlines keep closed `a:ln`. Invoice/text/corpus tests still pass.
- `export-docx` / `export-pptx`: multi-line text that starts with a superscript/subscript no longer crushes line spacing (affiliations stacked on one baseline). Line clustering and spacing use the largest face in the box, not `text_runs[0]`; sub-face baseline gaps are ignored. Iceberg for any multi-line box with a leading super/sub run—not only academic header blocks.
- `export-docx` / `export-pptx`: list bullets no longer vanish on tight ATS marker columns. Floating `w:numPr` / `a:buChar` markers were clipped to a speck; bullets are a literal U+2022 text run (Arial) with lock hanging for wrap. Iceberg for any `list_item` bullet export—not only gallery CVs.
- `export-docx` / `export-pptx`: numbered list markers are literal `{n}.` text runs (same strategy as bullets). Host `w:numPr` / `a:buAutoNum` across floating text boxes renumbered by document/z-order and the DOCX numId pool capped at 64—bibliographies flipped to descending or reset to `1.` after item 64. Iceberg for any long numbered `list_item` export.
- `export-docx` / `export-pptx`: literal list hanging uses the lock marker column only (`body_left − ink_left`), with outer pad as shape `lIns`/`lIns`. Hanging by body-left alone double-counted the marker after prepending `{n}.` / `•`, so wrap lines sat too far right. Iceberg for any multi-line `list_item` export.
- `export-docx`: full-AABB `{id}::stroke` companions and stroke-only decorative frames no longer sit above nested labels (they were invisible click shields over cards/pages). Thin `::edge_*` bars still rise above opaque text underlays; closed outlines demote below overlapping text. Shell fold absorbs the `::stroke` companion into the folded text box so borders are not lost. Iceberg: any fill+four-sided stroke split over labels (menus, cards, legal party boxes, resume work cards)—not only gallery hits. Invoice/text/corpus tests still pass.
- `export-docx`: inject a `behindDoc` full-page paper wash when the lock omits a page-sized DrawBox. `w:background` alone is remapped/hidden in Word Dark Mode. Invoice/text/corpus tests still pass.
- `export-docx` / `export-pptx`: list items no longer ghost stacked bullets or shift later items. Each floating list text box gets its own `numId`, lists skip lock soft-break pinning (host wrap + hanging), and bullet markers pin Arial + RGB. Iceberg for any multi-item `list_item` export—not only this gallery report.
- `export-docx`: emit floating anchors high→low by `relativeHeight`. LibreOffice Writer largely ignores `relativeHeight` among `wps` shapes and paints earlier document anchors on top, so card fills emitted before images covered FIG photos and palette chrome. Word still keys off `relativeHeight`. Invoice/text/corpus tests still pass.
- `export-docx` / `export-pptx`: SVG assets rasterize to PNG at export (`decode_raster` / resvg), matching paint/PDF. Raw `image/svg+xml` was often blank in LibreOffice Writer (uneven in Word). Invoice/text/corpus tests still pass.
- `export-docx`: do not fold a card shell when it contains a picture/raster — fold kept the caption’s higher `relativeHeight` and raised the opaque fill above the image (blank FIG cards). Invoice/text/corpus tests still pass.
- `export-docx`: tiny paper-RGB chips (palette swatches) stay in front; only large paper-colored body clones (≥ ~0.5% of page area) use `behindDoc` under the page wash. Invoice/text/corpus tests still pass.
- `export-docx`: opaque text-box underlays (Word Dark Mode) no longer erase hairline table rules and closed outlines. Fill+`a:ln` always split to a `::stroke` sibling; classify raises `::edge_*` / `::stroke` above overlapping opaque text fills. Same-node pills still absorb the stroke companion into the text box. Iceberg: any partial-edge rule under a later underlay (booktabs, row underlines, framed cards)—not only gallery tables. Invoice/text/corpus tests still pass.
- `export-docx` / `export-pptx`: soft-wrapped or near-full **center** (and justify) frames keep `wrap=square` so Word/PowerPoint honor `w:jc` / `algn`. Pinning lock line breaks used to force `wrap=none`, and the ≥85% pill carve-out skipped page-width subtitles — hosts then painted at `lIns` (cover titles looked left-aligned). Narrow pills stay `wrap=none`. Invoice/text/corpus tests still pass.
- `export-docx`: contrasting full-page slide shells (cream paper + dark `slide.04`, dark paper + white `slide.02`) no longer go `behindDoc` — only paper-RGB solids and full-page gradients do. Dual full-page washes were remapped in Word Dark Mode on single pages. `paper_hex_eq` strips `#`. Invoice/text/corpus paths unchanged.
- `export-docx` / `export-pptx`: wide right-aligned one-liners use `wrap=square` so hosts honor `w:jc` / `algn` (same class of bug as center/justify). Unfilled table number cells used `wrap=none`, so Word painted at `lIns` and zigzagged against filled alt/total rows that already forced wrap via self-fill absorb. Glyph-tight right lines stay `wrap=none`. Invoice/text/corpus tests still pass.
- `export-docx`: multi-label card folds restore per-paragraph line pitch and inter-label `after` from the lock (host auto had crushed body leading in callouts/metric stacks). Align follows the widest label so a leading centered pill/kicker no longer centers the whole card. Iceberg: any folded shell with stacked labels—not only these two gallery hits. Invoice/text/corpus tests still pass.
- `export-docx`: folded card/pill labels. Mirror the lock left pad on the right so the wrap column is the shell interior (a kicker-narrow column mid-wrapped host-bold tracked labels like `SOUNDSTAGE`). Single-label pills pin face-size exact line spacing and vertically center so host auto single does not fill a tight badge. Multi-label stacks use per-paragraph `atLeast` pitch + lock gaps (not host-auto). Invoice/text/corpus tests still pass.
- `export-docx`: regression test that dark full-page paper (`#0B0F19`-class) stays a `behindDoc` wash (cards in front). Word Dark Mode remaps/hides `w:background` alone — gallery dark DOCX must keep the lock DrawingML paper. Invoice/text/corpus paths unchanged.
- `export-docx`: one stacking contract. Page paper is the lock's full-page solid (`behindDoc`, no txBox) plus `w:background` as the page-0 Office slot (Word Dark Mode may hide that slot). Cards/titles never go behindDoc — hosts z-order that stack by size, so a page-sized behindDoc drawing hid every smaller fill. Nested labels still fold into the shell. Invoice/text/corpus tests still pass.
- `export-pptx`: native table cells set `a:tcPr/@anchor` from lock leftover (PowerPoint ignores `bodyPr/@anchor` and defaults the cell to top). Non-centered cells copy first-line `y_offset` to `bodyPr tIns`; one-line cells pin `spcPts` to the face. Cell runs take lock glyph tracking like text boxes. `tcPr` margins stay 0 so insets are not doubled. Invoice/text/corpus tests still pass.
- `export-docx` / `export-pptx`: theme `dk1`/`lt1` stay `srgbClr` (never `sysClr`) but use sentinels `000002`/`FFFFFD`, not the content pin `000001`/`FFFFFE`. Word was snapping lock black/white onto those slots and remapping them in Dark Mode. Chrome-backed Word text boxes emit a fully-transparent `a:solidFill` instead of `a:noFill`. Invoice/text/corpus tests still pass.
- `export-docx`: skip the full-page solid as a DrawingML shape (hosts stack that behindDoc drawing on top of every other behindDoc fill). Nested labels fold into their parent shell as one `wrap=square` text box so title banners, table-header cells, and cards keep fill without covering glyphs. Same-node pills already folded; `wrap=none` was still shrinking those frames to the text. Later pages whose paper differs from page 0 emit a behindDoc `page_paper` drawing after that fold. Invoice/text/corpus tests still pass.
- `export-docx`: overlapped nearly-opaque `#RRGGBBAA` card shells (`fill_alpha >= 128`) go `behindDoc` like opaque plaques. Writer was compositing those fills over later labels, so dark social cards veiled every nested title, SVG, and CTA. Light frost (`alpha < 128`) and gradients stay in front. Outline split is unchanged. Invoice/text/corpus tests still pass.
- `export-docx` / `export-pptx`: running header/footer runs keep lock glyph tracking (`w:spacing` / `a:rPr spc`). The `{{page_*}}` path used to pass empty glyphs / `spc=0`, so tracked folios (contract headers, invoice labels) lost letter-spacing. Glyph *ranges* still cannot map the expanded string. Invoice/text/corpus tests still pass.
- `export-docx`: `{{page_current}}` / `{{page_total}}` running labels paint in the body with lock-resolved numbers (same as PPTX). PAGE/NUMPAGES in `footer1.xml` were invisible in LibreOffice Writer whenever `pgMar` is 0. Invoice/text/corpus tests still pass.
- `export-docx`: overlapped opaque card shells (abstract/quote/figure plaques) go `behindDoc` so Writer cannot paint the fill above later labels. Dropping the empty `txBox` was not enough; thin rules, gradients, and glass stay in front. Invoice/text/corpus tests still pass.
- `export-docx`: a four-sided stroke on an overlapped opaque card (colophon, figure frame, drop-cap box) stays in front as a fill-less outline. Classify used to send the merged fill+`a:ln` behind, so Writer hid the border under `w:background`. Same split as paper-colored frames at create time. Invoice/text/corpus tests still pass.
- `export-docx`: shadowed boxes stay native fill+stroke instead of an opaque `k2f-raster` PNG (Writer paints that slice above later labels even as `wps:wsp`; glow is a v1 gap). Also drop the empty `txBox` on any in-front fill/raster that later paint intersects (same overlap rule as lock images). Invoice/text/corpus tests still pass.
- `export-pptx`: translucent solids (`#RRGGBBAA` plaque/card fills) are native `a:solidFill`+`a:alpha` instead of `k2f-raster` pics. Gradients, blur, and math still raster. Invoice/text/corpus tests still pass.
- `export-pptx`: shadowed boxes stay native fill+stroke instead of an opaque `k2f-raster` PNG. The shadow crop is larger than the box (blur/spread), so the baked slice covered earlier labels in the glow halo (invoice surcharge vs total plaque). Glow is a v1 gap, same as Word. Invoice/text/corpus tests still pass.
- `export-docx` / `export-pptx`: `#RRGGBBAA` box borders kept lock alpha (`a:ln` / edge-bar fill). Opaque `#RRGGBB` strokes still omit `a:alpha`. Invoice/text/corpus tests still pass.
- `export-pptx`: partial-edge borders (`edges: left` quote rules, top/bottom hairlines) were a four-sided `a:ln`, so a left-only callout became a full rectangle. Same split as Word: four-edge strokes stay `a:ln`; other edges are filled bars. Invoice/text/corpus tests still pass.
- `export-docx`: LibreOffice Writer paints `pic:pic` above every DrawingML shape, so a decorative lock image under later labels hid the title. Images that later paint overlaps become `wps:wsp`+`a:blipFill` without an empty txBox (Writer paints a large empty text frame over later labels); logos that do not overlap stay `pic:pic`. Text-box Dark Mode underlay uses the topmost covering layer, so a full-page paper fill behind an SVG does not paint a white rectangle over the art. Invoice/text/corpus tests still pass.
- `export-docx`: folding a same-node DrawBox into the text box (pills) dropped the outline, so outlined badges lost their stroke. Fill, corner radius, and `a:ln` now fold together. `export-docx` / `export-pptx`: lock-wrapped body in a frame that only fits those lines gets hard line breaks (`wrap=none`) so a host-wider last word is not clipped; wrapped frames also set DrawingML overflow. Invoice/text/corpus tests still pass.
- `export-docx` / `export-pptx`: lock-wrapped text whose last line overflows the geometry box (box sized for N−1 lines) expands the Office frame to that ink and pins the last paragraph to face-size line spacing. Inter-line leading as trailing space on the last para was painting over the next node in PowerPoint and clipping the last line in Writer. Invoice/text/corpus tests still pass.
- `export-docx` / `export-pptx`: pinning lock wrap breaks no longer inserts a blank Office paragraph when the next lock line already follows an explicit `\n` (hero titles that hard-break, then wrap). Wrap-only paragraphs still get a single hard break. Invoice/text/corpus tests still pass.
- `export-docx` / `export-pptx`: explicit `\n` two-line labels in a padded cell (invoice line-items) stay `wrap=none`. A frame that still fits N+1 lines used to host-wrap the long second paragraph onto a third row over the next cell. Intra-paragraph lock wrap is unchanged. Invoice/text/corpus tests still pass.
## [0.2.3] - 2026-09-09

### Fixed

- Align published Rust (`k2f`, `k2f_sdk`), Python (`k2f`), and npm (`@openk2f/k2f`) with the template-bundling removal shipped on `main` after 0.2.2 (crates.io and PyPI 0.2.2 still exposed named template APIs).

## [0.2.2] - 2026-09-09

### Added

- `init_package.py` page presets `square` (1:1, 600pt), `portrait-45` (4:5), and `card` (US 3.5×2", 252×144pt), default margin 0. Same paged `ex_poster_shell` as slides; set `layout.height` to the page height. Duplex cards: two shells, back `break_before: page`, `--expect-pages 2`.
- `PAGE_UNDERFILL` compile diagnostic when a page content box is ≥25% empty at the bottom (not the last page of a multi-page flow). Warning only — `pack_verify.py` does not fail. Catalog [`ex_filled_page.json`](skills/k2f/catalog/content/ex_filled_page.json). `--render` also writes `preview-N.png` for extra pages.

### Changed

- `k2f pack`, `k2f compile`, and `Editor.save_bytes` coverage-subset large CJK faces in the package to GB2312 ∪ Big5 level 1 ∪ JIS X 0208 Han, plus all non-Han glyphs. Author directories keep the original face. Faces that would not drop any Han stay byte-identical. Missing Han still fails closed (`FONT_MISSING_GLYPH`).
- Agent skill: invoice/CV/flyer/poster are composed filled pages (`PAGE_UNDERFILL` = must-fix there). Contract/report/thesis is one flow tree — no `p1`/`p2` page containers; `break_before` on chapter/annex/signature is allowed. `LAYOUT_SLACK` stays warning-only.
- Theme roles other than `default` may omit `font_family` / `font_size` / `line_height_mult` / `color`; compile fills them from `default`. `default` still requires all four. Sheet fill remains the root role `box_decoration.background` (full page, including margins) — not `page_config.background`.
- `LAYOUT_SLACK` / `PAGE_UNDERFILL` skip miniature pages (content box shorter than 180pt) and unused gaps under 36pt, so card inset is not treated as a hollow grower. A4 / 16:9 growers are unchanged.
- `FONT_MISSING_GLYPH` names the uncovered code points and tells the author to `--add-font` a covering TTF/OTF (still fail-closed; no OS fallback). Package-embedded faces already fall back to each other.

### Fixed

- Grid `{auto:true}` tracks that would overflow the available axis now shrink in proportion instead of overlapping sibling columns/rows. Short auto content still hugs; leftover still goes to `fr`. Same rule on both axes.
- Root `layout` of `grid` / `overlay` / `columns` is a compile error (was a silent vertical-flow fallback). Nest under a child, e.g. `root.grid`.
- Linear gradient paint fails closed on unparseable stop colors (no silent one-stop / empty fill). `angle_degrees` is 0=right, 90=down — documented on the primitive schema and in the skill fills note (not CSS).
- SVG `<text>` `IMAGE_SIZE` message tells agents to drop the tags and use a K2F text node (or `<path>`), matching the skill error table.
- `export-docx` / `export-pptx`: one color path — lock sRGB only (`000000`/`FFFFFF` stored as `000001`/`FFFFFE` so Office does not snap them to the OS theme). Theme `dk1`/`lt1` are those same RGBs, not `sysClr`. Word `wps:style` fillRef is omitted. Word does not emit `w:tbl` (Dark Mode inverts `w:shd`); table cells are the same boxes + text as the rest of the page. PPTX keeps DrawingML tables; shape fills, strokes, slide backgrounds, and table cell/border RGB use the same pin. Invoice/text/corpus tests still pass.
- Four-edge box borders follow `corner_radius` in PNG/viewer (and dashed/dotted PDF), matching fill/shadow. Partial-edge borders stay straight.
- `export-docx`: LibreOffice Writer paints `pic:pic` above every DrawingML shape, so a full-page gradient/glass raster hid later text. Linear gradients and translucent solids are native `a:gradFill` / `a:solidFill`+`a:alpha`. A full-page fill is `behindDoc=1` (and `w:background` uses the first gradient stop) so it is not an in-front empty text frame covering labels. Remaining blur/shadow/math slices (`k2f-raster:`) use `wps:wsp` + `a:blipFill`. Text boxes skip the paper-white Dark Mode underlay when a gradient, glass, or raster already covers the box. Lock `DrawImage` that later paint overlaps also uses that `wps:wsp` path; non-overlapping logos stay `pic:pic`. Invoice/text/corpus tests still pass.

### Removed

- Named official templates from every published SDK (`official_templates`, `copy_template`, `resolve_template`, `Editor.open_template` / `openTemplate`). WASM no longer `include_dir`s `k2f/templates/`. Create with `init_package.py` / unpack / Gallery, then `Editor.open_dir` or `Editor.open` / `open_bytes`. CLI `k2f markdown --template` is a required author directory. JS `markdownToK2f` takes `templateBytes`.

## [0.2.1] - 2026-09-07

### Fixed

- npm `@openk2f/k2f` 0.2.0 shipped the viewer-only WASM as the SDK binary (identical `wasm/` and `wasm-viewer/` files), so `markdown_to_k2f` and `Editor` were missing. SDK and viewer-only builds now use isolated cargo target dirs.

## [0.2.0] - 2026-09-07

### Added

- Agent skill **design-first workflow** — write a design specification Markdown before authoring K2F JSON; implement `theme.json` + content from the spec, then render and iterate until the output matches.
- `k2f export-pptx <package> -o <out.pptx>` — draw the published lock into PowerPoint. Not a second layout engine; slight text reflow is expected. PPTX is not a K2F source (`PPTX_IS_NOT_A_SOURCE`).
- `k2f export-docx <package> -o <out.docx>` — draw the published lock into Word. Not a second layout engine; slight text reflow is expected. DOCX is not a K2F source (`DOCX_IS_NOT_A_SOURCE`).
- Desktop reader: HUD format **DOCX** and `k2f-reader --export-docx out.docx file.K2F` (same lock bytes as GUI Export; conflicts with `--export-pdf` / `--export-pptx`).
- `k2f compile` prints `LAYOUT_SLACK` when a large stretched box is empty at the bottom — usually a `{fr:1}` grower packed with an auto-height stack, not the page shell (warning, exit 0, lock unchanged).
- Grid tracks `{ "auto": true }` — content-sized from measured cells; leftover space goes to `fr`. Table `column_widths` stay `{pt}` / `{fr}` only.
- Optional grid `rows`: omit → `{auto:true}` tracks `ceil(n_children / n_columns)`. Declared rows do not grow. `fr` rows still need a finite outer height.
- Catalog [`ex_glass.json`](skills/k2f/catalog/content/ex_glass.json) — `card` variant `glass` via named `box_decoration.blur` and a translucent surface (not a radial gradient).
- Line wrap: hyphen-minus and U+00AD are break opportunities (hyphen stays at the line end). A word still wider than the line is character-split. No hyphenation dictionary, no auto-shrink.

### Changed

- Font load/render parse only `.ttf`/`.otf`. License/sidecar files under `assets/fonts/` stay in the ZIP and `Package.fonts` (appearance hash unchanged) but are skipped at `Face::parse`. Invalid faces report `FONT_INVALID` with the path, not `UnknownMagic`. Auto-`default` and “at least one font” count faces only. PDF/DOCX/PPTX export apply the same face-path filter (do not embed or resolve license `.txt` as a font).
- SVG `<text>` / `<tspan>` / `<textPath>` / `<foreignObject>` detection strips XML comments and CDATA first, then matches real elements. Pack and compile reject them (`IMAGE_SIZE`); paint still fail-closed. Agent skill writing Rule 4 states the same at the writing-loop gate.
- Agent skill: flowing articles use **one** unpadded `columns` container with `column_span: all` (do not `break_before: page` on figures). Variant text fields stay under `text_overrides`. Compact tables: widen columns, lower role `font_size`, or insert U+00AD.

- Agent skill: poster/slide page shell is a pinned-height **grid** with `{auto:true}` header/footer and a `{fr:1}` grower ([`ex_poster_shell.json`](skills/k2f/catalog/content/ex_poster_shell.json)); fill the grower with nested `{fr:1}` rows ([`ex_poster_growers.json`](skills/k2f/catalog/content/ex_poster_growers.json)), not an auto-height stack. Zero-padding vertical stacks split by child when the page remainder is too small.
- `export-docx` / `export-pptx`: pin Office theme `dk1`/`lt1` to RGB black/white instead of `sysClr windowText`/`window`. Text that is lock-black / lock-white is stored as `000001` / `FFFFFE` so Word/PowerPoint Dark Mode cannot treat it as Automatic and invert it on a still-white page. Word export now includes `word/theme/theme1.xml`, explicit `w:background`, `w14:textFill`, RGB `wps:style` fontRef, and an opaque text-box underlay matching the shape behind the text so unfilled boxes are not remapped in Dark Mode.
- `export-docx` / `export-pptx`: resolve lock font keys (package path stem such as `Roboto-Regular`, plus `default`) to the embedded TTF family name instead of leaking the alias into `typeface` / `w:rFonts`. Character tracking is taken from lock glyph extra-advance (Word `w:spacing`, DrawingML `a:rPr spc`). Role `padding_pt.top` on a text node becomes text-box `tIns` from the first-line glyph `y_offset` (skipped when the box is vertically centered).
- `export-docx` / `export-pptx`: disable text-frame wrap when the lock is a single line that already fills the box, **or when the frame is only tall enough for one line**. Host bold/metrics that are wider than rustybuzz were wrapping the last word onto a clipped second line (title rows, padded pills). Multi-line lock boxes and tall frames with leftover width still wrap. One-line frames pin exact line spacing to the lock font (hosts otherwise use ~12pt pitch and hide 7–9pt text) and set DrawingML `horzOverflow`/`vertOverflow` to `overflow`. Symmetric glyph padding is treated as center (not left+`lIns`) so host-wider labels keep slack on both sides. PPTX copies lock glyph left/right gaps into `lIns`/`rIns` for true left/right frames. Word folds a same-node DrawBox into the text box (corner radius + fill) so LibreOffice cannot paint the empty pill on top of the label.
- `export-docx` / `export-pptx`: explicit `\n` lines in a frame that only fits those lines stay `wrap=none` so a host-wider last line does not create a clipped extra row (footer event labels). Lock-wrapped paragraphs (more glyph lines than newline paragraphs) still wrap. Left-aligned lines that already fill ≥85% of the padded width drop `lIns`; short lines keep lock left padding. Invoice/text/corpus tests still pass.
- `export-docx` / `export-pptx`: emit Office hyperlinks only when a `link` modifier intent is a URI. Theme keys such as `default` stay underline-from-lock and are not `hlinkClick`/`w:hyperlink` targets. Hosts that still wrap real URLs ignore run fill and paint theme `hlink` (blue); pin that slot, Word's Hyperlink style, and underline color to the lock run RGB.
- `export-docx`: LibreOffice Writer paints pictures above DrawingML shapes regardless of `behindDoc`, so a later opaque container hid stamps and address text. Large fill boxes now use `behindDoc=1`; thin fills (1 pt rules and few-pt accent bars, min side ≤ 8 pt) stay in front; images fully covered by a later opaque lock box are omitted (they are invisible in the lock). Landscape `w:pgSz` sets `w:orient="landscape"`. A large fill that also has a four-sided stroke (drawing frames, card shells) splits: fill stays behind, outline is a separate in-front shape so Writer does not hide the border under `w:background`.
- `export-docx` / `export-pptx`: cluster lock glyphs into lines with a font-relative y-tolerance (half the face, 2–7.5pt) instead of a fixed 7.5pt. 5–7pt wrapped body was merging two lines, so wrap and line pitch followed host defaults. 11–12pt tests keep the old cap.
- `export-pptx`: native table cells follow lock borders (explicit `a:noFill` when the lock has none) instead of a fake four-side `#D0D0D0` 0.5pt grid. Cell paragraph align is inferred from lock glyphs, matching Word. Invoice/text/corpus tests still pass.
- `export-docx` / `export-pptx`: `emphasis` intent `italic` sets italic, not bold. Re-applying every emphasis as bold made lock-italic runs (keywords, captions) export as bold-italic.
- `export-docx`: static running-header/footer paint (logos, labels without `{{page_*}}`) is drawn in the body at lock coordinates. LibreOffice Writer does not paint `header1.xml` / `footer1.xml` when `pgMar` header/footer is 0. PAGE/NUMPAGES fields stay in the footer part so invoice page numbers are not duplicated.
- `export-docx`: native table row height follows lock y-delta (cell box plus `gap`), matching PPTX. Cell-only height plus `atLeast` was collapsing gapped rows (cover meta grids, invoice line items) in Writer. Invoice/text/corpus tests still pass.
- `export-docx`: large fills use `behindDoc=1` only when they match the page paper color (white containers covering stamps). Contrasting shells (cover body, yellow band) stay in front as empty text boxes so LibreOffice does not hide them under white `w:background` and does not paint the fill over later labels. Thin fills and fill/stroke split are unchanged. Invoice/text/corpus tests still pass.
- `export-docx` / `export-pptx`: pin text-frame size with DrawingML `a:noAutofit` so Word/PowerPoint do not shrink a page-width center box to the glyph span and leave it on the left origin. Center/justify frames that still have leftover width (not glyph-tight pills) use `wrap=square` so hosts honor `w:jc` / `algn`. Left one-liners and ≥85%-full pills stay `wrap=none`. Invoice/text/corpus tests still pass.
- `export-docx` / `export-pptx`: empty in-front fill shells (cover body, accent band) also pin `a:noAutofit` and `wrap=square`. Word was size-to-fitting those empty `wrap=none` txBoxes to a leftover strip on the left while label underlays stayed full width. Behind-doc paper fills are unchanged. Invoice/text/corpus tests still pass.

### Removed

- Agent skill **`looks/`** — removed bundled design skins (`quiet-light`, `vivid-blocks`, `night-wash`) and `verify_looks.py`; styling is authored from the design spec instead

## [0.1.2] - 2026-08-30

### Added

- **`pip install k2f` ships the CLI** — `k2f` console script on PATH (pack, compile, verify, render, export-pdf, markdown); agents no longer need `cargo install k2f`

### Changed

- Agent skill install guidance: **prefer `pip install k2f`**; `cargo install k2f` is optional for Rust-only environments

## [0.1.1] - 2026-08-30

### Added

- Agent skill **`looks/`** — three example design skins (`quiet-light`, `vivid-blocks`, `night-wash`): complete `theme.json`, composition guides, and shared extension roles (`display`, `kicker`, `caption`, `metric`, `shell`). Agents rewrite a package theme from these examples when the user did not specify a style.
- `Editor.set_running_header` / `set_running_footer` for manifest running blocks

### Changed

- Unified six agent skills into one `skills/k2f/` skill with workflow references (writing, converting-markdown, exporting-pdf, publishing, embedding-viewer)
- Agent skill: merged **generating** and **editing** into a single **writing** workflow (JSON + `k2f unpack` / `pack_verify.py`); optional Python `Editor` moved to `references/writing/sdk.md`
- Public creation path is `Editor.open_template` / `open_dir` / `open_bytes` plus `insert_node` JSON

### Removed

- Public `Composer` / `Document` builder (`add_heading`, `add_body`, `add_warning`, `add_table`)

## [0.1.0] - 2026-08-28

### Added

- K2F format specification v0.1 (ZIP package, semantic tree, theme, compiled lock)
- Reference engine: layout, paint, PDF export, package validation, Ed25519 signing
- CLI (`k2f`) — pack, compile, verify, sign, render, export-pdf, markdown
- Python SDK (`pip install k2f`) — Document, Editor, Markdown bridge
- JavaScript / WASM SDK (`npm i @openk2f/k2f`) — Document, Editor, `<k2f-viewer>` web component
- MCP stdio server (`k2f_mcp`) with tool catalog in `mcp_tools.json`
- Six agent skills under `skills/`
- Public documentation under `docs/` and integrity policy in `SECURITY.md`

[0.1.2]: https://github.com/suzheng/k2f/releases/tag/v0.1.2
[0.1.1]: https://github.com/suzheng/k2f/releases/tag/v0.1.1
[0.1.0]: https://github.com/suzheng/k2f/releases/tag/v0.1.0
[0.2.0]: https://github.com/suzheng/k2f/releases/tag/v0.2.0
[0.2.1]: https://github.com/suzheng/k2f/releases/tag/v0.2.1
