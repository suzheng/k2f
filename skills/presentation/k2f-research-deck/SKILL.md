---
name: k2f-research-deck
description: Turn papers, study notes, and preliminary data into decision-grade research decks for lab meetings, PI review, grants, and seminars—with faithful numbers, evidence chains, and caveats. Use when the user asks for a research deck, lab meeting slides, study summary deck, academic review deck, or paper-to-slides for a faithful talk (not stylized social image decks). Create research decks with AI-native design and the highest aesthetic standards, powered by the K2F format and engine. Purpose-built for AI agents, K2F lets them focus on semantic content and design intent while automatically handling layout, typography, and formatting complexity.
---

# K2F Research Deck — AI-Native Aesthetic Design

## Purpose

Teach agents to blueprint **research decks** before layout: one scientific claim per slide, traceable evidence, explicit caveats before the close, and no numbers or figures redrawn from memory.

## Built on K2F

This skill produces research presentations as `.K2F` packages. Follow the **k2f** skill for authoring: unpack or init a workspace, edit the semantic tree, pack/verify, and export. K2F keeps agents on *meaning* (slide roles, evidence, visual intent)—not coordinates—while the engine compiles layout, typography, and theme into consistent, projector-ready output.

Before you build, install the core skill: `npx skills add suzheng/k2f --skill k2f`.

## When to Use

- Preparing a lab meeting or PI update from a paper, preprint, or experiment log.
- Building a grant or internal review deck that must prove novelty without overclaiming.
- Converting dense manuscripts into a 6–15 slide talk with faithful metrics and figures.
- Rebuilding generic “paper summary” AI slides into a skimmable evidence story.

## Workflow

### Step 1. Generate the content_and_design.md

Single blueprint for semantic content and aesthetic intent. Use labeled blocks per slide (role, headline, body, visual type, asset notes) that the **k2f** skill can map later.

**Sub-steps**

1. **Brief frame** — Audience, **decision target** (inform, approve next step, defend novelty), time budget, language. **Live talk** = sparse bullets; **async deck** = self-explanatory headlines per slide.

2. **Source-of-truth audit** — List abstract, methods sketch, figures/tables, exact results, citations, limitations. Tag `NEEDS USER INPUT` for gaps; never invent stats, n, or citations.

3. **Evidence spine** — Arc before titles: stakes → gap → question/hypothesis → design → results (strongest first) → interpretation → caveats → next step/ask. External reviewers: contributions early; lab updates: lead with freshest result.

4. **Slide architecture (include / cut)** — Target **6–15 slides**; merge or omit when inputs are thin:
   - Title (paper metadata) → Context/problem → Gap/question → Approach overview → Methods (diagram-led) → Main results (split quant vs qual if both exist) → Mechanism/interpretation → Limitations → Next steps → (optional) key references.
   - **Cut** literature surveys, duplicated abstract+introduction, method prose that belongs in backup, and extra ablation slides that do not change the decision.
   - **Split** when two claims share one slide (problem+hypothesis, multiple unrelated figures, or results+interpretation).

5. **Per-slide content contract** — For each slide specify:
   - **Headline**: conclusion, not section label (“Knockdown reduces latency 42% at n=12” not “Results”).
   - **Body**: max 3–5 bullets or one visual block; ~2 lines per bullet; no orphan single-word lines.
   - **Visual type**: methods pipeline, results chart, comparison matrix, qualitative grid, equation focus, contribution list, agenda/outline—prefer **chart/table/figure intent** over paragraphs for data.
   - **Asset fidelity**: **source figure/table** (figure #, caption) vs **schematic only**; do not bulletize tables that should display as tables.

6. **Numbers** — One unit system; matching metrics across slides; n/CI/test on hero numbers; `[VERIFY]` if uncertain.

7. **Design intent** — **Tone**: restrained, evidence-led. **Hierarchy**: title largest; one hero figure/metric/equation per slide. **Density**: one message per slide; section breaks between movements. **Palette**: calm academic, one accent, projection-safe contrast.

**Rules & constraints**

- **One-claim gate**: if the headline needs “and,” split the slide or demote a thread to backup.
- **Fidelity gate**: quantitative slides use user-supplied numbers or explicit TBD—not rounded or “typical” values.
- **Figure gate**: main plots/tables use source assets or explicit chart intent—not prose-redrawn data.
- **Caveat gate**: limitations or confounds before the closing ask.
- **Novelty gate**: one scannable “what’s new”; cut known background.

**Common failure modes**

- Ten slides of background, one of results → reorder spine; promote results and design earlier.
- Methods as paragraph walls → one pipeline diagram + 3 bullets max.
- Garbled or invented statistics → revert to source table/chart intent or mark TBD.
- Missing limitations → add explicit caveats slide; avoids PI “what about confound X?” derail.
- Duplicate paper abstract on multiple slides → single context slide, then gap.
- Crowded multi-panel figures → one panel per slide or callouts in the blueprint.
- Overclaiming headlines → match data; speculation under “open questions.”

**Quality checklist**

- [ ] Decision target and audience stated at top of the blueprint.
- [ ] Every slide has one headline claim and a declared visual type.
- [ ] Metrics and assets trace to inputs, TBD, or `[VERIFY]`.
- [ ] Caveats and next steps before the final slide.
- [ ] 6–15 slides (or justified); design intent covers tone and hierarchy for the deck.

### Step 2. Use the content_and_design.md and k2f skill to generate a K2F file and export to an appropriate file format.

Follow **k2f** to instantiate slides from the blueprint, apply roles from design intent, verify, and export the presentation-ready deliverable.
