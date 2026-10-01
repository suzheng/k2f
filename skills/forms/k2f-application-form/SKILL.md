---
name: k2f-application-form
description: Blueprint job, program, or membership application forms with clear field architecture, calibrated answer spaces, and professional fillable layout intent—not a resume paste or generic questionnaire dump. Use when the user asks for an application form, printable application, enrollment form, or structured apply document (including role-specific or attachment-backed applications). Create application forms with AI-native design and the highest aesthetic standards, powered by the K2F format and engine. Purpose-built for AI agents, K2F lets them focus on semantic content and design intent while automatically handling layout, typography, and formatting complexity.
---

# K2F Application Form — AI-Native Aesthetic Design

## Purpose

Teach agents to blueprint **application forms** in `content_and_design.md`: which questions belong on the form versus attachments, how to group and sequence fields, and aesthetic intent (tone, hierarchy, fillable density)—without inventing employers, programs, legal clauses, or eligibility rules the user did not supply.

## Built on K2F

This skill produces application forms as `.K2F` packages. Follow the **k2f** skill for authoring: unpack or init a workspace, edit the semantic tree, pack/verify, and export. K2F keeps agents on *meaning* (sections, labels, field roles, instructions)—not coordinates—while the engine handles layout and typography.

Before you build, install the core skill: `npx skills add suzheng/k2f --skill k2f`.

## When to Use

- Designing or redesigning a **printable or fillable application** for a job, fellowship, grant, membership, volunteer role, or program intake.
- Consolidating scattered PDF questions into one scannable form.
- Prefilled answers in the blueprint when the user is applying; use **k2f-cover-letter** or **k2f-resume** for long prose attachments.

## Workflow

### Step 1. Generate the content_and_design.md

Single blueprint for form content and design intent. Use labeled blocks (header, instructions, sections, field rows, attestations) the **k2f** skill can map later.

**Sub-steps**

1. **Form frame** — Type (employment, academic, grant, membership), organization, role or program, submission channel, **page budget** (often **one page**; **two** only if several narratives are required), tone band (formal, warm nonprofit, startup-direct).

2. **Requirement audit** — From the user brief, posting, or template: list **must-collect data** (identity, contact, eligibility, dates, references, uploads). Mark **attachment-backed** items (resume, portfolio) so the form asks once—not twice. Flag `NEEDS USER INPUT` for missing legal text, equal-opportunity wording, or consent language; do not invent compliance copy.

3. **Information architecture (include / cut / merge)** — Default stack (reorder only when the frame demands it):
   - **Header** → title, org, role/program line, optional logo note.
   - **Instructions** (2–4 short lines): what to attach, deadlines, how answers will be used.
   - **Applicant identity** → name, contact, location or work authorization when relevant.
   - **Eligibility & factual** → degrees, years, licenses, yes/no gates in compact rows.
   - **Fit & experience** → targeted prompts, not a CV re-entry.
   - **Open narrative** → only prompts that need prose; state word limits in the label.
   - **References / links** → list pattern with URL lines.
   - **Attestation** → signature, date, optional checkbox confirmations.
   - **Cut** duplicate resume fields, trait adjectives (“passionate team player”), essay prompts that repeat the cover letter, and optional hobbies unless normative for the program.

4. **Field typing pass** — Assign **question type** and **answer-space intent** per prompt: factual one-liners; experience snapshots (2–4 sentences); why-here (~150 words); portfolio lines with URLs; behavioral STAR (~150–250 words); about-you (100–200 words); opinion (100–150 words). When in doubt, **shorter**; committees skim.

5. **Prefill layer (optional)** — If the user supplies a CV, JD, and answers: draft **field-level response text** that is direct and specific—each answer serves **one field purpose**, then stops. Mirror JD language only where it clarifies fit; never paste the posting back as the answer. Use `metric TBD` or `NEEDS USER INPUT` instead of inventing projects or metrics.

6. **Design intent** — **Tone**: calm, applicant-friendly; legalese only when supplied. **Hierarchy**: title and role/program first; strong labels; fields subordinate. **Density**: space between sections; tight label–field pairs; no blank-line walls. **Rhythm**: grid short rows; larger bands for narratives. **Brand**: restrained, one accent, readable contrast.

**Rules & constraints**

- **Truth gate**: organization names, eligibility rules, dates, and prefilled answers trace to user input or explicit stubs.
- **One-purpose gate**: each field asks one thing; split compound prompts.
- **Non-resume gate**: if data lives on an attached CV, collect a pointer or summary line—not full employment history again.
- **Length gate**: overflow → move long prose to attachments or cut prompts; do not shrink type in the blueprint.
- **Honesty gate**: prefilled years and ratings stay credible, not optimized.
- **Specificity gate**: correct role and org names; no placeholder employer in final text.

**Common failure modes**

- Cover-letter prose crammed into a small box → split prompts or redirect to an attachment.
- Generic trait prompts or JD repetition in labels → replace with concrete, role-specific questions.
- Essay sprawl → fewer narrative fields with stated word caps.
- Undifferentiated underline grid → section breaks and mixed row vs multiline fields.
- Missing instructions or attestation block → add brief how-to-submit and signature/date intent.
- Prefilled answers that list skills without context → years + one project anchor per claim.

**Quality checklist**

- [ ] Frame, page budget, tone, and requirement audit (must-collect vs attachments) documented.
- [ ] Section stack follows default architecture; cuts and merges explained.
- [ ] Every field has type, label text, and answer-space / word-limit intent.
- [ ] Optional prefill respects one-purpose and honesty gates; stubs flagged.
- [ ] Design intent covers tone, hierarchy, density, rhythm, and brand for the full form.

### Step 2. Use the content_and_design.md and k2f skill to generate a K2F file and export to an appropriate file format.

Follow **k2f** to instantiate sections and form fields from the blueprint, apply design intent, verify, and export the submission-ready form (typically fillable PDF or DOCX).
