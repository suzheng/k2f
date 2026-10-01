---
name: k2f-survey-form
description: Blueprint feedback, research, and pulse survey forms with bounded scope, consistent scales, and credible fillable layout intent—not an unfocused question dump or generic Google Form paste. Use when the user asks for a survey form, questionnaire, feedback form, customer or employee survey, evaluation form, or structured opinion instrument. Create survey forms with AI-native design and the highest aesthetic standards, powered by the K2F format and engine. Purpose-built for AI agents, K2F lets them focus on semantic content and design intent while automatically handling layout, typography, and formatting complexity.
---

# K2F Survey Form — AI-Native Aesthetic Design

## Purpose

Teach agents to blueprint **survey forms** in `content_and_design.md`: what to measure, how to scope and sequence questions, and aesthetic intent (neutral credibility, scale rhythm, hierarchy)—without inventing study hypotheses, sampling plans, or consent language the user did not supply.

## Built on K2F

This skill produces survey forms as `.K2F` packages. Follow the **k2f** skill for authoring: unpack or init a workspace, edit the semantic tree, pack/verify, and export. K2F keeps agents on *meaning*—sections, prompts, response types—not coordinates.

Before you build, install the core skill: `npx skills add suzheng/k2f --skill k2f`.

## When to Use

- **Printable or fillable surveys** for customer feedback, NPS-style pulses, event or course evaluation, employee engagement, UX research, or academic data collection.
- Redesigning long email questionnaires or legacy PDFs into one coherent, completion-friendly instrument.

## Workflow

### Step 1. Generate the content_and_design.md

Single blueprint for survey content and design intent. Use stable labeled blocks (header, intro, sections, questions, scales, thank-you) the **k2f** skill can map later.

**Sub-steps**

1. **Survey frame (triage)** — Freeze **what** is measured, **who** responds, **why** now, and how results will be used. Set **mode** (customer, product/UX, employee pulse, event eval, academic). Note anonymity, channel, **time budget**, and tone (neutral research, warm brand, formal).

2. **Measurement contract** — List **must-learn constructs**; mark **out of scope** topics and deferred waves. Unknown targets or legal copy → `OPEN QUESTION` / `NEEDS USER INPUT`; do not fabricate IRB, GDPR, or policy text.

3. **Information architecture (include / cut / merge)** — Default stack:
   - **Header** → survey title, sponsor/org, optional version or wave label.
   - **Introduction** (3–5 lines): purpose, estimated time, confidentiality/anonymity as supplied, how to skip optional blocks.
   - **Screener** (only if needed) → eligibility or experience gates; short yes/no rows.
   - **Core blocks** → one **theme per section**; state the section’s measurement goal in one line before questions (claim-then-items, not mystery headings).
   - **Open feedback** → at most one or two capped prompts; place after scaled blocks unless the user requires early qualitative probes.
   - **Demographics** → usually **last** (unless screener requires them); collect only fields tied to stated analysis—default cut is “nice to know.”
   - **Thank-you / close** → appreciation and optional contact line if identified follow-up is allowed.
   **Cut** duplicate constructs, double-barreled prompts, leading or loaded wording, essay sprawl, and every question that does not serve the frozen **what/why**.

4. **Question design pass** — One construct per item; split double-barreled prompts. Keep response types consistent within a block; define **shared scale anchors** once and reuse—do not mix agreement and frequency in one matrix. Note **branching intent** in prose; state limits on open text.

5. **Burden & order pass** — Easy, non-sensitive items first; sensitive or identifying items late. Group **matrix-style** items by shared stem where it aids scanning. If the frame risks fatigue, drop optional sections before shortening scales in the blueprint.

6. **Design intent** — **Tone**: neutral stems; warmth only in intro/thank-you. **Hierarchy**: title, sponsor, section goals, then items. **Density**: section breaks; aligned stems/options; no underline-only grids. **Rhythm**: compact scale rows; separate open-comment bands. **Brand**: restrained, one accent, readable contrast.

Log **assumptions** (scale choice, demographics omitted) vs **open questions** (consent, sampling, analysis plan).

**Rules & constraints**

- **Scope gate**: one primary survey purpose; secondary topics become `FUTURE WAVE` or cut.
- **Alignment gate**: every heading matches only the questions beneath it; move strays or rename the section.
- **Construct gate**: each question maps to one listed construct; no orphan “just curious” items.
- **Scale gate**: one anchor set per block; symmetric labels unless user specifies otherwise.
- **Truth gate**: org names, confidentiality claims, and prefilled defaults trace to user input or stubs.
- **Burden gate**: exceeding time budget → cut optional blocks, not micro-type in the blueprint.

**Common failure modes**

- Unbounded laundry list → triage + construct list + explicit cuts.
- Demographics before trust is established → move to end unless screener requires.
- Mixed scales in one matrix → split blocks or unify anchors.
- Section titles that do not match contents → alignment pass.
- Cover-letter tone in Likert labels → neutral stems and balanced anchors.
- Open-ended overload → cap count and place after scaled core.

**Quality checklist**

- [ ] Frame documents what/who/why, mode, time budget, anonymity, and measurement contract.
- [ ] Section stack stable; each core block opens with a one-line measurement goal.
- [ ] Every item has type, label text, scale or option intent, and construct mapping.
- [ ] Branching, optional blocks, and cuts documented; legal/confidential slots sourced or stubbed.
- [ ] Design intent covers tone, hierarchy, density, rhythm, and brand for the full form.

### Step 2. Use the content_and_design.md and k2f skill to generate a K2F file and export to an appropriate file format.

Follow **k2f** to instantiate sections and response fields from the blueprint, apply design intent, verify, and export the completion-ready survey (typically fillable PDF or DOCX).
