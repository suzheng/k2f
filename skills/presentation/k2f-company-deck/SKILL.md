---
name: k2f-company-deck
description: Turn company facts, brand inputs, and stakeholder goals into a polished company overview deck with clear positioning, proof, and contact-ready closure. Use when the user asks for a company deck, corporate overview, company profile presentation, or brand deck (not a fundraising pitch or sales proposal). Create company decks with AI-native design and the highest aesthetic standards, powered by the K2F format and engine. Purpose-built for AI agents, K2F lets them focus on semantic content and design intent while automatically handling layout, typography, and formatting complexity.
---

# K2F Company Deck — AI-Native Aesthetic Design

## Purpose

Teach agents to blueprint **company decks** before layout: a stakeholder-ready story that explains who the company is, what it delivers, and why it matters—without inventing funding, customers, metrics, or leadership the user did not supply.

## Built on K2F

This skill produces company decks as `.K2F` packages. Follow the **k2f** skill for authoring: unpack or init a workspace, edit the semantic tree, pack/verify, and export. K2F keeps agents on *meaning* (slide roles, narrative, visual intent)—not coordinates—while the engine compiles layout, typography, and theme into export-ready output.

Before you build, install the core skill: `npx skills add suzheng/k2f --skill k2f`.

## When to Use

- Introducing the company to partners, enterprise buyers, recruits, press, or board guests (no investment ask).
- Consolidating website copy and bios into one coherent overview.
- Refreshing a generic “about us” deck for skimmable positioning and contact in under ten minutes.

## Workflow

### Step 1. Generate the content_and_design.md

Single blueprint for semantic content and aesthetic intent. Use labeled blocks per slide (role, headline, body, visual type) that the **k2f** skill can map later.

**Sub-steps**

1. **Deck frame** — Primary audience (partner, customer, talent, media), setting (first meeting, conference, onboarding), and time budget. Choose emphasis: **credibility + capabilities** for B2B; **mission + culture** for talent; **news + proof** for press. Drop fundraising slides unless the user explicitly wants a hybrid.

2. **Input audit** — Inventory verified inputs: positioning, products, public pricing, permitted logos, milestones, team, partnerships, news. Flag `NEEDS USER INPUT` for gaps—never fabricate headcount, revenue, clients, or awards.

3. **Narrative spine** — Plan flow before titles: **identity** → **problem space** → **offer** → **how it works** → **proof** → **difference** → **people & scale** → **direction** (vision, not roadmap dump) → **engage**. Opening = name + tagline; closing = one CTA.

4. **Slide architecture (include / cut)** — Default overview stack (~12–16 slides); merge or omit when inputs are thin:
   - Title → Company snapshot → Mission / values (optional) → Problem & opportunity (light) → Solutions / product lines → How we work or platform → Customers & outcomes → Differentiation → Team → Milestones / footprint → Partners or ecosystem (optional) → Vision → Contact.
   - **Cut** investor-only blocks (TAM essay, use-of-funds, cap table), duplicate product slides, long feature lists, and internal org charts unless audience is recruits.
   - **Split** when identity and product story share one slide, or when two products each need a hero proof point.

5. **Per-slide content contract** — For each slide specify:
   - **Headline**: outcome or position, not a section label (“Payments infrastructure for internet businesses” not “About us”).
   - **Body**: max 3–5 bullets or one visual; ~2 lines per bullet; parallel grammar on lists.
   - **Visual type**: logo strip, product frame, workflow diagram, metric callouts, timeline, team grid, map—prefer **visual intent** over paragraphs for comparisons, milestones, and portfolios.
   - **Footnotes**: source + year for external claims; logo approval notes if needed.

6. **Proof layering** — Permitted logos or named references first; anonymize if required. Scale metrics only when user-supplied; else qualitative proof with TBD.

7. **Design intent** — For the engine: **tone** institutional, from user brand voice—not generic hype; **hierarchy** one hero per slide, calm contact block; **density** one message per slide, grids for portfolios; **palette** user colors or neutral + one accent, WCAG AA, consistent logos/timelines.

**Rules & constraints**

- **Audience gate**: every slide must earn its place for the chosen audience; recruit slides ≠ partner slides unless user wants one master deck.
- **Truth gate**: metrics, customers, funding, and headcount match user materials or are marked TBD—no rounded-up fiction.
- **One-idea gate**: if the headline needs “and,” split or cut.
- **Brand gate**: tagline, product names, and legal entity spelling identical everywhere.
- **Logo gate**: only logos the user confirmed; otherwise “category examples” or omit.

**Common failure modes**

- Pitch deck as company deck → drop ask, financials, TAM; strengthen identity and contact.
- Website walls of text → headline + proof only; detail in blueprint notes.
- Feature lists without outcomes → tie capabilities to customer results.
- Weak differentiation → two-column comparison or cut.
- Long team bios → name, role, one line each.
- Cluttered closing → one CTA plus minimal contact.

**Quality checklist**

- [ ] Tagline and company snapshot appear in the first three slides; contact/CTA on the last slide.
- [ ] Every slide has one headline claim and a declared visual type.
- [ ] No slide exceeds density limits; external stats have sources or TBD flags.
- [ ] Product and company names spelled consistently; repeated metrics match.
- [ ] Audience-irrelevant sections removed or moved to optional appendix notes in the blueprint.
- [ ] Design intent names tone, hero emphasis, and accent usage for the full deck.

### Step 2. Use the content_and_design.md and k2f skill to generate a K2F file and export to an appropriate file format.

Follow **k2f** to instantiate slides from the blueprint, apply roles from design intent, verify, and export the stakeholder-ready deliverable.
