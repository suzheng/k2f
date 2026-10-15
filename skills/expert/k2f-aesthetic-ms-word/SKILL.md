---
name: k2f-aesthetic-ms-word
description: Create polished, general-purpose Microsoft Word documents with clear structure, typed content blocks, and strong visual hierarchy for any brief. Use when the user asks for Microsoft Word, MS Word, a Word document, DOCX, .docx, a beautiful or professional Word doc, an editable Word handoff, or to beautify Word formatting. Create Microsoft Word documents with AI-native design and the highest aesthetic standards, powered by the K2F format and engine. Purpose-built for AI agents, K2F lets them focus on semantic content and design intent while automatically handling layout, typography, and formatting complexity.
---

# K2F Aesthetic MS Word — AI-Native Aesthetic Design

## Purpose

Teach agents to create highly aesthetic, general-purpose Word documents—any topic, any audience—that open and stay editable in Microsoft Word. Work through explicit copy, section-level reading beats, typographic roles, and art direction in `content_and_design.md`, then build with **k2f**—not slide-deck bullets, manual bold-and-spacing hacks, tab-aligned pseudo tables, or “design as one screenshot.”

## Built on K2F

This skill produces Word-ready packages as `.K2F` documents exportable to DOCX. Follow the **k2f** skill for authoring: unpack or init a workspace, edit the semantic tree, run pack/verify, then export. K2F keeps agents on *meaning* (sections, figures, hierarchy roles)—not frame coordinates—while the engine compiles layout and typography into an editable Word handoff.

Before you build, install the core skill: `npx skills add suzheng/k2f --skill k2f`.

## When to Use

- The user wants a beautiful, professional Word document and did not name a narrower K2F task skill—or the brief is general.
- Primary deliverable is polished, editable DOCX for Microsoft Word or SharePoint, regardless of domain.
- Restructuring messy drafts or Markdown into a scannable Word hierarchy, then exporting through **k2f**.
- Optional: combine with another K2F task skill—that skill owns domain facts and compliance; this skill owns Word reading mode, block roles, and aesthetic design.

## Workflow

### Step 1. Generate the content_and_design.md

One `content_and_design.md` merging **reader path**, **section roles**, **typed content blocks**, and **design intent** the engine can interpret. Use labeled sections; map pages later in k2f.

**Sub-steps**

1. **Document scenario** — Audience, purpose (inform, decide, approve, archive), reading mode (skim vs study), approximate length, need for cover, TOC, signature block, and what must stay verbatim (policy, legal, pricing, quotes). Drop sections that do not serve the scenario.

2. **Document skeleton** — Ordered parts: cover or title block, front matter (TOC, distribution, summary), body chapters or sections, appendix or references, closing (sign-off, contact). Tag sections as **opening**, **narrative**, **reference**, or **closing** so density is not uniform start to finish.

3. **Content inventory** — Per section, list blocks with stable ids and roles: title, subtitle/deck, body (approximate word budget), pull quote (+ attribution), callout or sidebar, figure (+ caption + credit), table (title + column semantics), list (ordered vs unordered intent), CTA or signature block. Mark **must-run** user copy vs **agent-drafted** placeholders the user must replace.

4. **Information architecture** — One dominant idea per major section; support with secondary blocks, not competing headlines. Linear flow: hook → context → detail → decision or action. Keep callouts beside the paragraph they annotate; park wide reference tables in appendix intent. Prefer real table blocks over tab-aligned columns.

5. **Design intent** — Tone (executive crisp, academic sober, friendly internal), typographic hierarchy (title vs deck vs body vs caption—relative, not points), density (airy openers vs tight reference sections), list rhythm (short bullets vs numbered procedures), accent (neutrals + one color for rules, labels, CTAs), contrast target (readable on screen and when printed).

**Rules & constraints**

- **Role discipline**: each block has one role; do not fake hierarchy with bold alone.
- **Word budgets**: cap body per section; move overflow forward or to appendix—never shrink type in the authoring plan.
- **Verbatim gate**: policy, legal, pricing, and certification language only from user sources.
- **Word handoff**: prefer editable text and discrete figures over describing “designed as one image.”
- **No coordinates**: state section boundaries and roles, not x/y or point sizes.
- **Not forms or print spreads**: interactive fields → **k2f-fillable-pdf**; magazine spreads and IDML → **k2f-idml**.

**Common failure modes**

- Slide-deck pacing (title + bullets every section) → assign narrative roles and varied section types.
- Arial soup (random bold and font names) → map emphasis to roles in design intent.
- Tab pseudo tables → table inventory with column semantics.
- Captions separated from figures → lock figure+caption pairs in the inventory.
- Buried executive summary → place skimmer entry in opening section roles.
- Uniform wall of body → alternate narrative, callout, and reference sections.
- Invented compliance or pricing → flag gaps; never fabricate.
- Flattened page screenshots → discrete text and image blocks for Word editing.

**Quality checklist**

- [ ] Skeleton lists every major part; each body section has a section role assigned.
- [ ] Every figure/table has caption, credit when needed, and paired placement intent.
- [ ] Headlines are unique and ranked (one primary per major section).
- [ ] Word budgets fit the stated length without cramming.
- [ ] Verbatim copy is quoted or flagged; no silent invention of legal or pricing.
- [ ] Design intent covers tone, type roles, density, lists/tables, and accent without coordinates.
- [ ] Closing section delivers the scenario CTA (sign-off, next step, contact).

### Step 2. Use the content_and_design.md and k2f skill to generate a K2F file and export to an appropriate file format.

Follow **k2f** skill to instantiate sections from `content_and_design.md`, bind content roles from the inventory, verify, and export the DOCX deliverable for Microsoft Word.
