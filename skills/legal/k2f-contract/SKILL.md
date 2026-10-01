---
name: k2f-contract
description: Draft commercial agreements (NDA, MSA, SOW, SaaS, vendor) from user terms and deal context—signature-ready layout, counsel review required, no invented law. Use when the user asks to draft, author, template, or format a contract or agreement (not review-only). Create contracts with AI-native design and the highest aesthetic standards, powered by the K2F format and engine. Purpose-built for AI agents, K2F lets them focus on semantic content and design intent while automatically handling layout, typography, and formatting complexity.
---

# K2F Contract — AI-Native Aesthetic Design

## Purpose

Teach agents to blueprint **contracts** before layout: agreement frame, party roles, consistent clause stack, and formal hierarchy for navigation and signing—without fabricating parties, amounts, governing law, or positions the user did not supply.

## Built on K2F

This skill produces contracts as `.K2F` packages. Follow the **k2f** skill for authoring: unpack or init a workspace, edit the semantic tree, run pack/verify, then export. K2F keeps agents on *meaning* (articles, defined terms, exhibits, signature blocks)—not pixel coordinates—while the engine compiles layout, typography, and theme into consistent export-ready output.

Before you build, install the core skill: `npx skills add suzheng/k2f --skill k2f`.

## When to Use

- Authoring a new NDA, MSA, SOW, SaaS order form, vendor agreement, or amendment from deal notes or a playbook.
- Reformatting messy draft text into a signable structure with exhibits and defined terms.
- Turning approved commercial terms into a polished agreement for counterparty circulation (counsel review still required).

## Workflow

### Step 1. Generate the content_and_design.md

Single blueprint for semantic content and aesthetic intent. Use labeled blocks the **k2f** skill can map to articles and pages later.

**Sub-steps**

1. **Agreement frame** — Type (NDA, MSA, SOW, license, SaaS, procurement, etc.), effective date logic, and **which party you draft for** (supplier, customer, licensor, licensee). State governing jurisdiction and currency **only if user provided**. Omit entire article families the type does not need (e.g., skip payment schedules on a mutual NDA).

2. **Parties & recitals** — Legal names, entity types, addresses, and defined short names. Recitals: background facts from user inputs only—no invented deal size or strategic narrative. Flag missing signatory authority or entity details as `NEEDS USER CONFIRMATION`.

3. **Defined terms block** — Centralize capitalized terms used across articles (Services, Deliverables, Confidential Information, Fees). One definition per term; no duplicate glossaries in body text.

4. **Clause architecture (include / cut / order)** — Plan articles before drafting prose. Default stack; trim per frame:
   - Scope / Services → Fees & payment → Term & termination → IP → Confidentiality → Data protection (or DPA exhibit pointer) → Warranties → Limitation of liability → Indemnification → Insurance → General (assignment, notices, force majeure, entire agreement, amendments, counterparts).
   - **Cut** duplicate obligations and marketing language.
   - **Cross-link**: liability cap vs indemnity; survival vs termination—no contradictions.

5. **Commercial & risk allocation (user-sourced)** — Payment, caps, indemnity, IP, termination, and data terms **verbatim or TBD**. Label playbook clauses *must-have*, *negotiable*, or *placeholder*—do not soften must-haves.

6. **Exhibits & schedules** — Move long tables (SOW milestones, SLA, DPA, pricing) to exhibits with stable labels (Exhibit A, Schedule 1). Body references exhibits by label; avoid orphan exhibits or terms defined only in attachments.

7. **Execution block** — Signature lines, name/title/date per party; witness/notary only if user requested. Match party order to the preamble.

8. **Design intent** — Formal document tone and hierarchy for the engine:
   - **Tone**: precise, neutral, institution-grade; no sales voice inside operative articles.
   - **Hierarchy**: agreement title and party block strongest on cover/first pages; **article numbers** (1, 1.1, 1.1(a)) as primary scan anchors; defined-terms section visually distinct; exhibits and signatures clearly terminal.
   - **Density**: moderate legal density—short paragraphs, numbered lists for obligations; avoid wall-of-text articles; tables for fees and deliverables only in exhibits.
   - **Palette**: restrained neutrals, minimal accent (signature/date fields); high contrast for body text; WCAG AA; no decorative graphics that compete with clause numbers.

**Rules & constraints**

- **Counsel gate**: state clearly the blueprint is not legal advice; binding language requires qualified counsel review.
- **Side gate**: obligations and protections must favor the documented drafting party; if side is unknown, stop and ask—wrong-side drafting inverts risk.
- **Silence gate**: flag absent material topics (liability cap, change orders, data processing) instead of inventing market-standard terms.
- **Consistency gate**: defined terms, exhibit references, and party names match everywhere; one currency and one governing-law block unless user confirms split structures.
- **Honesty gate**: never fabricate registration numbers, insurance limits, fee amounts, or statutory citations.

**Common failure modes**

- Pasting deal email as unstructured paragraphs → re-architect into numbered articles and exhibits.
- Definitions scattered through body → consolidate Defined Terms; fix capitalization drift.
- Liability and indemnity that read unlimited while a “cap” exists elsewhere → reconcile in blueprint notes before layout.
- SOW scope in prose without deliverables, acceptance, or change process → add exhibit table or mark gaps.
- Generic “Agreement” with no parties or effective date → fill from user or flag.
- Over-designed marketing cover on a contract → shift flair to title block only; keep articles austere.

**Quality checklist**

- [ ] Contract type, drafting side, and omitted articles are explicit.
- [ ] Parties and defined terms are complete and consistent with exhibits.
- [ ] Each material topic is present, deliberately omitted, or marked TBD—not silently assumed.
- [ ] Cross-clause interactions (cap, indemnity, survival, termination) noted without contradiction.
- [ ] Exhibits referenced in body exist; signature block matches party count.
- [ ] Design intent names emphasis for article numbering, defined terms, exhibits, and signatures.
- [ ] Counsel-review reminder included; no fabricated legal or commercial facts.

### Step 2. Use the content_and_design.md and k2f skill to generate a K2F file and export to an appropriate file format.

Follow **k2f** to instantiate the contract from the blueprint, apply article and exhibit roles from design intent, verify, and export the signable deliverable.
