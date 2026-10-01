---
name: k2f-fillable-pdf
description: Blueprint professional fillable PDF forms—applications, registrations, intake packets, and compliance worksheets—with clear field inventory, section flow, and print-friendly hierarchy before layout. Use when the user asks for a fillable PDF, PDF form, AcroForm-style document, or interactive application packet. Create fillable pdfs with AI-native design and the highest aesthetic standards, powered by the K2F format and engine. Purpose-built for AI agents, K2F lets them focus on semantic content and design intent while automatically handling layout, typography, and formatting complexity.
---

# K2F Fillable Pdf — AI-Native Aesthetic Design

## Purpose

Teach agents to author `content_and_design.md` for forms people actually complete: explicit static copy, a complete field inventory with types and constraints, logical section flow, and aesthetic intent tuned for scanning and typing—not a generic questionnaire dump or a static PDF with boxes pasted on.

## Built on K2F

This skill produces fillable documents as `.K2F` packages. Follow the **k2f** skill for authoring: unpack or init a workspace, edit the semantic tree, run pack/verify, then export. K2F keeps agents on *meaning* (labels, field roles, instructions, grouping)—not pixel coordinates—while the engine compiles layout, typography, and interactive fields into consistent output.

Before you build, install the core skill: `npx skills add suzheng/k2f --skill k2f`.

## When to Use

- New application, registration, intake, onboarding, or compliance forms from requirements or an existing paper PDF.
- Restructuring a cluttered legacy form before rebuild.
- Splitting one long form into scannable sections while preserving exact legal or policy wording supplied by the user.

## Workflow

### Step 1. Generate the content_and_design.md

One blueprint merging **what readers see** (static) and **what they enter** (interactive), plus **design intent** the engine can interpret. Use labeled blocks; map pages later in k2f.

**Sub-steps**

1. **Completion scenario** — Who fills it (applicant, staff, patient), alone or with attachments, estimated time, submission path, and required vs optional sections. Drop fields that do not change an outcome.

2. **Static layer** — Title, version or effective date, issuer identity, short “how to complete” steps, privacy or retention notice **only from user text**, section introductions, and footers (page x of y, contact). Never invent legal clauses.

3. **Field inventory** — Table or structured list: stable id, human label, control kind (single-line text, multiline, checkbox, radio group, dropdown, date, signature attestation), required flag, placeholder or format hint, options for choice fields, max length or line count, help text one line max. Group radios and checkbox sets under one question with mutually exclusive or multi-select semantics spelled out.

4. **Information architecture** — Order for cognitive flow: identify → details → choices → authorization → review. Cluster 5–9 fields per section; start a new section when the topic shifts. Pair short labels with inputs in two-column rows; use full width for paragraphs, multiline answers, and declaration blocks. Specify conditional blocks (“show employer block only if Employment = Yes”) as explicit rules, not implied layout.

5. **Design intent** — Tone (institutional, clinical calm, friendly intake), hierarchy (title largest; section H2; labels medium weight; help text smaller/lighter), density (generous vertical rhythm; avoid wall-of-boxes), alignment (labels flush right or top-aligned in pairs; choice groups indented under question), accent (one color for section bars or required markers), contrast (WCAG AA on labels and instructions). State where checkboxes need a clear hit target vs inline legal text.

**Rules & constraints**

- **Label clarity**: every field answers “of what?” (“Work email”, not “Email”).
- **One question, one control pattern**: exclusives = radio; multi = checkboxes; don’t mix.
- **Required honesty**: mark optional fields; don’t default everything required.
- **Verbatim gate**: policy, consent, and certification language copied from user sources only.
- **Completion path**: last section includes certification name/date/signature fields when the use case needs it.

**Common failure modes**

- Placeholder replaces the label → add visible labels; placeholders are hints only.
- Fishing-expedition forms → cut fields without a stated reviewer or system use.
- Random field order → reorder to scenario flow; group related rows.
- Duplicate asks (city in two sections) → merge or cross-reference once.
- Unspecified radio/checkbox options → list every option label explicitly.
- Dense single-column marathon → split sections; use label+field pairs for short items.
- Missing conditional logic → document show/hide rules beside affected fields.
- Generic “form template” voice → set tone and issuer context in design intent.

**Quality checklist**

- [ ] Every interactive control appears in the field inventory with type and required flag.
- [ ] Section order matches completion scenario; tab flow is top-to-bottom, left-to-right within pairs.
- [ ] Static instructions precede the fields they govern.
- [ ] Choice fields list complete, non-overlapping options.
- [ ] Legal or policy text is user-supplied and unchanged.
- [ ] Design intent covers tone, hierarchy, density, and accent without layout coordinates.
- [ ] No orphan labels or fields lacking purpose in the scenario.

### Step 2. Use the content_and_design.md and k2f skill to generate a K2F file and export to an appropriate file format.

Follow **k2f** to instantiate pages from the blueprint, bind field roles from the inventory, verify, and export the fillable PDF deliverable.
