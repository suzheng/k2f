---
name: k2f-policy-document
description: Draft employee- and organization-facing policy documents—handbook sections, HR rules, conduct, benefits summaries, and compliance procedures—from user source material, outlines, or legacy policy text. Use when the user asks for a policy document, employee handbook section, HR policy, workplace rule write-up, or formal policy refresh. Create policy documents with AI-native design and the highest aesthetic standards, powered by the K2F format and engine. Purpose-built for AI agents, K2F lets them focus on semantic content and design intent while automatically handling layout, typography, and formatting complexity.
---

# K2F Policy Document — AI-Native Aesthetic Design

## Purpose

Teach agents to blueprint **policy documents** before layout: scannable rule architecture, plain-language layers for employees, and formal hierarchy—without inventing benefits, legal obligations, jurisdictions, or approval paths the user or HR/legal did not supply.

## Built on K2F

This skill produces policy documents as `.K2F` packages. Follow the **k2f** skill for authoring: unpack or init a workspace, edit the semantic tree, run pack/verify, then export. K2F keeps agents on *meaning* (sections, emphasis, tables)—not pixel coordinates—while the engine compiles layout, typography, and theme into consistent export-ready output.

Before you build, install the core skill: `npx skills add suzheng/k2f --skill k2f`.

## When to Use

- New or refreshed **single-policy** pages (PTO, remote work, expenses, conduct, travel, security, leave types).
- Handbook **sections** that must read consistently with house style but stay findable on first scan.
- Reformatting draft or pasted policy language into a publishable structure for HR or legal review.
- Not for chat-only policy Q&A, AgentRC scoring policies, or counsel-grade contracts (use **k2f-contract**).

## Workflow

### Step 1. Generate the content_and_design.md

One blueprint pairing **information architecture** with **design intent**. Label blocks the **k2f** skill can map to sections, callouts, and emphasis later.

**Sub-steps**

1. **Document frame** — Policy title, document ID if user provided, **effective date**, version, policy owner (team/role), and **applicability** (entities, locations, employment types). State whether this is **new**, **amendment**, or **supersedes** prior text; link prior version only from user inputs.

2. **Purpose & scope** — Two short blocks: why the policy exists (one paragraph max) and who/when it applies. Cut narrative that repeats the title; flag `HR VERIFY` when scope crosses regions the user did not specify.

3. **At-a-glance layer** — **Quick summary** (3–5 bullets or one short paragraph): what employees must know before reading detail. This is the scan anchor—not a second copy of every rule.

4. **Definitions** — Only terms that change compliance meaning in *this* policy; omit dictionary glossaries.

5. **Policy statements** — Numbered **rules**: clear subject, modal verb (`must` / `may not` / `requires approval`), and outcome. One obligation per bullet; no mixed “policy + how-to” in the same bullet.

6. **Procedures** — Separate numbered flows: how to request, report, escalate, or document compliance (forms, systems, timelines from user only). Procedures reference rules; they do not restate them in prose.

7. **Roles & responsibilities** — Compact matrix or bullets: employee, manager, HR, IT, finance, legal—who does what at each step.

8. **Exceptions & approvals** — Who may grant exceptions, required documentation, and where records live. If unknown, `APPROVAL PATH — USER TO CONFIRM`.

9. **Related policies & references** — Cross-links and external regs **only** when user named them; no fabricated citations.

10. **Enforcement & questions** — Consequences or progressive discipline **only if user supplied**; otherwise `LEGAL/HR TO CONFIRM`. **Who to contact** for edge cases and a single questions channel.

11. **Revision history** — Table or list: version, date, summary of change, approver—placeholders marked TBD until user fills.

12. **Design intent** — Clarity over decoration:
    - **Tone**: neutral, authoritative, plain language in summary; formal precision in numbered rules; no marketing or apology voice.
    - **Hierarchy**: title and applicability first; **at-a-glance** and **policy statements** strongest; procedures and roles secondary; revision history tertiary.
    - **Density**: moderate—short paragraphs, bullets for rules and procedures; tables for roles or thresholds when user provided numbers; avoid wall-of-text essays.
    - **Palette**: restrained neutrals, high body contrast, minimal accent on **must/may not**, deadlines, and approval paths; WCAG AA; no stock imagery that competes with rules.

**Rules & constraints**

- **Source gate**: mandatory or jurisdiction-specific language stays **verbatim** from user; mark all other drafts `FOR HR/LEGAL REVIEW`.
- **Honesty gate**: no invented PTO days, dollar limits, legal citations, systems, or approver names.
- **Separation gate**: rules in **policy statements**; steps in **procedures**; do not hide the rule inside a procedure paragraph.
- **Applicability gate**: every obligation names who it binds; ambiguous geography → flag, do not assume US-default silently.
- **Single-topic gate**: one policy subject per blueprint; split unrelated topics (e.g., travel vs. expenses) unless user explicitly wants a combined handbook chapter with named parts.

**Common failure modes**

- Essay introduction with no **at-a-glance** → employees never find the rule.
- Rules buried in FAQ narrative → extract imperatives into numbered **policy statements**.
- Procedures before rules → reorder so “what” precedes “how.”
- Duplicate quick summary and full section → summary stays short; detail lives below.
- Generic AI handbook voice → replace with user terms, systems, and approvers or mark TBD.
- Missing **who to contact** → edge cases bounce to informal channels.
- Amendment without **what changed** in revision history → compliance tracking breaks.
- Mixing employee-facing plain text with privileged legal analysis → keep external-facing tone only.

**Quality checklist**

- [ ] Frame includes applicability, effective date, and new/amend/supersedes status.
- [ ] At-a-glance layer stands alone; policy statements are numbered and imperative.
- [ ] Procedures separated; roles and exception path present or marked TBD.
- [ ] User-mandated text verbatim; no invented limits, citations, or contacts.
- [ ] Related policies listed only from user inputs.
- [ ] Design intent specifies tone, hierarchy, density, and emphasis for rules vs. procedures.

### Step 2. Use the content_and_design.md and k2f skill to generate a K2F file and export to an appropriate file format.

Follow **k2f** to build the policy from the blueprint, apply design-intent roles, verify, and export for HR/legal review (typically PDF or DOCX).
