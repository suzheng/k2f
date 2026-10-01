---
name: k2f-purchase-order
description: Create issuance-ready purchase orders from requisitions, quotes, or line-item lists—structured so vendors can fulfill, ship, and invoice without clarification loops. Use when the user asks for a purchase order, PO, procurement order, or buyer-issued order to a supplier. Create purchase orders with AI-native design and the highest aesthetic standards, powered by the K2F format and engine. Purpose-built for AI agents, K2F lets them focus on semantic content and design intent while automatically handling layout, typography, and formatting complexity.
---

# K2F Purchase Order — AI-Native Aesthetic Design

## Purpose

Teach agents to blueprint purchase orders before layout: unambiguous buyer–vendor identity, ship and bill routing, a reconciled line-item commitment, and visual hierarchy that surfaces **what is ordered, for where, by when, and under which terms**—without inventing SKUs, prices, or authorization the user did not supply.

## Built on K2F

This skill produces purchase orders as `.K2F` packages. Follow the **k2f** skill for authoring: unpack or init a workspace, edit the semantic tree, run pack/verify, then export. K2F keeps agents on *meaning* (parties, line roles, totals, delivery block)—not pixel coordinates—while the engine compiles layout, typography, and theme into consistent output across PDF and other formats.

Before you build, install the core skill: `npx skills add suzheng/k2f --skill k2f`.

## When to Use

- Issuing a PO from an approved quote, requisition, catalog cart, or pasted line items.
- Rebuilding a vague or generic AI PO so references, quantities, and totals match what procurement will send.
- Standard goods PO, services PO, or blanket/call-off PO where only user-defined scope belongs on the document.

## Workflow

### Step 1. Generate the content_and_design.md

Single blueprint for semantic content and aesthetic intent. Use labeled blocks the **k2f** skill can map to regions later.

**Sub-steps**

1. **PO type & commitment frame** — Standard goods, services, blanket/call-off, or change order against an existing PO. Note currency, locale formatting, and whether prices are firm or estimate-only. Omit sections the type does not need (e.g., skip ship-to on pure services if user says digital delivery only).

2. **Party & routing block** — **Buyer** (issuing entity): legal name, address, contact, logo intent. **Vendor**: supplier name and address **only from user**. **Ship-to** and **bill-to** when they differ from buyer HQ; attention or receiving dock notes verbatim. Never fabricate vendor tax IDs, DUNS, or buyer approver names.

3. **Reference metadata** — PO number, issue date, requested or **expected delivery** date (header and/or per line if user supplied). Linked quote, RFQ, requisition, contract, or prior PO for amendments. One currency for the whole document; flag missing delivery expectation before layout.

4. **Line-item architecture** — Table-first, not narrative paragraphs. Per row: line number, description, optional part/SKU/model, quantity, unit, unit price, extended amount. Group sections (hardware vs installation) only when user data already splits that way. Change-order lines reference what they replace or add—never merge unrelated SKUs into one vague row.

5. **Totals story (write the math in the doc)** — Subtotal → freight/handling, each tax or fee with labeled rate → **PO total**. State rounding (per line vs document). If quote and requisition totals conflict, mark `NEEDS USER CONFIRMATION` instead of picking one silently.

6. **Terms & fulfillment footer** — Payment terms, incoterms or shipping method **verbatim from user**. Special instructions (partial ship OK, packing, invoicing rules, remit-to). Authorized-by or signature block only if user named a role or name. Keep legal boilerplate minimal unless user supplied text.

7. **Design intent** — Tone, hierarchy, density, palette for the engine:
   - **Tone**: authoritative, operational B2B; zero sales language—this is a binding request to supply.
   - **Hierarchy**: “Purchase Order” + PO number and **vendor name** scannable in header; **line table** primary; **ship-to** and **PO total** strong in secondary zones; terms footer tertiary.
   - **Density**: warehouse-friendly table rhythm; stable column order (#, item, qty, unit price, extended); right-align money; tabular numerals; one compact notes block max on the main view.
   - **Palette**: restrained neutrals, one accent on PO number and total; WCAG AA; clear separation between routing addresses and line data.

**Rules & constraints**

- **Issuance gate**: no send-ready blueprint without PO number, issue date, currency, reconciled total, vendor identity, and ship/bill routing when physical goods are involved.
- **Single source of truth**: line extensions drive subtotal; fees and taxes apply to the documented base only.
- **Reference discipline**: quote/req/contract IDs appear in metadata and optionally per line—not duplicated in prose.
- **Honesty gate**: mark missing tax, freight, or authorization; never infer catalog prices or approval without user input.

**Common failure modes**

- Paragraph “we need 50 laptops” → rewrite as numbered rows with qty, unit, and price.
- Missing ship-to or conflated buyer and ship-to → split addresses explicitly.
- Quote marketing copy in line descriptions → replace with fulfillment labels (SKU, spec, qty).
- Buried PO total or delivery date → promote in metadata and design intent.
- Invented part numbers, payment terms, or approver → strip; use `TBD — user must supply`.
- Mixed currencies or duplicate line numbers → renumber or escalate.

**Quality checklist**

- [ ] Buyer, vendor, and ship/bill addresses match user-supplied data only.
- [ ] Line qty × unit price reconciles to subtotal under stated rounding.
- [ ] Fees/taxes sum to PO total; amendment POs reference parent PO if applicable.
- [ ] Expected delivery aligns with terms and line-level dates when both exist.
- [ ] Design intent names emphasis for PO number, vendor, table, ship-to, and total.
- [ ] All figures traceable to user inputs or explicit formulas in the doc.

### Step 2. Use the content_and_design.md and k2f skill to generate a K2F file and export to an appropriate file format.

Follow **k2f** to instantiate the purchase order from the blueprint, apply roles from design intent, verify, and export the issuance-ready deliverable.
