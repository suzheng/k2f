---
name: k2f-lesson-plan
description: Build classroom-ready lesson plans with standards-aligned arcs, student-facing materials, and observation look-fors from teacher notes or topic prompts. Use when the user asks for a lesson plan, daily plan, mini-lesson, unit slice, or says they are teaching a specific grade, subject, or topic tomorrow. Create lesson plans with AI-native design and the highest aesthetic standards, powered by the K2F format and engine. Purpose-built for AI agents, K2F lets them focus on semantic content and design intent while automatically handling layout, typography, and formatting complexity.
---

# K2F Lesson Plan — AI-Native Aesthetic Design

## Purpose

Teach agents to blueprint a teachable lesson *before* layout: anchored standards, a timed instructional arc, concrete student tasks, and synchronized teacher/student/observation artifacts—without generic filler, curriculum plagiarism, or documents that drift apart.

## Built on K2F

This skill produces lesson packages as `.K2F` documents. Follow the **k2f** skill for authoring: unpack or init a workspace, edit the semantic tree, run pack/verify, then export. K2F keeps agents on *meaning* (phases, tasks, look-fors)—not pixel coordinates—while the engine compiles layout, typography, and theme.

Before you build, install the core skill: `npx skills add suzheng/k2f --skill k2f`.

## When to Use

- A teacher needs a new lesson (math, ELA, science, social studies, or adjacent K–12/higher-ed seminar formats).
- The user wants a plan plus handouts, source excerpts, or an observation grid in one coherent packet.
- Not for grading rubrics, quiz-only requests, or differentiating an *existing* plan without re-architecting content.

## Workflow

### Step 1. Generate the content_and_design.md

One blueprint for **what** gets taught and **how** it should read and scan. Structure it so shared elements are named once and referenced everywhere the k2f skill will later map to pages.

**Sub-steps**

1. **Route & clarify** — Infer subject, grade band, duration, and curriculum context from the prompt. Ask at most two high-leverage questions (e.g., standard to target, class constraints); default the rest. If subject pedagogy diverges (phenomenon-led science vs. inquiry social studies vs. problem-based math vs. text-centric ELA), note which arc applies.

2. **Standards anchor** — State the performance target in teacher-usable form: standard code (if known) plus a ≤10-word gist. List prerequisites and key vocabulary the lesson assumes so the teacher can spot a class mismatch early. Flag likely misconceptions to address in facilitation—not as a lecture block, but in phase notes.

3. **Lesson-at-a-glance** — Three sentences max: what students do, why it works for this class, and the measurable exit. Then a **timed phase list** (name, minutes, one line each) that sums to the block; every minute should tie to student action or formative check, not teacher monologue.

4. **Artifact map & shared registry** — Declare which documents belong in the packet (minimum: teacher lesson plan; often: student materials, observation template; sometimes: source packet or printable manipulatives). Register **once** in a `Shared content` section anything that must match across pages: phenomenon/context, numbers/names in problems, vocabulary list, exit ticket, discussion prompts, look-fors. Cross-reference by stable keys in each artifact outline so edits cannot fork silently.

5. **Per-artifact outlines** — **Lesson plan**: objective, materials, phase-by-phase facilitation (including questions and anticipated responses), differentiation hooks, closure. **Student materials**: only tasks students need on paper—workspace, stems, diagrams, sorting categories—at the lesson’s reading level. **Observation template**: narrow look-fors aligned to the objective (not a generic checklist). Oral-only lessons: state explicitly that no handout ships and what students use instead.

6. **Design intent** — Capture aesthetic direction for the engine (not layout coordinates):
   - **Tone**: calm, print-friendly, professional teacher tool; student pages friendlier but not childish unless grade demands it.
   - **Hierarchy**: lesson title and objective highest contrast; phase names scannable; facilitation detail nested under phases, not flattened walls of prose.
   - **Density**: plan readable in prep time—bullets and numbered steps over paragraphs; student sheets leave intentional white workspace.
   - **Consistency**: one vocabulary for phases and tasks across all artifacts; section titles vs. in-block labels must not repeat the same words twice.
   - **Palette**: minimal color, high B&W print fidelity; one accent at most for phase labels or objective callout.

**Rules & constraints**

- **Original instructional writing** — Curriculum or KG materials may inform scope and structure; never reproduce publisher student text, teacher scripts, or problem sets verbatim.
- **Golden thread** — Standard → objective → phases → student work → exit ticket → look-fors; cut interesting tangents that do not advance the objective.
- **Time budget honesty** — If activities cannot fit, narrow the objective or drop a phase; do not shrink student thinking time to zero.
- **Audience split** — Teacher plan holds rationale and pivots; student materials hold only what learners see; observation holds evidence columns, not lesson script.

**Common failure modes**

- Activities without a measurable exit → add or rewrite the exit ticket first, then align phases.
- Phase list is all “explain” → insert pair-share, practice, or investigation with named student products.
- Student handout duplicates the full plan → strip to tasks and graphics only.
- Shared context changes in one artifact only → run a consistency sweep across every outline block.
- Heading and block label repeat → move the task name to one location only.
- Prerequisites omitted → teacher discovers mismatch on lesson day; surface assumed skills in the glance section.

**Quality checklist**

- [ ] Standard anchor and objective match; exit ticket assesses the same skill.
- [ ] Phases sum to allotted time; each phase has a student-visible action.
- [ ] Shared registry lists every cross-document element with stable keys.
- [ ] Student reading level fits grade; observation look-fors are observable in one period.
- [ ] Design intent states tone, hierarchy, density, and print-safe palette.
- [ ] No verbatim third-party curriculum text; sources paraphrased or excerpted with purpose noted.

### Step 2. Use the content_and_design.md and k2f skill to generate a K2F file and export to an appropriate file format.

Follow **k2f** to instantiate the packet from the blueprint, apply roles from design intent, verify, and export the classroom-ready deliverables.
