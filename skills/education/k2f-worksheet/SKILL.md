---
name: k2f-worksheet
description: Design printable, fill-in-ready worksheets—practice sets, graphic organizers, lab records, reflections, or gap-closure forms—from learning goals and source material. Use when the user asks for a worksheet, handout with answer spaces, practice problems, graphic organizer, or structured questions students complete in class or at home. Create worksheets with AI-native design and the highest aesthetic standards, powered by the K2F format and engine. Purpose-built for AI agents, K2F lets them focus on semantic content and design intent while automatically handling layout, typography, and formatting complexity.
---

# K2F Worksheet — AI-Native Aesthetic Design

## Purpose

Teach agents to blueprint a **worksheet** before layout: a single learner-facing task surface with scannable instructions, well-scoped items, and intentional response space—without essay dumps, duplicate prompts, or items that cannot be answered from what the sheet provides.

## Built on K2F

This skill produces worksheets as `.K2F` packages. Follow the **k2f** skill for authoring: unpack or init a workspace, edit the semantic tree, run pack/verify, then export. K2F keeps agents on *meaning* (sections, items, response types)—not pixel coordinates—while the engine compiles layout, typography, and theme.

Before you build, install the core skill: `npx skills add suzheng/k2f --skill k2f`.

## When to Use

- Educators need practice, rehearsal, or application aligned to a lesson, unit, or standard.
- The user wants structured blanks, organizers, short constructed responses, or checklists—not a full lesson plan (**k2f-lesson-plan**) or course overview (**k2f-syllabus**).
- Analyst-style gap worksheets: turn messy inputs into categorized, answerable question blocks (inspired by gap-analysis patterns)—still learner- or facilitator-completable on paper.

## Workflow

### Step 1. Generate the content_and_design.md

One blueprint for **what** appears on the sheet and **how** it should scan when printed. Register shared facts once; reference them from every item so the k2f skill can map sections without silent drift.

**Sub-steps**

1. **Route & clarify** — Infer subject, grade band, duration, and worksheet type (drill, scaffolded practice, lab/data, reflection, organizer, gap-closure). Ask at most two high-leverage questions (e.g., target standard, with/without answer key); default item count and difficulty from context.

2. **Learning frame** — Title, optional standard or skill gist (≤10 words), and **what success looks like** in one sentence. State prerequisites or materials the sheet assumes (textbook page, prior lesson, calculator)—not repeated per item.

3. **Information architecture** — Layer top to bottom:
   - **Orientation** (brief): purpose and how to use the sheet; no teaching lecture.
   - **Optional anchor** (one block max): stimulus—passage excerpt, diagram description, data table, scenario—only if items depend on it.
   - **Sections**: grouped by sub-skill or cognitive step; each section has a short heading and 3–12 items unless the user specifies otherwise.
   - **Response contract**: per item, declare response type (blank line, checkbox, table cell, short paragraph box, label-on-diagram) and approximate space needed.
   - **Answer key** (if requested): separate outline keyed by item ID; do not interleave solutions with student items on the same page role.

4. **Item design** — One measurable prompt per item; split compound questions. Order easy → hard within a section unless spiral review demands otherwise. For gap-style sheets, group by category; **omit empty categories**. Each gap row: short source snippet (or “not stated”), one specific question, blank answer field. Prioritize scope, core skill, and blocking ambiguities before nice-to-haves.

5. **Shared registry** — Numbers, names, vocabulary, dataset values, and scenario facts live once under `Shared content`; items reference stable keys so edits cannot fork.

6. **Design intent** — For the engine, not coordinates:
   - **Tone**: calm, classroom-credible; friendly for younger grades without clip-art clutter.
   - **Hierarchy**: title and learning frame strongest; section heads scannable; item numbers or letters consistent.
   - **Density**: instructions tight; generous workspace where handwriting or drawing is expected; avoid paragraph walls.
   - **Consistency**: parallel wording across items; same label scheme for sections and response types.
   - **Palette**: print-safe B&W; at most one accent for section headers or the learning frame.

**Rules & constraints**

- **Sheet must stand alone** — Every item answerable from orientation + anchor + shared registry; no “discuss with teacher” placeholders without a written product.
- **Golden thread** — Learning frame → sections → items → response spaces; cut items that do not practice the stated skill.
- **Original prompts** — Source material may inform scope; never copy publisher problem sets or copyrighted stimuli verbatim.
- **Honest scope** — If time or page budget is tight, reduce items or narrow the frame; do not shrink response space to illegibility.
- **Facilitator vs learner** — Teacher notes or scoring rubrics belong in the blueprint as separate artifacts, not mixed into student-facing item text.

**Common failure modes**

- Items test recall of unstated context → add anchor block or shared registry entry.
- One question bundles multiple skills → split items and reorder.
- Student sheet includes answer key or lengthy teaching → move to separate outline role.
- Every section repeats the same instructions → lift to orientation once.
- Gap categories with no real gaps → skip the section entirely.
- Vague prompts (“Explain more”) → rewrite to name evidence, length, or format expected.
- Mismatched difficulty in one section → rebalance or regroup.

**Quality checklist**

- [ ] Learning frame and every item align; cut orphans.
- [ ] Sections omitted when empty; item IDs stable for answer key.
- [ ] Shared registry lists all cross-item facts with keys.
- [ ] Each item has a declared response type and adequate space.
- [ ] Reading level fits grade; stimuli length justified.
- [ ] Design intent states tone, hierarchy, density, consistency, palette.
- [ ] No verbatim third-party item banks; gap questions specific and answerable.

### Step 2. Use the content_and_design.md and k2f skill to generate a K2F file and export to an appropriate file format.

Follow **k2f** to instantiate the worksheet from the blueprint, apply roles from design intent, verify, and export print-ready PDF or other deliverables.
