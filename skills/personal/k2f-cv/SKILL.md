---
name: k2f-cv
description: Compile a complete academic or senior professional curriculum vitae from notes, publication lists, and appointment history—with correct section order, honest bibliographic detail, and readable long-list hierarchy (not a one-page job resume). Use when the user asks for a CV, curriculum vitae, academic vita, faculty application CV, or multi-page career record. Create cvs with AI-native design and the highest aesthetic standards, powered by the K2F format and engine. Purpose-built for AI agents, K2F lets them focus on semantic content and design intent while automatically handling layout, typography, and formatting complexity.
---

# K2F Cv — AI-Native Aesthetic Design

## Purpose

Teach agents to blueprint **CVs** in `content_and_design.md`: scholarly sections, tiered long lists, citation-ready entries, and design intent—without inventing degrees, papers, grants, or metrics.

## Built on K2F

This skill produces CVs as `.K2F` packages. Follow the **k2f** skill for authoring, pack/verify, and export. K2F keeps agents on sections and list roles while the engine handles multi-page flow and typography.

Before you build, install the core skill: `npx skills add suzheng/k2f --skill k2f`.

## When to Use

- Building or updating an **academic or research CV** (faculty, postdoc, PhD, clinician-scientist, senior technical fellow).
- Merging exports (Scholar, BibTeX, old Word) into one structure.
- **Selected vs full** lists for a search—not one-page resumes (**k2f-resume**).

## Workflow

### Step 1. Generate the content_and_design.md

Single blueprint for semantic content and design intent. Use labeled blocks per section (identity, appointments, education, publications, etc.) that the **k2f** skill can map later.

**Sub-steps**

1. **CV frame** — Discipline, career stage, opportunity type (tenure-track, fellowship, industry lab lead), geography, and **page budget** (often 2–6+ pages; justify length). Note required section order if the user cites a department or funder template.

2. **Input audit** — Inventory appointments, degrees, publications, preprints, grants, teaching, service, talks, patents, supervision, awards, and links. Flag `NEEDS USER INPUT` for missing DOIs, dates, or author order; never fabricate journals, amounts, or co-authors.

3. **Section architecture (include / cut / merge)** — Default academic stack (reorder only when the frame demands it):
   - **Identity & contact** → **Research interests** (optional, short) → **Education** → **Appointments / experience** (reverse chronological) → **Publications** → **Grants & funding** → **Teaching** → **Invited talks** → **Service & leadership** → **Awards** → **Skills / methods** (sparse) → **Supervision & mentoring**.
   - **Cut** high-school, obsolete jobs unrelated to the search, duplicate talk lists, full grant narratives, hobby sections unless normative in field.
   - **Merge** thin consulting into appointments; **split** publications into *Selected* vs *Full* when lists exceed ~15–20 items or the user targets a page cap.

4. **Scan layers** — **Glance**: name, affiliation, contact, brief interests. **Scan**: section titles and entry leaders. **Detail**: dates, venues, IDs—subordinate in design intent.

5. **Entry patterns** — **Publications**: one style (authors, title, venue, year, DOI); peer-reviewed vs invited; own-name emphasis only if field-appropriate. **Appointments**: role, unit, dates; 0–3 impact lines with user-supplied metrics. **Grants/teaching**: agency, amounts, course + term when known.

6. **Tailoring pass** — For a stated search, elevate matching themes (methods, disease area, pedagogy) via section order, a short interests line, and *Selected publications*—not by adding unverified work.

7. **Design intent** — Tone, hierarchy, density for long documents:
   - **Tone**: scholarly, precise, calm; no marketing superlatives; active voice for impact lines, neutral for bibliography.
   - **Hierarchy**: name and current affiliation dominate; section titles clear and conventional; entry titles stronger than dates and parentheticals.
   - **Density**: list-forward; avoid essay blocks; publications as continuous list rhythm (hanging indent intent); whitespace between major sections, tight within homogeneous lists.
   - **Palette**: restrained academic; one accent; readable contrast in long lists.

**Rules & constraints**

- **Truth gate**: every degree, title, paper, grant, and number traces to user input or an explicit stub.
- **Bibliography gate**: do not invent DOIs, page numbers, or author lists; use `metadata incomplete` rather than guessing.
- **Length gate**: overflow → shorten via *Selected* lists and cut ancillary sections in the blueprint, not microscopic type.
- **Consistency gate**: one date format, one citation style, stable institution names across sections.
- **Resume boundary**: if the user needs a 1-page industry resume, redirect to **k2f-resume** in the blueprint note.

**Common failure modes**

- Resume layout on a faculty CV → scholarly sections and publication emphasis.
- Publication dump without citation rules → one style block for all entries.
- Equal weight on every item → *Selected* plus optional full lists.
- Missing appointment dates or degree years → halt with `NEEDS USER INPUT`; do not interpolate.
- Keyword stuffing in a “Skills” wall → compact methods line or categorized skills tied to real work.
- Inconsistent bolding of author name or mixed citation formats → single rule in design intent.
- Burying current affiliation below ancient history → appointments and education ordered reverse-chron within each section.

**Quality checklist**

- [ ] Frame: discipline, stage, opportunity, page budget, template constraints noted.
- [ ] Section stack is conventional for field; cuts and *Selected* vs *Full* choices documented.
- [ ] Glance layer shows identity, affiliation, and positioning without scrolling.
- [ ] Publications (and grants/talks) share one citation pattern; own-name emphasis rule stated.
- [ ] Appointments and education are complete on dates and institutions or flagged stubs.
- [ ] Tailoring (if any) uses reorder/selection only—no fabricated entries.
- [ ] Design intent covers tone, hierarchy, density, list rhythm, and accent for multi-page read.

### Step 2. Use the content_and_design.md and k2f skill to generate a K2F file and export to an appropriate file format.

Follow **k2f** to instantiate sections and flowing lists from the blueprint, apply roles from design intent, verify, and export the submission-ready CV (typically PDF or DOCX).
