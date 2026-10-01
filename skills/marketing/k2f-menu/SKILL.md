---
name: k2f-menu
description: Turn venue notes, dish lists, and brand cues into a scannable food or beverage menu with clear categories, consistent item anatomy, and mood-matched visual intent—not a flyer, recipe book, or website sitemap. Use when the user asks for a restaurant menu, café menu, bar or wine list, catering sheet, room-service card, tasting menu, or seasonal insert for hospitality print or PDF. Create menus with AI-native design and the highest aesthetic standards, powered by the K2F format and engine. Purpose-built for AI agents, K2F lets them focus on semantic content and design intent while automatically handling layout, typography, and formatting complexity.
---

# K2F Menu — AI-Native Aesthetic Design

## Purpose

Teach agents to author `content_and_design.md` for **hospitality menus**: category and item structure, inclusion cuts, and aesthetic intent (tone, hierarchy, density)—not K2F packaging or export.

## Built on K2F

This skill produces menus as `.K2F` packages. Follow the **k2f** skill for authoring: unpack or init a workspace, edit the semantic tree, pack/verify, and export. K2F keeps agents on *meaning* (sections, dishes, prices, dietary notes)—not pixel coordinates—while the engine compiles layout, typography, and theme into consistent output across PDF and other formats.

Before you build, install the core skill: `npx skills add suzheng/k2f --skill k2f`.

## When to Use

- Restaurants, cafés, bars, bakeries, food trucks, hotels, catering, or private dining.
- Boards, folded inserts, food/drink splits, wine or cocktail lists, kids or brunch add-ons.
- Print or digital PDF—not full websites or POS exports.

## Workflow

### Step 1. Generate the content_and_design.md

One blueprint pairing **information architecture** with **design intent**. Long menus: one file per major section slug (e.g. `content_and_design_mains.md`).

**Sub-steps**

1. **Menu lock** — Venue type, service style, daypart, language(s), currency, brand constraints. **Guest job**: pick a dish or drink quickly while seated or at a board.

2. **Section map** — Order categories as guests decide (starters → mains → desserts → drinks, or bar-first). Label each **Open**, **Featured** (2–6 heroes), or **Omit** (verbal, QR, other card). Split food/drinks or lunch/dinner when one scan path fails.

3. **Item anatomy** — Consistent fields: **Name**, **Description** (length scales with venue), **Price** (one grammar site-wide), **Tags** (only user-supplied or required), **Modifiers** (sizes, add-ons) nested under one parent—not duplicate dishes.

4. **Content tiers** — **Identity** (name, logo direction), **Navigation** (section titles), **Catalog** (items + prices), **Guidance** (one short chef or pairing note max), **Compliance** (allergens, service charge). Keep essays out of Catalog.

5. **Genre fit** — Fine dining: brevity, seasonal focus. Casual: moderate density, combos. Bar/wine: column-friendly; glass | bottle. Hotel/catering: packages, per-person lines, tray or service notes in Compliance.

6. **Cut pass** — Drop duplicates, unpriced market items without confirmation, mismatched price formats, and naming inconsistency within sections.

7. **Design intent** (no coordinates) — **Tone**: one adjective pair from cuisine/venue cues. **Hierarchy**: sections > item names > descriptions; prices in one scan band (column or dot leaders). **Density**: board vs booklet vs fine dining whitespace; note two-column intent for bar lists. **Brand**: contrast on photos/dark boards (panel intent); 2–3 colors; display vs detail roles. **Channel**: print header scale vs mobile PDF section breaks.

**Rules & constraints**

- Prices and names scannable within ~10 s per section.
- One price grammar; each dish listed once.
- No invented dishes, prices, or allergen claims; label placeholders.
- Compliance stays short unless the user requires more.

**Common failure modes**

| Failure | Fix |
|--------|-----|
| Flyer creep | Promos → separate flyer; menu = Identity + Catalog. |
| Adjective walls | One description line; move story to Guidance. |
| Price chaos | Unify format; state alignment intent. |
| Category soup | Merge small sections; Featured for orphans. |
| Website paste | Strip nav/SEO; keep menu tiers only. |

**Quality checklist**

- [ ] Menu lock and guest job; section map with Open/Featured/Omit.
- [ ] Item anatomy + unified prices; tiers assigned.
- [ ] Genre density; design intent (tone, hierarchy, density, channel).
- [ ] Facts traceable to user input; placeholders marked.

### Step 2. Use the content_and_design.md and k2f skill to generate a K2F file and export to an appropriate file format.

Follow **k2f** to map the blueprint into the semantic tree, apply theme roles from design intent, verify, and export (typically PDF).
