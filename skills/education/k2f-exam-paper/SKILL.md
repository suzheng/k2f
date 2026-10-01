---
name: k2f-exam-paper
description: Design formal exam papers—midterms, finals, unit tests, or take-home assessments—from a syllabus, rubric, past papers, or source readings, with weighted coverage, unambiguous calls for response, and intentional answer space. Use when the user asks for an exam paper, test, quiz, final, midterm, or printable assessment to distribute to students. Create exam papers with AI-native design and the highest aesthetic standards, powered by the K2F format and engine. Purpose-built for AI agents, K2F lets them focus on semantic content and design intent while automatically handling layout, typography, and formatting complexity.
---

# K2F Exam Paper — AI-Native Aesthetic Design

## Purpose

Teach agents to blueprint an **exam paper** before layout: syllabus-faithful scope, stable format fingerprints, fair point weighting, and prompts students can parse under clock pressure—not study sheets, predicted questions, or item banks pasted without editorial structure.

## Built on K2F

This skill produces exam papers as `.K2F` packages. Follow the **k2f** skill for authoring: unpack or init a workspace, edit the semantic tree, run pack/verify, then export. K2F keeps agents on *meaning* (sections, items, stimuli, response roles)—not pixel coordinates—while the engine compiles layout, typography, and theme.

Before you build, install the core skill: `npx skills add suzheng/k2f --skill k2f`.

## When to Use

- An instructor or instructional designer needs a **deliverable exam document** (in-class, take-home, or hybrid).
- Inputs include syllabus topics, mark weightings, learning outcomes, approved readings, or past exams to match style—not when the user only wants study notes or “what might be on the exam.”
- Distinct from **k2f-worksheet** (low-stakes practice) and **k2f-lesson-plan** (teaching sequence).

## Workflow

### Step 1. Generate the content_and_design.md

One blueprint pairing **assessment content** with **aesthetic intent**. Register shared facts once; every question references stable keys so stimuli and numbering cannot drift.

**Sub-steps**

1. **Exam lock** — Course, assessment title, sitting length, total points, open/closed/take-home rules, and allowed materials. If past papers exist, note format fingerprints (question count, style mix, fact-pattern density, call style) to **match or deliberately change**—state which.

2. **Coverage map** — Table syllabus topics → target points (%). Prefer explicit syllabus weightings; otherwise infer from contact hours and learning outcomes. Flag topics taught but **under-weighted on purpose** vs accidentally omitted.

3. **Information architecture** — Layer top to bottom: **identification**; **instructions once** (time, materials, integrity, labeling); optional **sections** when styles differ; **questions** with stable IDs and points that sum to the exam lock; **stimuli** in shared content when multiple sub-parts share facts; **response contract** per sub-part (type + space budget); **scoring outline** in a separate role (model answers, keywords, partial credit)—not on the student layer unless requested.

4. **Item design** — One assessable outcome per sub-part; split compound calls. Match **question style** to the map: recall, application, issue-spotting, policy, quantitative, MCQ. Order within a section: warm-up → anchor items → discriminating items unless spiral review is intended. When source readings bound scope, every item must be answerable from syllabus + named materials; mark `[OUT OF SCOPE]` gaps instead of inventing content.

5. **Pattern fidelity** — When matching past papers, note stable vs variable elements (topic emphasis, style mix, fact density). Preserve **structure**, never verbatim prior text.

6. **Design intent** — **Tone**: formal, neutral; no coaching fluff. **Hierarchy**: question numbers and points scan first; stimuli grouped under shared headings. **Density**: tight stems, generous handwriting space. **Consistency**: parallel verbs and labeling. **Palette**: print-safe B&W; one accent max for section warnings.

**Rules & constraints**

- **Syllabus is the ceiling** — Do not test untaught topics unless the user explicitly expands scope.
- **Points must sum** — Section totals, question totals, and exam lock agree; show the arithmetic in the blueprint.
- **Single instructions block** — Per-question repeats of “show your work” belong in the block once unless a section truly differs.
- **Honest difficulty** — If time or page budget is tight, drop items or narrow coverage; do not shrink answer space below what the call requires.
- **Original items** — Source material informs scope; never lift publisher or prior-year text verbatim without rights.
- **Separate roles** — Student paper vs scoring outline vs optional cover sheet are distinct artifacts in the design doc.

**Common failure modes**

- Unstated facts required → extend shared registry or cut the item.
- Compound questions → split sub-parts and re-weight points.
- Rubric on student layer → move to scoring outline.
- Weighting drift vs coverage map → rebalance or revise the map.
- Vague “Discuss” calls → add length, format, and evidence expectations.
- Format mismatch (take-home vs timed) → fix exam lock and instructions.

**Quality checklist**

- [ ] Exam lock complete; points sum to total; materials policy clear.
- [ ] Coverage map aligned with every question; orphan topics cut or justified.
- [ ] Shared registry lists all cross-item facts, data, and stimuli with keys.
- [ ] Each sub-part has points, response type, and adequate space.
- [ ] Instructions appear once; numbering stable for scoring outline cross-reference.
- [ ] Scoring outline keyed by question ID; keywords/partial credit where constructed responses apply.
- [ ] Design intent covers tone, hierarchy, density, consistency, palette.
- [ ] Original items only; out-of-scope gaps flagged.

### Step 2. Use the content_and_design.md and k2f skill to generate a K2F file and export to an appropriate file format.

Follow **k2f** to instantiate the exam from the blueprint, apply student vs instructor roles from the design doc, verify, and export print-ready PDF or other deliverables.
