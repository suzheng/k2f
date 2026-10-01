---
name: k2f-syllabus
description: Draft instructor-ready course syllabi with aligned outcomes, a scannable schedule spine, grading architecture, and policy layers from prompts, outlines, or legacy syllabi. Use when the user asks for a syllabus, course outline, curriculum map, or term schedule for a class they are teaching or redesigning. Create syllabi with AI-native design and the highest aesthetic standards, powered by the K2F format and engine. Purpose-built for AI agents, K2F lets them focus on semantic content and design intent while automatically handling layout, typography, and formatting complexity.
---

# K2F Syllabus — AI-Native Aesthetic Design

## Purpose

Teach agents to blueprint a **course syllabus** before layout: day-one essentials, scannable layering, and aligned outcomes, meetings, and grades—without policy dumps, orphaned readings, or schedules that hide assessment logic.

## Built on K2F

This skill produces course syllabi as `.K2F` documents. Follow the **k2f** skill for authoring: unpack or init a workspace, edit the semantic tree, run pack/verify, then export. K2F keeps agents on *meaning* (sections, schedule rows, policy blocks)—not pixel coordinates—while the engine compiles layout, typography, and theme.

Before you build, install the core skill: `npx skills add suzheng/k2f --skill k2f`.

## When to Use

- An instructor needs a new or revised syllabus (higher ed, professional continuing ed, or structured K–12 course overview).
- The user supplies partial inputs: catalog description, week topics, textbook, or an old syllabus to modernize.
- Not for supplementary reading-list generators, exam prediction from past tests, or lesson-level daily plans (use **k2f-lesson-plan**).

## Workflow

### Step 1. Generate the content_and_design.md

One blueprint for **what** belongs in the syllabus and **how** it should read when printed or shared as PDF. Structure it so the schedule spine and shared facts are named once and referenced everywhere the k2f skill will map to sections.

**Sub-steps**

1. **Route & clarify** — Infer discipline, term length, meeting pattern, and audience (intro undergrad through professional). Ask at most two high-leverage questions (e.g., mandated policy text vs. draft, credit hours); default term length when unspecified.

2. **Course identity** — One **identity block**: title, catalog code, instructor contact, term dates, room/platform, office hours, prerequisites—do not repeat elsewhere.

3. **Course story** — Brief description plus **3–6 learning outcomes** with verbs suited to audience. Infer missing outcomes from the description; mark `[inferred]`. Link each outcome to an assessment or major assignment.

4. **Schedule spine** — Week/unit rows: topic, in-class focus, readings, due dates. Compress long topic lists into **6–12 units** with stated grouping logic. Label topics with **applied domain** (course context, not generic keywords). Place high-stakes deadlines on the spine, not only in grading prose.

5. **Assessment & grading** — Categories, weights to 100%, short performance cues, outcome alignment. Late-work rules here only when they change grades.

6. **Policies & support** — Attendance, integrity, accommodations, tech, communication—brief and neutral unless the user supplies approved text (use verbatim). List materials, tools, safety/field notes, and help resources.

7. **Design intent** — For the engine, not coordinates:
   - **Tone**: credible welcome; grads assume fluency, intro defines terms once.
   - **Hierarchy**: title/term first; outcomes and schedule next; policies last.
   - **Density**: tight schedule rows; policies as bullets; no duplicate identity narrative.
   - **Consistency**: one date format and unit label scheme; parallel schedule columns.
   - **Palette**: print-safe; one accent for outcomes or schedule header; clear dates/weights.

**Rules & constraints**

- **Golden thread** — Outcomes → schedule → assignments → weights; cut units that do not serve outcomes unless labeled enrichment.
- **Honest inference** — Tag inferred fields; never fabricate accreditation details or mandatory policy quotes.
- **Arithmetic discipline** — Weights total 100%; workload fits credit level.
- **User-owned policy** — Mandated language verbatim; otherwise generic drafts flagged for registrar review.

**Common failure modes**

- Policies before purpose → identity, story, outcomes, then schedule.
- Too many vague outcomes → consolidate to assessable skills.
- Readings only in an appendix → tie to schedule rows.
- Grading labels with no spine assignment → rename or add rows.
- Repeated contact info → identity block only.
- Audience mismatch in jargon or load → recalibrate verbs and readings.

**Quality checklist**

- [ ] Identity block complete; contacts not duplicated.
- [ ] 3–6 outcomes linked to assessment/spine; `[inferred]` tagged.
- [ ] Full-term spine; major dates visible; 6–12 units if compressed.
- [ ] Weights total 100% and match spine assignments.
- [ ] Policies concise; materials and support listed.
- [ ] Design intent covers tone, hierarchy, density, consistency, palette.

### Step 2. Use the content_and_design.md and k2f skill to generate a K2F file and export to an appropriate file format.

Follow **k2f** to instantiate the syllabus from the blueprint, apply roles from design intent, verify, and export the shareable syllabus (typically PDF).
