---
name: k2f-quotation
description: Create client-ready sales quotations from scope notes, line items, rates, and terms—structured so buyers can compare options, accept, or convert to an invoice without rework. Use when the user asks for a quotation, quote, estimate, proposal pricing, or RFQ response. Create quotations with AI-native design and the highest aesthetic standards, powered by the K2F format and engine. Purpose-built for AI agents, K2F lets them focus on semantic content and design intent while automatically handling layout, typography, and formatting complexity.
---

# K2F Quotation — AI-Native Aesthetic Design

## Purpose

Teach agents to blueprint **commercial quotations** before layout: a scannable offer story, reconciled pricing, explicit boundaries (what is in / out), and design intent that foregrounds **value, total, and validity**—without inventing rates, deliverables, or legal terms the user did not supply.

## Built on K2F

This skill produces quotations as `.K2F` packages. Follow the **k2f** skill for authoring: unpack or init a workspace, edit the semantic tree, run pack/verify, then export. K2F keeps agents on *meaning* (parties, scope roles, option tiers, totals)—not pixel coordinates—while the engine compiles layout, typography, and theme into consistent output across PDF and other formats.

Before you build, install the core skill: `npx skills add suzheng/k2f --skill k2f`.

## When to Use

- Drafting a new quote from a brief, RFP excerpt, discovery notes, or a rate card.
- Rebuilding a generic AI quote so scope, math, and validity read as one coherent offer.
- Presenting **good / better / best** tiers or optional add-ons the user already defined.

## Workflow

### Step 1. Generate the content_and_design.md

Single blueprint for semantic content and aesthetic intent. Use labeled blocks the **k2f** skill can map to regions later.

**Sub-steps**

1. **Quote type & decision frame** — Fixed price, time-and-materials cap, milestone-based, or tiered packages. Note currency, locale formatting, tax treatment (inclusive/exclusive/unknown), and whether this is binding or indicative. Omit sections the type does not need (e.g., skip acceptance block on a rough ballpark).

2. **Party & identity** — Seller and client names, addresses, contacts, logo intent. Registration or tax IDs **only if user provided**. Never fabricate signatures or authorized signatory names.

3. **Reference metadata** — Quote number, issue date, **valid-until** date (or explicit validity rule). Project name, RFP/PO reference, version (v1, v2). One currency for the whole document; flag missing validity before layout.

4. **Offer narrative (assertion, then evidence)** — One short block states **what the client gets and why this price** (outcome-oriented, not marketing fluff). Follow with evidence: line table, tier summary, or milestone list—not long prose that repeats table rows. If the user pasted client requirements, keep **verbatim ask** in a distinct subsection and map **your priced response** beside it; do not merge RFP text into line descriptions.

5. **Line-item architecture** — Table-first. Per row: deliverable or SKU label, qty, unit, unit price, line total. Group sections (implementation, licensing, support) only when user data already splits that way. Optional add-ons and alternates as labeled blocks—not hidden footnotes. Discounts as explicit lines or documented percent rules.

6. **Totals story (write the math)** — Subtotal → taxes/fees with labeled rates → **quote total** → optional “from” pricing when tiers exist. State rounding (per line vs document). If inputs conflict, mark `NEEDS USER CONFIRMATION` instead of guessing.

7. **Boundaries & terms** — Inclusions, exclusions, assumptions, dependencies on client inputs. Payment or deposit milestones **verbatim from user**. Acceptance steps, change-order pointer, and warranty/support only if supplied. No invented liability caps or SLA numbers.

8. **Design intent** — Tone, hierarchy, density, palette for the engine:
   - **Tone**: confident, precise B2B; persuasive through clarity, not hype.
   - **Hierarchy**: quote title + number in header; **scope headline** and **quote total** strongest; line table secondary; validity and exclusions visible without hunting.
   - **Density**: readable table rhythm; right-align money; tabular numerals; at most one short assumptions block on the main view.
   - **Palette**: restrained neutrals, one accent on total and primary CTA (e.g., “Accept by …”); WCAG AA; tier columns visually parallel for comparison.

**Rules & constraints**

- **Acceptance gate**: no send-ready blueprint without quote number, dates, currency, reconciled total, and stated validity.
- **Separation gate**: narrative, priced facts, and legal/assumption notes stay in distinct labeled sections—readers must not confuse interpretation with committed scope.
- **Traceability gate**: every figure and deliverable ties to user input or a formula written in the doc; mark missing tax or FX explicitly.
- **Honesty gate**: no fabricated upsells, cross-sells, or “this also covers …” unless the user said so.

**Common failure modes**

- Paragraph pricing → rewrite as rows with qty, unit, and amount.
- Buried total or missing valid-until → promote in metadata and design intent.
- RFP prose pasted into line items → split verbatim ask vs priced response.
- Tier options unequal or incomparable columns → align units, term length, and subtotals per tier.
- Invented rates, VAT, or payment terms → strip; use `TBD — user must supply`.
- Marketing superlatives with no scoped deliverable → replace with measurable inclusion lines.

**Quality checklist**

- [ ] Parties and IDs match user-supplied data only.
- [ ] Line math reconciles to subtotal and total under stated rounding.
- [ ] Valid-until and version align with narrative (no stale dates).
- [ ] Inclusions/exclusions do not contradict line items.
- [ ] Design intent names emphasis for scope headline, total, validity, and table.
- [ ] Optional tiers are comparable; primary recommendation is obvious if user named one.

### Step 2. Use the content_and_design.md and k2f skill to generate a K2F file and export to an appropriate file format.

Follow **k2f** to instantiate the quotation from the blueprint, apply roles from design intent, verify, and export the client-ready deliverable.
