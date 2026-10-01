---
name: k2f-checklist
description: Blueprint operational, launch, diligence, or readiness checklists with phased flow, verifiable items, and scannable hierarchy—not a flat bullet dump or generic productivity template. Use when the user asks for a checklist, pre-flight list, deploy runbook checks, onboarding steps, audit tracker, or "what do we need before we ship." Create checklists with AI-native design and the highest aesthetic standards, powered by the K2F format and engine. Purpose-built for AI agents, K2F lets them focus on semantic content and design intent while automatically handling layout, typography, and formatting complexity.
---

# K2F Checklist — AI-Native Aesthetic Design

## Purpose

Teach agents to blueprint **checklists** in `content_and_design.md`: what must be verified, how to group and sequence items, and aesthetic intent (scanability, checkbox rhythm, operational tone)—without inventing org policy, compliance gates, or thresholds the user did not supply.

## Built on K2F

This skill produces checklists as `.K2F` packages. Follow the **k2f** skill for authoring: unpack or init a workspace, edit the semantic tree, pack/verify, and export. K2F keeps agents on *meaning*—sections, item labels, check states—not coordinates.

Before you build, install the core skill: `npx skills add suzheng/k2f --skill k2f`.

## When to Use

- Pre-launch, deploy, migration, event, travel, or handoff verification the user will run repeatedly or once under time pressure.
- Consolidating runbooks or diligence lists into one printable or fillable checklist.
- Multi-workstream readiness when grouped sections beat a flat list.

## Workflow

### Step 1. Generate the content_and_design.md

Single blueprint for checklist content and design intent. Use labeled blocks (title band, context line, sections, checkbox rows, triggers, notes) the **k2f** skill can map later.

**Sub-steps**

1. **Checklist frame** — Domain, who runs it, one-shot vs recurring use, and **page budget** (often one page per phase). Tone: calm procedural, time-critical deploy, or formal compliance.

2. **Scope & requirement audit** — Gates, tools, risks, approvals, and **decision criteria** (stop, rollback, escalate). Unknowns → `OPEN QUESTION` or `NEEDS USER INPUT`; never fabricate SLAs or thresholds.

3. **Information architecture (include / cut / merge)** — Choose the dominant spine:
   - **Chronological phases** when order matters (e.g., Pre → Execute → Post; Plan → Pack → Go).
   - **Parallel workstreams** when teams advance independently (finance, legal, ops)—each section self-contained.
   - **Hybrid** only when needed; phase headers with workstream sub-blocks.
   Default blocks:
   - **Title & context** → checklist name, date/version slot, owner or role if supplied.
   - **How to use** (2–3 lines): when to run it, estimated time, what “done” means.
   - **Core sections** → themed groups with 4–12 items each; prefer **verbs + observable outcome** (“CI green on release branch”) over topics (“Testing”).
   - **Triggers / exceptions** → rollback, stop-ship, or escalation lines the user defined.
   - **Optional appendix intent** → links, ticket IDs, or data-room pointers—not duplicate checklist items.
   **Cut** vague items (“double-check quality”), duplicates across phases, narrative paragraphs, and nice-to-haves that do not gate the stated goal.

4. **Item contract pass** — One action, one verifiable done state per row; parallel verbs across siblings. Add priority or owner only when requested. Pair “If yes…” follow-ups under the parent.

5. **Customization pass** — Add context-specific sections (migrations, flags, sector items) only from user input or named templates. Keep stable section order for recurring use.

6. **Design intent** — **Tone**: direct; urgency via section labels, not alarm copy. **Hierarchy**: title, context, then section heads over checkbox rows. **Density**: split long undifferentiated lists. **Rhythm**: one line per item; triggers separated from routine rows. **Brand**: restrained accent; print-friendly contrast.

Log **assumptions** (section order, omitted optionals) vs **open questions** (missing thresholds, owners, or policy text).

**Rules & constraints**

- **Verifiability gate**: every item answers “how would I know it’s done?”—if not, rewrite or cut.
- **Truth gate**: approvals, environments, metrics, and legal steps trace to user input or explicit stubs.
- **Scope gate**: document **in / out of scope** for this checklist; post-checklist workflow is one line, not a runbook essay.
- **One-action gate**: split compound rows (“review and deploy and notify”) into separate items or ordered sub-steps.
- **Gate ordering**: blockers (tests, approvals, backups) appear before irreversible steps in chronological checklists.
- **Living-doc gate**: note status-tracking intent only when requested; do not invent status enums.

**Common failure modes**

- Flat laundry list with no phases or themes → regroup by time or workstream.
- Items before prerequisites (“deploy to prod” before “staging verified”) → reorder chronological spine.
- Missing triggers or rollback lines when user mentioned risk → add triggers section or flag `NEEDS USER INPUT`.
- Generic template ignoring user stack → inject only user-named tools and gates.
- Essay instructions replacing checkable rows → convert to checkbox items plus a short how-to-use block.
- Duplicate checks in every phase → keep once at the earliest gating section.

**Quality checklist**

- [ ] Frame, page budget, tone, and requirement audit (gates, tools, triggers) documented.
- [ ] IA choice (phases, workstreams, or hybrid) explicit; in/out scope and cuts explained.
- [ ] Every item is one verifiable action with parallel wording; follow-ups tied to parents.
- [ ] Triggers/stop conditions present or stubbed; assumptions vs open questions listed.
- [ ] Design intent covers tone, hierarchy, density, rhythm, and brand for the full checklist.

### Step 2. Use the content_and_design.md and k2f skill to generate a K2F file and export to an appropriate file format.

Follow **k2f** to instantiate sections and checkbox items from the blueprint, apply design intent, verify, and export the runnable checklist (typically fillable PDF or DOCX).
