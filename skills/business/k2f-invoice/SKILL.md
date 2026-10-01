---
name: k2f-invoice
description: Create payment-ready invoices from line items, rates, client details, tax rules, and terms—structured so totals reconcile and AP can pay without back-and-forth. Use when the user asks for an invoice, bill, receivable, pro forma invoice, or branded invoice PDF. Create invoices with AI-native design and the highest aesthetic standards, powered by the K2F format and engine. Purpose-built for AI agents, K2F lets them focus on semantic content and design intent while automatically handling layout, typography, and formatting complexity.
---

# K2F Invoice — AI-Native Aesthetic Design

## Purpose

Teach agents to blueprint invoices before layout: legally and operationally complete party and reference data, a reconciled line-item and tax math story, and visual hierarchy that surfaces **what is owed, by when, and how to pay**—without inventing banking, tax IDs, or amounts the user did not supply.

## Built on K2F

This skill produces invoices as `.K2F` packages. Follow the **k2f** skill for authoring: unpack or init a workspace, edit the semantic tree, run pack/verify, then export. K2F keeps agents on *meaning* (parties, line roles, totals, payment block)—not pixel coordinates—while the engine compiles layout, typography, and theme into consistent output across PDF and other formats.

Before you build, install the core skill: `npx skills add suzheng/k2f --skill k2f`.

## When to Use

- Issuing a new invoice from a quote, timesheet, contract, or pasted line items.
- Rebuilding a messy or generic AI invoice so math, references, and payment details align.
- Pro forma or deposit invoices where due date and amount semantics differ from a standard bill.

## Workflow

### Step 1. Generate the content_and_design.md

Single blueprint for semantic content and aesthetic intent. Use labeled blocks the **k2f** skill can map to regions later.

**Sub-steps**

1. **Invoice type & jurisdiction lock** — Standard bill, pro forma, deposit/partial, or credit-style adjustment. Note currency, locale (date and number formatting), and whether tax is inclusive or exclusive. Omit sections the type does not need (e.g., skip “amount due” hero on non-collecting pro forma).

2. **Party & identity block** — Seller: legal name, address, contact, logo intent, tax/VAT ID **only if user provided**. Buyer: bill-to name and address; optional ship-to or attention line. Never fabricate registration numbers or bank details.

3. **Reference metadata** — Invoice number, issue date, due date (or explicit “Due on receipt / Net N”). PO, project, contract, or customer reference when supplied. One currency code for the whole document; flag missing due date before layout.

4. **Line-item architecture** — Table-first, not prose paragraphs. Per row: clear description, quantity, unit, unit price, line amount. Group optional sections (services vs expenses, milestones) only when the user’s data already splits that way. Discounts as explicit negative lines or percent note tied to a subtotal rule—never hidden in copy.

5. **Totals story (write the math in the doc)** — Subtotal → each tax/discount line with rate label → **total** → optional amount paid / balance due. State rounding rule (per line vs invoice-level) and reconcile: line sums, tax base, and total must match within one cent of stated rule. If inputs conflict, mark `NEEDS USER CONFIRMATION` instead of guessing.

6. **Payment & compliance footer** — Payment terms in plain language; methods the user named (wire, ACH, card link, check). Bank/wallet fields **verbatim from user**; pay-link call-to-action when a hosted URL exists. Late-fee or remittance notes only if provided. Keep legal boilerplate minimal unless user supplied text.

7. **Design intent** — Document tone, hierarchy, density, palette for the engine:
   - **Tone**: calm, trustworthy B2B; no marketing fluff on a collection document.
   - **Hierarchy**: document title + invoice number scannable in header; **amount due** and **due date** strongest emphasis in totals/payment zone; line table secondary; footer tertiary.
   - **Density**: generous table row rhythm; right-align money columns; tabular numerals for all amounts; avoid more than one short notes block.
   - **Palette**: restrained neutrals with one accent on total/due; WCAG AA; logo space without decorative clutter.

**Rules & constraints**

- **Payability gate**: no send-ready blueprint without invoice number, dates, currency, reconciled total, and user-confirmed pay instructions when payment is expected.
- **Single source of truth**: line items drive subtotal; tax rates apply to the documented base only.
- **Reference discipline**: PO/project IDs appear once in metadata and optionally on lines—not repeated in random paragraphs.
- **Honesty gate**: mark missing tax treatment or FX; never infer statutory rates without user input.

**Common failure modes**

- Paragraph line items → rewrite as table rows with qty/rate/amount.
- Tax on wrong base or double-counted → restate base, rate, and rounded tax lines explicitly.
- Generic “Invoice” with no due date or number → fill from user or flag gaps.
- Amount due buried below fold in design intent → promote total/due in hierarchy notes.
- Invented IBAN, VAT, or pay links → strip and replace with `TBD — user must supply`.
- Mixed currencies on one invoice → split or escalate unless user confirms FX line.

**Quality checklist**

- [ ] Parties and tax IDs match user-supplied data only.
- [ ] Line amounts × qty reconcile to subtotal under stated rounding.
- [ ] Tax/discount lines sum to total; balance due correct if partial payment noted.
- [ ] Due date and payment instructions align with terms.
- [ ] Design intent names emphasis for amount due, table, and payment block.
- [ ] All figures traceable to user inputs or explicit formulas in the doc.

### Step 2. Use the content_and_design.md and k2f skill to generate a K2F file and export to an appropriate file format.

Follow **k2f** to instantiate the invoice from the blueprint, apply roles from design intent, verify, and export the payment-ready deliverable.
