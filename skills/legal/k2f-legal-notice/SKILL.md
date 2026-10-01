---
name: k2f-legal-notice
description: Draft formal legal notices—litigation holds (issue, refresh, release), preservation letters, and counsel-directed custodian communications—from matter context and house templates. Use when the user asks for a legal notice, litigation hold notice, preservation notice, hold refresh, or hold release. Create legal notices with AI-native design and the highest aesthetic standards, powered by the K2F format and engine. Purpose-built for AI agents, K2F lets them focus on semantic content and design intent while automatically handling layout, typography, and formatting complexity.
---

# K2F Legal Notice — AI-Native Aesthetic Design

## Purpose

Teach agents to blueprint **legal notices** before layout: notice type, recipient-facing obligations, scannable scope and systems blocks, and formal hierarchy—without inventing jurisdiction, custodians, scope, or privilege markings the user or counsel did not supply.

## Built on K2F

This skill produces legal notices as `.K2F` packages. Follow the **k2f** skill for authoring: unpack or init a workspace, edit the semantic tree, run pack/verify, then export. K2F keeps agents on *meaning* (notice role, sections, emphasis)—not pixel coordinates—while the engine compiles layout, typography, and theme into consistent export-ready output.

Before you build, install the core skill: `npx skills add suzheng/k2f --skill k2f`.

## When to Use

- First issuance of a litigation or preservation hold to named custodians.
- Periodic reaffirmation (refresh) when scope, systems, or custodians changed.
- Release when counsel confirms preservation may end.
- Reformatting draft hold language into a consistent, institution-grade notice for attorney review.

## Workflow

### Step 1. Generate the content_and_design.md

Single blueprint for semantic content and aesthetic intent. Label blocks the **k2f** skill can map to sections and emphasis later.

**Sub-steps**

1. **Notice frame** — Classify **issue**, **refresh**, or **release**. State effective date, matter short name, issuing entity, and signer (from user or house style). For refresh, anchor to original issue date; for release, state end date and post-release retention instruction (user-sourced). Flag forum/jurisdiction only if user provided—do not invent preservation triggers or sanctions rules.

2. **Routing block** — Date, To (custodian or role), From (authorized signer), Re line (`LITIGATION HOLD NOTICE — [matter]` or release equivalent). Record intended **privilege / confidentiality marking** as `USER OR COUNSEL TO CONFIRM`—external custodian notices often differ from internal work-product headers.

3. **Why this recipient** — One plain sentence: why they are on the notice (role, project, data access). Avoid prejudicial narrative, admissions, or strategy; enough context to understand relevance.

4. **Core obligation (hero)** — Issue/refresh: **effective immediately**, preserve listed categories. Release: hold **released** as of [date]; normal retention resumes per stated instruction. This line is the visual and semantic anchor.

5. **Scope layer** — Numbered bullets: document categories, subjects, counterparties, date ranges (preserve from [event] through present). Start specific; mark `COUNSEL VERIFY — breadth` when scope is provisional. Refresh: call out **adds, amends, or removals** since last version.

6. **Systems & locations layer** — Separate block listing channels (email, chat, drives, BYOD, paper, voicemail, calendars). Keeps scope about *what* and systems about *where*.

7. **Prohibited conduct** — Short, imperative list (no delete, modify, destroy, auto-delete, inbox-zero). Issue/refresh only.

8. **Operational boundaries** — Legal contact for questions; acknowledgment method and deadline (e.g., three business days); duration until written release; periodic reaffirmation if applicable. Clarify: business discussions may continue; do not discuss the notice, litigation, or legal strategy with colleagues unless directed.

9. **Safety line** — “If unsure whether something is covered, preserve it” (issue/refresh).

10. **Signature block** — Signer name, title, contact; match house template order.

11. **Design intent** — Formal, custodian-grade clarity over ornament:
    - **Tone**: direct, authoritative, calm; no marketing or apology language.
    - **Hierarchy**: Re line and **effective immediately** strongest; numbered scope next; **DO NOT** visually distinct (weight or label, not shouty color); systems as secondary scan list; contact/acknowledgment in a closing band.
    - **Density**: moderate—short paragraphs, bullets for scope and systems; one screen-width idea per block; release notices may be a single dense paragraph plus retention line.
    - **Palette**: restrained neutrals, high body contrast, minimal accent on dates and deadlines; WCAG AA; no decorative graphics that compete with obligations.

**Rules & constraints**

- **Counsel gate**: blueprint is for attorney review; distribution to custodians requires explicit user/counsel approval.
- **Scope gate**: propose scope from user inputs; flag too-broad (operational burden) vs too-narrow (preservation gap)—never silently widen or narrow without marking `VERIFY`.
- **Custodian gate**: named individuals only from user/matter context; departed custodians on refresh → note IT-level preservation, not only individual notice.
- **Urgency gate**: if suit served or imminent, state same-day issuance in blueprint; do not bury urgency in prose.
- **Honesty gate**: no fabricated systems, custodians, legal citations, or acknowledgment URLs.

**Common failure modes**

- Essay-style scope → re-layer into numbered bullets plus systems block.
- Missing **DO NOT** / auto-delete → custodians assume normal deletion is fine.
- Prejudicial or strategic detail in the opening → shorten to neutral one-liner.
- Refresh without reaffirmation opener or scope delta → custodians cannot tell what changed.
- Release without retention instruction → ambiguous whether archives stay frozen.
- Wrong-side privilege banner on external notice → mark for counsel correction.
- Acknowledgment buried with no deadline → compliance tracking fails.

**Quality checklist**

- [ ] Notice type (issue / refresh / release) and effective dates are explicit.
- [ ] Recipient rationale, scope, systems, and contacts are complete or marked TBD.
- [ ] Prohibitions and preserve-if-unsure language present for issue/refresh.
- [ ] Refresh notes scope/custodian changes; release states retention after hold.
- [ ] Design intent names emphasis for Re line, core obligation, scope, DO NOT, and acknowledgment.
- [ ] Counsel-review reminder included; no invented legal or operational facts.

### Step 2. Use the content_and_design.md and k2f skill to generate a K2F file and export to an appropriate file format.

Follow **k2f** to instantiate the notice from the blueprint, apply section roles from design intent, verify, and export the counsel-review deliverable.
