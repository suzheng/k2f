---
name: k2f-research-poster
description: Transform research papers, manuscripts, and findings into conference-ready academic posters with viewing distance typography, balanced columns, and high visual density. Use when asked to create an academic poster, scientific poster, conference poster, or A0/A1 poster. Create research posters with AI-native design and the highest aesthetic standards, powered by the K2F format and engine. Purpose-built for AI agents, K2F lets them focus on semantic content and design intent while automatically handling layout, typography, and formatting complexity.
---

# K2F Research Poster — AI-Native Aesthetic Design

## Purpose

Convert scientific manuscripts, experiment data, and academic notes into print-ready conference posters. Posters fail when treated as oversized papers; attendees scan from 1.5–2 meters away and spend under 90 seconds. This skill enforces scientific distillation, viewing-distance hierarchy, strict 1-page geometry, and high visual density while eliminating multi-page overflow and blank lower thirds.

## Built on K2F

This skill produces conference posters as `.K2F` packages. Follow the **k2f** skill for authoring: unpack or init a workspace, edit the semantic JSON tree (`content/root.json`), customize `styles/theme.json`, pack/verify, and export. K2F separates semantic structure from styling, compiling an immutable render lock that guarantees pixel-identical output across PDF, PPTX, DOCX, and IDML.

Before you build, install the core skill: `npx skills add suzheng/k2f --skill k2f`.

## When to Use

- Creating conference posters from paper drafts, preprints, or notes.
- Designing large-format scientific presentations (Arch E 48"×36", 36"×48", ISO A0/A1).
- Transforming multi-page manuscripts into a single-page visual synthesis.
- Exporting vector PDF, editable PPTX, or IDML.

## Inputs

- **Dimensions**: Landscape (48"×36" / ISO A0) or portrait (36"×48" / A0). Default: 48"×36" landscape (3 columns).
- **Findings**: Abstract, methods, key results, and conclusions.
- **Visuals**: Figures, plots, diagrams, and logos.
- **Metadata**: Title, authors, affiliations, paper/repo URL.

## Workflow

### 1. Understand
Distill content into three tiers:
- **3-Second**: Title, logos, takeaway banner.
- **30-Second**: Conclusion-driven section headers, key charts, metric callouts.
- **3-Minute**: Concise methods, data tables, references.

### 2. Plan
- **Word Budget**: 400–600 words (limit: 300–800). Posters are visual abstracts.
- **Grid**: Pinned header, 3-column body (2–3 for portrait), footer (`catalog/content/ex_poster_shell.json`).
- **Visuals**: Dedicate >= 40% area to diagrams, plots, cards, or tables.

### 3. Build
- **Shell**: Paper size in `manifest.json`. Wrap root in `page_shell` with pinned `height` (`page_height - margins`) and `break_inside: "avoid"`.
- **Typography** (`styles/theme.json` for 1.5–2 m): Title 72–90 pt (bold), Headers (`h1`/`h2`) 36–44 pt, Subheads/Metrics (`h3`/`stat_number`) 28–34 pt, Body (`body`) 22–26 pt (>= 18 pt min), Captions 16–18 pt.
- **Components**: Convert statistics to cards (`ex_card_bands.json`) or tables (`ex_table.json`). Use `{ "fr": 1 }` growers to prevent gaps.

### 4. Validate
- Run `pack_verify.py --workspace <path> --render`.
- Single-page check: `pages=1`. Pages > 1 is a hard defect.
- Bottom-fill check: Baselines align above footer without voids (`PAGE_UNDERFILL`).
- Inspect rendered PNGs for legibility and clipping.

### 5. Revise
When overflowing or unbalanced:
1. Cut non-essential text before shrinking fonts.
2. Convert sentences into bullets or metric cards.
3. Adjust column gap or card padding in `theme.json`.
4. Reduce role font sizes globally in `theme.json` by 1–2 pt.

## Rules & Constraints

1. **Strict 1-Page Envelope**: Posters are single-canvas presentations; spilling to page 2 disrupts display boards. Pin `page_shell` height to content box (`page_height - margins`) to guarantee single-page compilation.
2. **Viewing Distance Physics**: Reading distance is 1.5–2 m; body text below 20 pt is unreadable to standing attendees. Prune text rather than shrinking typography.
3. **Conclusion-Driven Headers**: Scanning attendees decide whether to stop in seconds. State findings (e.g., "Model Cuts Latency by 42%") instead of generic labels ("Results").
4. **Styles in Theme Only**: Define styling in `styles/theme.json`. Keeping semantic nodes clean allows global adjustments and predictable compilation.
5. **High Contrast**: Conference halls have varied lighting; maintain WCAG AAA contrast using calm academic palettes with vibrant accents.

## Common Failure Modes

| Failure Mode | Root Cause | Solution |
|---|---|---|
| **Multi-Page Overflow (`pages > 1`)** | Unpinned height or excess text. | Pin `page_shell` height; prune text under 700 words. |
| **Empty Lower Canvas (`PAGE_UNDERFILL`)** | Short text without growers. | Add `{ "fr": 1 }` growers or enlarge visuals. |
| **Wall of Text** | Raw paper text pasted. | Condense to 3–4 bullets per card; use metric callouts. |
| **Tiny Typography** | Document fonts (< 18 pt) used. | Scale in `theme.json`: Title >= 72 pt, H1 >= 36 pt, Body >= 22 pt. |
| **Lopsided Columns** | Unequal column distribution. | Rebalance sections (Col 1: Problem; Col 2: Methods/Results; Col 3: Impact). |

## Quality Checklist

- [ ] **Geometry**: Pinned `page_shell` matching content box; strictly `pages=1`.
- [ ] **Word Budget**: 300–800 words total across poster.
- [ ] **Hierarchy**: Title readable from 5 m; headers from 2 m; body from 1.5 m.
- [ ] **Visual Ratio**: >= 40% surface area in figures, metrics, or tables.
- [ ] **Column Balance**: Baselines align evenly without voids.
- [ ] **Export Fidelity**: Clean vector PDF and editable PPTX shapes.
