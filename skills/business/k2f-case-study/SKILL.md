---
name: k2f-case-study
description: Create portfolio- or client-ready case studies from project notes, resume bullets, or outcome data—with a credible narrative arc, process evidence, quantified impact, and visual storytelling intent. Use when the user asks for a case study, portfolio piece, project writeup, work sample, or client success story. Create case studies with AI-native design and the highest aesthetic standards, powered by the K2F format and engine. Purpose-built for AI agents, K2F lets them focus on semantic content and design intent while automatically handling layout, typography, and formatting complexity.
---

# K2F Case Study — AI-Native Aesthetic Design

## Purpose

Teach agents to blueprint case studies before layout: show *how* and *why* work happened—not a resume repeat—with honest scope, evidence-backed results, and hierarchy that hooks skimmers and rewards deep readers.

## Built on K2F

This skill produces case studies as `.K2F` packages. Follow the **k2f** skill for authoring: unpack or init a workspace, edit the semantic tree, run pack/verify, then export. K2F keeps agents on *meaning* (section roles, narrative beats, visual placeholders)—not pixel coordinates—while the engine compiles layout, typography, and theme into consistent output across PDF, PPTX, DOCX, and other formats.

Before you build, install the core skill: `npx skills add suzheng/k2f --skill k2f`.

## When to Use

- Expanding bullets or slide decks into a portfolio or interview-ready project story.
- Documenting a client or internal win for sales, marketing, or leadership audiences.
- Tailoring one project for PM, design, engineering, or marketing emphasis without rewriting from scratch.

## Workflow

### Step 1. Generate the content_and_design.md

Single blueprint for semantic content and aesthetic intent. Use labeled blocks the **k2f** skill can map to sections and visuals later.

**Sub-steps**

1. **Story charter** — Primary reader (recruiter skim, hiring manager, client prospect), target read depth (3–5 min essential path vs 10–15 min optional depth), and the one outcome that earns attention. List inputs the user provided; mark gaps instead of inventing metrics, quotes, or artifacts.

2. **Role clarity** — Your title, team size, timeline, scope, and *your* contributions vs collective work. Write contribution claims in first person; attribute team-owned areas explicitly.

3. **Information architecture** — Default spine (shorten for skim-only audiences):
   - **Hero / quick facts**: project name, hook metric or outcome, role, timeline, team—scannable metadata strip.
   - **Overview**: 2–3 sentences; what was delivered and why it mattered.
   - **Problem / challenge**: business context, user or customer pain, goals, constraints—why this work was worth doing.
   - **Process**: research and discovery, options explored (breadth), decisions and rationale (depth), iteration from feedback or tests—highlight pivots and insights, not a diary of every meeting.
   - **Solution**: walkthrough of what shipped; key features or deliverables tied back to stated pains; note where visuals carry the story (before/after, flows, architecture diagram).
   - **Results / impact**: quantitative outcomes with timeframe and baseline; qualitative signals; business impact; optional “what we’d do differently.”
   - **Reflection** (optional for portfolio): learnings, skills grown, influence on later work.
   - **Visual asset list**: each planned image with caption intent, sensitivity redactions, and whether it is essential or nice-to-have.

4. **Audience emphasis** — Adjust depth without changing facts: PM/strategy (prioritization, trade-offs, metrics); design/UX (research, iterations, usability); engineering (architecture, constraints, performance); marketing (audience, creative, ROI). Cut sections that do not serve the charter reader.

5. **Design intent** — Document tone, hierarchy, density, and palette for the engine:
   - **Tone**: credible storyteller—specific, modest where evidence is thin; no hype or generic “passion” filler.
   - **Hierarchy**: lead with the strongest outcome in hero and overview; process subheads surface *decisions* and *insights*; results use numbers as visual anchors.
   - **Density**: short paragraphs and bullets; one idea per block; tables for metric before/after; push long methodology to optional subsections or appendix intent.
   - **Visual rhythm**: alternate narrative blocks with figure roles; favor journey (sketch → iteration → final) over a single polished screenshot; consistent figure sizing and explanatory captions in the blueprint.
   - **Palette**: clean portfolio or editorial (neutral body, one accent for metrics and callouts); WCAG AA; restrained use of pull quotes for user or stakeholder voice.

**Rules & constraints**

- **Show thinking, not just output**: every major solution block links to a stated problem or insight.
- **Evidence ladder**: separate **fact** (user data), **interpretation** (your reading), and **projection** (expected impact)—never upgrade guesses to results.
- **Process proportion**: enough to prove judgment; omit ceremonial steps with no decision or learning.
- **Contribution honesty**: no “we” vagueness for interview-facing work; no solo heroics for team-owned delivery.
- **Skim path**: a reader who stops after hero + problem + results still understands value.

**Common failure modes**

- Resume bullet paste without narrative arc → add problem stakes and decision moments.
- Solution gallery with no “so what” → tie each artifact to a pain point or metric.
- Process encyclopedia → collapse to research insight, option trade-off, chosen path.
- Missing or vague results → quantify where data exists; label `NEEDS USER INPUT` where it does not.
- Undifferentiated team story → sharpen “I led / I built / I validated” boundaries.
- Only final mocks → plan before/after or iteration figures in the asset list.
- Generic case study length → match charter depth; cut reflection if the reader is a prospect skimming one win.

**Quality checklist**

- [ ] Charter names audience, read depth, and hook outcome.
- [ ] Hero facts match overview and results; no contradictory metrics.
- [ ] Problem states goals and constraints; solution explicitly addresses them.
- [ ] Process includes at least one decision with rationale and one iteration or test signal.
- [ ] Results separate measured vs inferred; timeframe and baseline stated.
- [ ] Visual asset list covers essential story beats with caption intent.
- [ ] Design intent specifies hierarchy for hero, results, and figure rhythm.

### Step 2. Use the content_and_design.md and k2f skill to generate a K2F file and export to an appropriate file format.

Follow **k2f** to instantiate sections and figure roles from the blueprint, apply design intent, verify, and export the audience-ready case study.
