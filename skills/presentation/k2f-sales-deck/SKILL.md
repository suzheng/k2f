---
name: k2f-sales-deck
description: Turn discovery notes, proposals, and product proof into buyer-ready sales decks with a prospect-centered story, one objective per meeting, and evidence-backed slides. Use when the user asks for a sales deck, demo presentation, proposal deck, executive briefing, or sales slides (not a fundraising pitch deck or generic company overview). Create sales decks with AI-native design and the highest aesthetic standards, powered by the K2F format and engine. Purpose-built for AI agents, K2F lets them focus on semantic content and design intent while automatically handling layout, typography, and formatting complexity.
---

# K2F Sales Deck — AI-Native Aesthetic Design

## Purpose

Teach agents to blueprint **sales decks** before layout: one meeting, one deal step—with the prospect as hero, pain in their words, proof tied to outcomes—without inventing ROI, logos, or results the user did not supply.

## Built on K2F

This skill produces sales decks as `.K2F` packages. Follow the **k2f** skill for authoring: unpack or init a workspace, edit the semantic tree, pack/verify, and export. K2F keeps agents on *meaning* (slide roles, narrative, visual intent)—not coordinates—while the engine compiles layout, typography, and theme into export-ready output.

Before you build, install the core skill: `npx skills add suzheng/k2f --skill k2f`.

## When to Use

- Building discovery, demo, proposal, or executive-briefing decks from call notes or CRM context.
- Personalizing a template deck for a named account and buying stage.
- Replacing feature-heavy slides with problem–solution–proof flow and a clear next-step ask.

## Workflow

### Step 1. Generate the content_and_design.md

Single blueprint for semantic content and aesthetic intent. Use labeled blocks per slide (role, headline, body, visual type) that the **k2f** skill can map later.

**Sub-steps**

1. **Meeting frame** — One type and **one objective**: discovery, demo, proposal, or executive briefing. Record audience mix, buying stage, objections, and desired next step—never multiple meeting goals in one deck.

2. **Input audit** — Inventory verified inputs: prospect name/context, pains in their language, scope, pricing rules, approved case studies, product screenshots, competitive facts. Flag `NEEDS USER INPUT` for missing proof; never fabricate metrics, customer names, or ROI.

3. **Narrative spine** — Choose a spine matched to the meeting:
   - **Problem–solution–proof** (default): their world today → better future with your approach → why you (credibility).
   - **SPIN-style demo**: situation recap → articulated problem → cost of status quo → payoff tied to their use cases.
   - **Executive briefing**: why we’re here → what we learned → recommendation → investment/timeline → discussion.
   Prospect is the hero; your product is the guide—lead with “you” outcomes, not company history.

4. **Slide architecture (include / cut)** — Map slides to the spine; typical counts: discovery 5–10, demo 5–15, proposal 10–20, executive 10–15.
   - Core path: title → agenda (3–5 items, end with next steps) → their challenge → approach/outcomes → proof → demo/product (if needed) → investment/ROI → mutual next steps.
   - **Cut** opening company history, generic “best in class” claims, full feature tours, duplicate problem/solution slides, and investor-only blocks (TAM, cap table).
   - **Split** when two beats share one slide (e.g., problem + recommendation, or price without value context).

5. **Per-slide content contract** — For each slide specify:
   - **Headline**: the point of the slide in buyer terms (“Cut onboarding time 40% for teams like yours” not “Our platform”).
   - **Body**: max 3–5 bullets or one visual; parallel structure on comparisons; quote their words on problem slides when available.
   - **Visual type**: workflow diagram, product frame, logo + outcome strip, ROI comparison, timeline, mutual action plan—use **chart/callout intent** for quantified impact, not paragraph dumps.
   - **Proof notes**: label each stat Measured / User-provided / TBD; similar industry/use case for case studies; anonymize if required.

6. **Audience depth** — Economic buyers: outcomes, ROI, risk, timeline first. Technical buyers: integration/fit slides or appendix pointers, not the whole deck. Mixed room: segment beats (“For leadership… / For your team…”) without doubling slide count.

7. **Design intent** — For the engine:
   - **Tone**: consultative, credible, personalized—no stock hype; mirror prospect formality.
   - **Hierarchy**: headline states the takeaway; one **hero metric or phrase** per slide when proof exists; subordinate footnotes for sources.
   - **Density**: one idea per slide; visuals over text walls; agenda and next-steps slides stay scannable.
   - **Palette**: restrained base, one accent for metrics and the ask; screen-share contrast; consistent logo and comparison styling.

**Rules & constraints**

- **One-meeting gate**: deck serves exactly one objective and one primary next step.
- **Personalization gate**: prospect name, pains, and use cases appear before generic product overview.
- **Proof gate**: logos, results, and ROI use user-approved facts or explicit TBD—not implied wins.
- **One-idea gate**: if the headline needs “and,” split or cut.
- **Ask gate**: closing slide names owners, dates, and a clear question—not a passive “any questions?”

**Common failure modes**

- Vendor monologue → lead with their challenge; one credibility beat max for “about us.”
- Generic or cold deck → personalize title, agenda, and “what we heard.”
- Feature-tour demo → top 3 use cases; note live-demo vs screenshot.
- Price before value or weak close → pair ROI with investment; mutual action plan with explicit ask.
- Wrong audience depth → rebalance or flag appendix slides in the blueprint.

**Quality checklist**

- [ ] Meeting type, single objective, and target next step stated at top of blueprint.
- [ ] Prospect-specific challenge appears before solution/product slides.
- [ ] Every slide has one headline claim and a declared visual type.
- [ ] Proof slides cite source, approval, or TBD; no unlabeled metrics.
- [ ] Agenda and final slide align (discussion + next steps with owners/dates).
- [ ] Design intent names tone, hero emphasis, and accent usage for the full deck.

### Step 2. Use the content_and_design.md and k2f skill to generate a K2F file and export to an appropriate file format.

Follow **k2f** to instantiate slides from the blueprint, apply roles from design intent, verify, and export the meeting-ready deliverable.
