---
name: k2f-idml
description: Blueprint multi-page print publications—brochures, magazines, catalogs, annual reports, lookbooks—for designer handoff with spread-aware story flow, typed editorial blocks, and art direction before layout. Use when the user asks for InDesign, IDML, editorial layout, print collateral, or an editable package for a production designer. Create idmls with AI-native design and the highest aesthetic standards, powered by the K2F format and engine. Purpose-built for AI agents, K2F lets them focus on semantic content and design intent while automatically handling layout, typography, and formatting complexity.
---

# K2F Idml — AI-Native Aesthetic Design

## Purpose

Teach agents to author `content_and_design.md` for publications meant to open in InDesign: explicit copy and assets, spread-level story beats, typographic roles, and art direction—not a slide deck pasted into pages or a wall of undifferentiated body text.

## Built on K2F

This skill produces print-ready packages as `.K2F` documents exportable to IDML. Follow the **k2f** skill for authoring: unpack or init a workspace, edit the semantic tree, run pack/verify, then export. K2F keeps agents on *meaning* (stories, figures, hierarchy roles)—not frame coordinates—while the engine compiles layout and typography into an editable InDesign handoff.

Before you build, install the core skill: `npx skills add suzheng/k2f --skill k2f`.

## When to Use

- New or refreshed brochures, magazines, catalogs, annual reports, program books, or brand lookbooks from briefs or draft copy.
- Restructuring long marketing copy into scannable editorial spreads before a designer refines in InDesign.
- Delivering native text and images to production—not a flattened PDF or full-page screenshot workflow.

## Workflow

### Step 1. Generate the content_and_design.md.

One blueprint merging **reader path**, **page/spread roles**, **typed content blocks**, and **design intent** the engine can interpret. Use labeled sections; map spreads later in k2f.

**Sub-steps**

1. **Publication scenario** — Audience, occasion (launch, conference, retail shelf), reading mode (skim vs study), approximate page count, binding or fold (saddle, perfect, tri-fold), and what must stay verbatim (brand claims, legal, pricing). Drop stories that do not serve the scenario.

2. **Document skeleton** — Ordered parts: cover (and optional wrap), front matter (TOC, letter, credits), body sections/chapters, back matter (index, colophon, contact). Note which spreads are **opening**, **feature**, **text-heavy**, **gallery**, or **closing** so rhythm is not uniform page-to-page.

3. **Content inventory** — Per spread or section, list blocks with stable ids and roles: display headline, deck/subhead, body (with approximate word budget), pull quote (+ attribution), sidebar/fact box, figure (+ caption + credit), table (title + column semantics), label/caption, CTA or contact strip. Mark **must-run** user copy vs **agent-drafted** placeholders the user must replace.

4. **Information architecture** — One dominant idea per opening spread; support with secondary blocks, not competing headlines. Sequence: hook → proof → detail → action. Pair every figure with caption on the same spread; keep related sidebar beside the paragraph it annotates. For catalogs, group by category with consistent product fields (name, spec, price, SKU)—do not mix narrative essays between SKU grids.

5. **Design intent** — Tone (luxury quiet, newsroom energetic, institutional sober), typographic hierarchy (display vs deck vs body vs caption sizes and weights—relative, not points), density (airy feature spreads vs tight reference pages), column intent (single narrative column vs multi-column body vs full-bleed hero), color (dominant neutrals + one accent for rules, labels, CTAs), imagery (subjects, crop style, negative space, when text overlays photo vs sits beside). State contrast target (readable captions on photos; no low-contrast gray body).

**Rules & constraints**

- **Spread thinking**: plan left–right pairs as a unit; avoid orphan headlines at the foot of a spread.
- **Role discipline**: each block has one role; do not stack three headline-sized lines without a clear primary.
- **Word budgets**: cap body per spread; move overflow to the next spread or a sidebar—never shrink type in the blueprint.
- **Verbatim gate**: trademarks, disclaimers, prices, and legal lines only from user sources.
- **Designer handoff**: prefer editable text and discrete figures over describing “designed as one image.”

**Common failure modes**

- Slide-deck pacing (title + bullets every page) → assign editorial roles and varied spread types.
- Uniform template fatigue → alternate feature, text, and gallery spreads.
- Captions separated from figures → lock figure+caption pairs in the inventory.
- Missing entry points → add pull quotes, decks, or section openers for skimmers.
- Generic stock-photo direction → specify subject, mood, and crop per slot.
- Catalog noise → separate storytelling spreads from SKU grids; consistent field order.
- Invented prices, awards, or compliance text → flag gaps; never fabricate.

**Quality checklist**

- [ ] Skeleton lists every major part; each body section has spread roles assigned.
- [ ] Every figure/table has caption, credit when needed, and same-spread pairing.
- [ ] Headlines are unique and ranked (one primary per spread).
- [ ] Word budgets sum to the stated page count without cramming.
- [ ] Verbatim copy is quoted or flagged; no silent invention of legal or pricing.
- [ ] Design intent covers tone, type roles, density, columns, color, and imagery without coordinates.
- [ ] Closing spread delivers the scenario CTA (contact, QR context, next step).

### Step 2. Use the content_and_design.md and k2f skill to generate a K2F file and export to an appropriate file format.

Follow **k2f** to instantiate spreads from the blueprint, bind content roles from the inventory, verify, and export the IDML package for InDesign.
