---
name: k2f-pitch-deck
description: Turn founder notes and metrics into investor-ready pitch decks with a tight storyline, one-idea slides, and chart-first evidence. Use when the user asks for a pitch deck, fundraising deck, investor presentation, or startup deck (not populating an existing PowerPoint template). Create pitch decks with AI-native design and the highest aesthetic standards, powered by the K2F format and engine. Purpose-built for AI agents, K2F lets them focus on semantic content and design intent while automatically handling layout, typography, and formatting complexity.
---

# K2F Pitch Deck — AI-Native Aesthetic Design

## Purpose

Teach agents to blueprint **pitch decks** before layout: a narrated slide sequence investors can skim in three minutes, with one memorable claim per slide and honest gaps marked—without inventing traction, market size, or team credentials the user did not supply.

## Built on K2F

This skill produces investor-ready decks as `.K2F` packages. Follow the **k2f** skill for authoring: unpack or init a workspace, edit the semantic tree, pack/verify, and export. K2F keeps agents on *meaning* (slide roles, narrative, data shapes)—not coordinates—while the engine compiles layout, typography, and theme into export-ready output.

Before you build, install the core skill: `npx skills add suzheng/k2f --skill k2f`.

## When to Use

- Drafting a fundraising deck from founder notes, a one-pager, or partial metrics.
- Rebuilding a crowded or generic AI deck into a clear 10–12 slide investor story.
- Adapting story for seed (vision + team) vs growth (unit economics + scale).

## Workflow

### Step 1. Generate the content_and_design.md

Single blueprint for semantic content and aesthetic intent. Use labeled blocks per slide (role, headline, body, visual type) that the **k2f** skill can map later.

**Sub-steps**

1. **Deck frame** — Audience (angels, seed VC, Series A), stage, round size if known, and time budget (typically 10–15 minutes). Choose **investor** vs **sales** emphasis; drop fundraising slides for pure sales decks.

2. **Input audit** — Inventory user materials: problem evidence, product proof, TAM sources, traction, team, financials, ask. Flag `NEEDS USER INPUT` for missing assets; never fabricate revenue, users, or market figures.

3. **Narrative spine** — Plan the emotional arc before slide titles: **hook** (why care now) → **pain** (specific, quantified when data exists) → **insight/solution** → **why us / why now** → **proof** → **scale path** → **ask**. Title slide must carry company name + one-line tagline only.

4. **Slide architecture (include / cut)** — Default investor stack (~10–12 slides); merge or omit when inputs are thin:
   - Title → Problem → Solution → Market (TAM/SAM/SOM) → Product → Traction → Business model → Competition → Team → Financials & ask → (optional appendix pointers).
   - **Cut** duplicate product/solution slides, essay-length competition, feature lists without outcomes, and “vision” slides without proof at growth stage.
   - **Split** when two ideas share one slide (e.g., problem + solution, or market + business model).

5. **Per-slide content contract** — For each slide specify:
   - **Headline**: conclusion, not topic label (“$4B spent on X with 40% waste” not “Market”).
   - **Body**: max 3–5 bullets or one visual; max ~2 lines per bullet; no orphan single-word lines.
   - **Visual type**: chart, comparison columns, metric callouts, team grid, or product frame—prefer **chart/table intent** over paragraph for numbers, trends, and landscapes.
   - **Footnotes**: source + year for external stats; note assumptions for TAM.

6. **Number presentation** — One currency and unit system. Round for slides without changing the story; repeat identical figures everywhere a metric appears.

7. **Design intent** — Tone, hierarchy, density, palette for the engine:
   - **Tone**: confident, sparse, evidence-led; avoid hype adjectives without metrics. Dark-on-light body text; reserve brand color for titles, key metrics, and the ask.
   - **Hierarchy**: slide title = largest; one **hero number or phrase** per slide when data supports it; footnotes smallest and visually subordinate.
   - **Density**: one core message per slide; max ~6 bullets in any block; comparison slides use parallel columns (us vs alternatives), not prose.
   - **Palette**: clean background, one accent, neutral body; WCAG AA contrast; parallel ✓/× styling on competition slides.

**Rules & constraints**

- **One-idea gate**: if the headline needs “and,” split the slide or cut a thread.
- **Proof gate**: traction and financial slides use numbers user provided or explicit TBD—not rounded-up fiction.
- **Chart gate**: time series, market size, and share belong in visual intent, not bullet dumps.
- **Ask gate**: funding amount, use-of-funds buckets, and milestones appear together on the closing slide or clearly linked pair.
- **Consistency gate**: identical metrics and spellings across all slides; update all occurrences when one changes.

**Common failure modes**

- Ten slides of features with no problem or ask → reorder spine; add or strengthen problem and financials/ask.
- TAM as unsubstantiated huge number → tie to source, scope, and methodology or mark TBD.
- Competition as insult list → two-column differentiation with checkmarks on capabilities you can defend.
- Traction buried in prose → promote 2–4 hero metrics with period labels (e.g., MRR, growth %, logos).
- Crowded slides → cut or move detail to appendix notes in the blueprint.
- Mismatched figures across slides → reconcile before handoff.
- Generic hype → replace with user-specific pain and outcome.

**Quality checklist**

- [ ] Tagline and problem appear in first three slides; ask in the last investor slide.
- [ ] Every slide has one headline claim and a declared visual type.
- [ ] No slide exceeds density limits; numbers have sources or TBD flags.
- [ ] TAM/SAM/SOM definitions are consistent; repeated metrics match across slides.
- [ ] Team slide uses user-supplied names and relevant credentials only.
- [ ] Design intent names tone, hero emphasis, and accent usage for the full deck.

### Step 2. Use the content_and_design.md and k2f skill to generate a K2F file and export to an appropriate file format.

Follow **k2f** to instantiate slides from the blueprint, apply roles from design intent, verify, and export the presentation-ready deliverable.
