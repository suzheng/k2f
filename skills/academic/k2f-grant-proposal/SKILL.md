---
name: k2f-grant-proposal
description: Turn research ideas, preliminary data, and agency solicitations into reviewer-ready grant proposals with a fundable hypothesis, independent specific aims, and agency-aligned significance and innovation. Use when the user asks for a grant proposal, specific aims page, NIH R01/R21/K, NSF proposal, or foundation grant narrative. Create grant proposals with AI-native design and the highest aesthetic standards, powered by the K2F format and engine. Purpose-built for AI agents, K2F lets them focus on semantic content and design intent while automatically handling layout, typography, and formatting complexity.
---

# K2F Grant Proposal — AI-Native Aesthetic Design

## Purpose

Teach agents to blueprint fundable grant narratives before layout: a falsifiable hypothesis, explicit gaps, independent aims, reviewer-scannable hierarchy, and honest feasibility—without inventing data or drowning reviewers in prose.

## Built on K2F

This skill produces grant packages as `.K2F` documents. Follow the **k2f** skill for authoring: unpack or init a workspace, edit the semantic tree, run pack/verify, then export. K2F keeps agents on *meaning* (section roles, aims, evidence)—not pixel coordinates—while the engine compiles layout, typography, and theme into consistent output across PDF, DOCX, and other formats.

Before you build, install the core skill: `npx skills add suzheng/k2f --skill k2f`.

## When to Use

- Drafting or restructuring NIH, NSF, ERC, or foundation proposals from notes or prior work.
- Rebuilding a weak Specific Aims page before the full research strategy.
- Aligning content depth and visual hierarchy to mechanism limits and review rubrics.

## Workflow

### Step 1. Generate the content_and_design.md

Single blueprint for semantic content and aesthetic intent. Structure it with labeled blocks the k2f skill can map to pages later.

**Sub-steps**

1. **Mechanism lock** — Agency, solicitation ID, mechanism, page limits, required sections, scoring criteria (NIH: Significance, Innovation, Approach, Investigators, Environment; NSF: Intellectual Merit, Broader Impacts). Drop optional material that exceeds limits.

2. **Four-question audit** — Answer in prose using user-supplied evidence only:
   - Central hypothesis: testable, falsifiable (not “we will study X”).
   - Why now: gap, urgency, new tools or data.
   - Innovation: conceptual, methodological, application, or novel outcomes—each with specifics.
   - Feasibility: team, timeline, resources, aim independence, pitfalls with Plan B.

3. **Specific Aims architecture** — One page, ~400–550 words. Narrative arc: hook → gap → long-term goal → hypothesis → 2–3 verb-led aims (testable, largely independent) → impact close. Per aim: 2–3 sentences, expected outcome, no fatal dependency on prior aim success.

4. **Strategy outline** — Significance (problem, state of field, gap, cost of not knowing, if-successful impact). Innovation as scannable bullets (“This project is innovative because…”). Approach per aim: rationale → strategy/controls/stats → expected outcomes → pitfalls/alternatives → milestones. Assign **word budgets** from page limits (~450 words per full strategy page); note where figures replace paragraphs (preliminary data, workflow, timeline).

5. **Design intent** — Document tone, hierarchy, density, palette for the engine:
   - **Tone**: formal, evidence-led; a non-expert primary reviewer grasps significance on page 1.
   - **Hierarchy**: Aims page = strongest contrast and largest headings; strategy H1s conclusion-driven (“Gap in X limits Y”) not generic labels.
   - **Density**: bullet-led Innovation; Approach subheads per aim; no paragraphs over ~6 lines.
   - **Palette**: calm academic (navy/slate body, one accent for aim numbers and key metrics); WCAG AA; no decorative clutter on the Aims page.

**Rules & constraints**

- **Golden thread**: problem → hypothesis → aims → approach → impact; cut interesting tangents.
- **Aims independence**: partial success must remain fundable; merge aims that share one point of failure.
- **Honesty gate**: mark missing preliminary data; never fabricate citations, pilot results, or budget figures.
- **Agency fit**: mirror funder vocabulary and review dimensions in outline order.

**Common failure modes**

- Fishing expedition without falsifiable hypothesis → rewrite as if/then with measurable endpoints.
- Methods-before-problem Aims page → reorder hook → gap → hypothesis first.
- “Innovative approach” without novelty → one bullet per concrete first/novel/application.
- Scope creep → fewer aims or narrower mechanism; align word budgets.
- Aim 2 impossible if Aim 1 fails → redesign independence or alternatives.
- Dense Aims essay → enforce aim cards, bullets, and word cap in the design doc.

**Quality checklist**

- [ ] Hypothesis on Aims page matches Approach.
- [ ] Each aim has pitfalls + alternatives; no single point of failure across aims.
- [ ] Significance states gap and cost of not knowing.
- [ ] Innovation bullets name concrete novelty, not superlatives.
- [ ] Word budgets sum under mechanism limits.
- [ ] All evidence traceable to user inputs.

### Step 2. Use the content_and_design.md and k2f skill to generate a K2F file and export to an appropriate file format.

Follow **k2f** to instantiate pages from the blueprint, apply roles from design intent, verify, and export the agency-ready deliverable.
