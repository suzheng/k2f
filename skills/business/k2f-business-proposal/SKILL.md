---
name: k2f-business-proposal
description: Turn briefs and discovery notes into persuasive business proposals with a decision-ready narrative, phased solution, and clear investment for executives who skim first. Use when the user asks for a business proposal, project proposal, commercial offer, or client-facing pitch (not a full legal SOW). Create business proposals with AI-native design and the highest aesthetic standards, powered by the K2F format and engine. Purpose-built for AI agents, K2F lets them focus on semantic content and design intent while automatically handling layout, typography, and formatting complexity.
---

# K2F Business Proposal — AI-Native Aesthetic Design

## Purpose

Teach agents to blueprint **business proposals** before layout: a self-contained story that earns trust, scopes deliverables, and surfaces price clearly—without inventing budgets, rates, case studies, or terms the user did not supply.

## Built on K2F

This skill produces business proposals as `.K2F` packages. Follow the **k2f** skill for authoring: unpack or init a workspace, edit the semantic tree, run pack/verify, then export. K2F keeps agents on *meaning* (section roles, narrative arc, tables)—not pixel coordinates—while the engine compiles layout, typography, and theme into consistent export-ready output.

Before you build, install the core skill: `npx skills add suzheng/k2f --skill k2f`.

## When to Use

- Drafting a project or services proposal from RFP excerpts, discovery notes, or a solution brief.
- Rebuilding a generic AI proposal so executive summary, scope, and pricing tell one coherent story.
- Preparing a formal offer where **validity**, **inclusions/exclusions**, and **next steps** must be explicit.

## Workflow

### Step 1. Generate the content_and_design.md

Single blueprint for semantic content and aesthetic intent. Use labeled blocks the **k2f** skill can map to sections later.

**Sub-steps**

1. **Proposal frame** — Audience (buyer vs evaluator), engagement type (fixed, phased, retainer), and desired decision. Set **valid-until** (default 30 days if user silent). Omit sections the frame does not need.

2. **Cover & metadata** — Title, optional subtitle, client organization and contact, your firm and contact, date, version. One currency throughout; flag locale formatting if known.

3. **Narrative spine (CLOSE)** — Plan flow before drafting sections:
   - **Context**: their situation in their language (from user inputs).
   - **Loss**: cost of inaction—quantified only when data exists; otherwise qualitative and labeled.
   - **Outcome**: future state and business value (measurable where possible).
   - **Solution**: approach, phases, deliverables, success criteria.
   - **Evidence**: proof you can deliver—case snippets, methodology, credentials **user provided**.
   Executive summary must stand alone (~400 words max): entire proposal understandable from that block alone.

4. **Section architecture (include / cut)** — Default stack; drop or shorten per frame:
   - Executive Summary → Understanding / Current State & Challenges → Proposed Approach (overview + phased delivery) → Expected Outcomes & ROI → Team & Governance (if relevant) → Investment → Why Us → Terms → Next Steps.
   - **Cut** feature dumps, duplicate methodology, competitor attacks, and deep legal boilerplate.

5. **Scope & delivery** — Per phase: objective, key activities, deliverables, success criteria, duration. **What's included** and **what's not included** as explicit lists—prevent day-one disputes. Assumptions and client dependencies in a distinct block, not buried in prose.

6. **Investment block** — Pricing model only if user specified. Table-first fees; payment milestones **verbatim from user**. Never fabricate rates or tax; use `NEEDS USER CONFIRMATION` when math is incomplete. Scannable path: model → totals → net → schedule.

7. **Commercial closure** — Start assumptions, duration, location, travel/expenses policy from user. IP, confidentiality, termination, change process at summary level; note legal review recommended. Numbered **next steps** with dates or triggers.

8. **Design intent** — Tone, hierarchy, density, palette for the engine:
   - **Tone**: consultative B2B; trust through understanding, not hype. Lead with their problem; keep price findable.
   - **Hierarchy**: executive summary and **Understanding** strongest; conclusion-led phase titles; total never buried.
   - **Density**: bullets and tables per phase; ~6-line paragraph cap; one outcomes/ROI block, not repeated.
   - **Palette**: restrained neutrals, one accent on phases, totals, CTA; WCAG AA; tabular fees, not inline dollars.

**Rules & constraints**

- **Discovery honesty**: thin inputs → narrow claims and mark assumptions; never invent client context.
- **Story gate**: CLOSE flow must read as narrative, not a checklist of empty headings.
- **Self-contained gate**: a reader without your chat history understands problem, offer, price boundaries, and what to do next.
- **Traceability gate**: every figure, timeline, and credential ties to user input or is explicitly marked TBD.
- **Separation gate**: client situation, your solution, and commercial terms stay in labeled sections—do not merge RFP text into deliverable descriptions.

**Common failure modes**

- Feature-dumping without linking to client pain → rewrite each capability as outcome tied to a stated challenge.
- Burying price or vague scope → promote investment summary; split inclusions/exclusions.
- Executive summary that is marketing fluff → replace with CLOSE paragraphs using client-specific facts.
- Phases with activities but no deliverables or success criteria → add both per phase or merge phases.
- Invented case studies, team names, or payment terms → strip; use placeholders.
- Missing valid-until or next steps → add in metadata and closure.

**Quality checklist**

- [ ] Executive summary stands alone and mirrors body (no contradictions).
- [ ] At least one section demonstrates deep understanding of **their** situation (user-sourced).
- [ ] Every phase has deliverables and success criteria; inclusions/exclusions align with phases.
- [ ] Investment tables reconcile; pricing flagged if unvalidated.
- [ ] Valid-until, version, and dates are consistent.
- [ ] Design intent names emphasis targets; no fabricated metrics or legal commitments.

### Step 2. Use the content_and_design.md and k2f skill to generate a K2F file and export to an appropriate file format.

Follow **k2f** to instantiate the proposal from the blueprint, apply section roles from design intent, verify, and export the client-ready deliverable.
