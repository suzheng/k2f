---
name: k2f-intake-form
description: Blueprint client, patient, or project intake forms with clear section flow, calibrated field density, and trustworthy fillable layout intent—not an unstructured questionnaire dump or generic web-form paste. Use when the user asks for an intake form, onboarding questionnaire, registration packet, new-client paperwork, or new-patient form. Create intake forms with AI-native design and the highest aesthetic standards, powered by the K2F format and engine. Purpose-built for AI agents, K2F lets them focus on semantic content and design intent while automatically handling layout, typography, and formatting complexity.
---

# K2F Intake Form — AI-Native Aesthetic Design

## Purpose

Teach agents to blueprint **intake forms** in `content_and_design.md`: what to collect before a relationship or case begins, how to sequence and group questions, and aesthetic intent (tone, hierarchy, fillable density)—without inventing clinical protocols, legal consent text, or eligibility rules the user did not supply.

## Built on K2F

This skill produces intake forms as `.K2F` packages. Follow the **k2f** skill for authoring: unpack or init a workspace, edit the semantic tree, pack/verify, and export. K2F keeps agents on *meaning*—sections, labels, field roles—not coordinates.

Before you build, install the core skill: `npx skills add suzheng/k2f --skill k2f`.

## When to Use

- **Printable or fillable intake** for clinics, agencies, legal, consulting, or enrollment—and redesigns of email questionnaires, legacy PDFs, or multi-step wizards as one coherent packet.

## Workflow

### Step 1. Generate the content_and_design.md

Single blueprint for form content and design intent. Use labeled blocks (header, instructions, sections, field rows, consent, review) the **k2f** skill can map later.

**Sub-steps**

1. **Intake frame** — Domain (healthcare, professional services, creative brief, internal project), organization, audience (new patient, client, vendor), submission path, **page budget** (often **1–2 pages**; add pages only when regulated sections or signature blocks require it), tone band (clinical-calm, warm service, formal compliance).

2. **Listen & requirement audit** — From brief or template: **must-collect** facts (identity, contact, reason for visit, dates, insurance IDs, emergency contact). Mark **attachment-backed** items once. Unknowns → `OPEN QUESTION` or `NEEDS USER INPUT`; never guess statutory or clinical wording.

3. **Information architecture (include / cut / merge)** — Default stack (reorder when the frame demands it):
   - **Header** → form title, org, optional logo note, version or effective date if supplied.
   - **Instructions** (2–4 lines): what to bring, estimated time, how data will be used, who to call with questions.
   - **Identity & contact** → legal name, DOB or age gate when relevant, phone, email, address; emergency contact when normative.
   - **Reason & context** → why they are here, referral source, urgency, service line or project type.
   - **Domain blocks** → only sections the user or template requires (e.g., insurance, medical history, scope, budget, stakeholders)—one theme per section.
   - **Preferences & logistics** → scheduling, communication, accessibility needs—compact rows, not essays.
   - **Consent & attestation** → signature, date, checkboxes **only with user-supplied or stubbed legal copy**.
   - **Review intent** → summary mirrors prior sections; identical labels.
   - **Cut** duplicate contacts, off-topic surveys, re-entered attachment data, optional hobbies.

4. **Section flow pass** — Easy facts first; sensitive history mid-document; consent last. Yes/no before “If yes, describe…”. Default one scrollable packet; name wizard steps in the blueprint only when the user requires them.

5. **Field typing pass** — Per prompt: one-line factual; short multiline (2–4 lines) for context; controlled yes/no/select rows; date fields with format note; signature block intent. State word or line limits in labels for open text. When in doubt, **shorter**—intake is discovery, not a memoir.

6. **Design intent** — **Tone**: respectful, low anxiety; legal/clinical copy only when supplied. **Hierarchy**: title and org first; strong section heads; clear labels. **Density**: section breaks; aligned label–field pairs; no underline grids. **Rhythm**: row facts plus small multiline bands; consent separated. **Brand**: restrained, one accent, readable contrast.

Log **assumptions** (order, omitted optionals) vs **open questions** (blockers for fields or consent).

**Rules & constraints**

- **Truth gate**: org names, clinical questions, consent clauses, and prefilled defaults trace to user input or explicit stubs.
- **Scope gate**: **in / out of scope** for this intake; post-submit flow is one instruction line, not a plan.
- **One-purpose gate**: each field asks one thing; split compound prompts.
- **Non-duplication gate**: data on attachments → pointer or “provided separately” checkbox—not full re-entry.
- **Consent gate**: no fabricated HIPAA, GDPR, or treatment consent; flag `NEEDS USER INPUT`.
- **Length gate**: overflow → drop optional sections or move to follow-up form; do not shrink type in the blueprint.

**Common failure modes**

- Planning essay or wizard bullet dump → labeled fields with types and answer-space intent.
- Sensitive questions before basic contact → reorder stack.
- Missing instructions or signature block → add brief how-to-submit and attestation intent.
- Generic questionnaire (“tell us about yourself”) → replace with domain-specific, concrete prompts.
- Review step that introduces new questions → review mirrors prior sections only.

**Quality checklist**

- [ ] Frame, page budget, tone, and requirement audit (must-collect vs attachments) documented.
- [ ] Section stack and in/out scope explicit; cuts explained.
- [ ] Every field has type, label text, and answer-space intent; conditional follow-ups noted.
- [ ] Consent and legal slots stubbed or sourced; assumptions vs open questions listed.
- [ ] Design intent covers tone, hierarchy, density, rhythm, and brand for the full form.

### Step 2. Use the content_and_design.md and k2f skill to generate a K2F file and export to an appropriate file format.

Follow **k2f** to instantiate sections and form fields from the blueprint, apply design intent, verify, and export the submission-ready intake (typically fillable PDF or DOCX).
