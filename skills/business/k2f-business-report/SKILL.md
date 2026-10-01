---
name: k2f-business-report
description: Create executive-ready business reports from KPIs, sales or operational data, and stakeholder notes—with prioritized findings, trend context, and actionable recommendations. Use when the user asks for a business report, performance analysis, business review, KPI dashboard narrative, weakness analysis, or strategic improvement report from business data. Create business reports with AI-native design and the highest aesthetic standards, powered by the K2F format and engine. Purpose-built for AI agents, K2F lets them focus on semantic content and design intent while automatically handling layout, typography, and formatting complexity.
---

# K2F Business Report — AI-Native Aesthetic Design

## Purpose

Teach agents to blueprint business reports before layout: a decision-oriented story from evidence to action, honest about data gaps, and hierarchy that lets executives grasp performance in minutes—not generic prose or chart dumps without interpretation.

## Built on K2F

This skill produces business reports as `.K2F` packages. Follow the **k2f** skill for authoring: unpack or init a workspace, edit the semantic tree, run pack/verify, then export. K2F keeps agents on *meaning* (section roles, metrics, recommendations)—not pixel coordinates—while the engine compiles layout, typography, and theme into consistent output across PDF, DOCX, PPTX, and other formats.

Before you build, install the core skill: `npx skills add suzheng/k2f --skill k2f`.

## When to Use

- Weekly, monthly, or quarterly performance reviews from spreadsheets, CRM exports, or pasted metrics.
- Diagnostic reports that name weak areas, severity, and improvement strategies from user-supplied data.
- Board- or leadership-ready summaries where trends, risks, and next steps must align with one narrative thread.

## Workflow

### Step 1. Generate the content_and_design.md

Single blueprint for semantic content and aesthetic intent. Use labeled blocks the **k2f** skill can map to sections and visuals later.

**Sub-steps**

1. **Report charter** — Audience (exec, ops, board), period covered, primary question (“Are we on track?” “Where are we leaking?”), and cadence (weekly pulse vs deep quarterly). List data sources the user actually provided; mark gaps instead of inventing figures.

2. **Clarify scope (when inputs are thin)** — Which metrics matter, comparison baseline (prior period, target, budget), and whether the reader needs full detail or highlights-only. Do not expand scope beyond the charter.

3. **Information architecture** — Default spine (adapt or shorten for weekly pulses):
   - **Cover / title block**: report name, period, prepared for, date.
   - **Executive summary**: headline verdict, 3–5 bullet takeaways, top risk and opportunity—numbers only where user data supports them.
   - **Key metrics**: small set of KPIs with current value, delta vs prior period and vs target; one line of interpretation each.
   - **Performance trends**: time-series or segment story; prefer chart roles over long paragraphs when shape matters (growth, decline, volatility).
   - **Findings / weak areas**: prioritized by severity; each finding = observation + business impact + plausible driver (labeled inference if not in data).
   - **Recommendations**: 3–6 initiatives tied to findings; per initiative—objective, 3–5 prioritized actions, expected impact (H/M/L), horizon, success metrics.
   - **Appendix intent** (optional): tables, definitions, methodology—only if the audience needs audit trail.

4. **Analytical discipline** — Separate **fact** (from supplied data), **interpretation** (your reading), and **recommendation** (proposed action). Flag anomalies and data quality issues. When severity is assigned, state the rule (e.g., negative growth, missed target, bottom-quartile segment). Never fabricate benchmarks or statistics.

5. **Recommendation shape** — Each strategy names a clear program, links to a specific weak area, and ends with measurable success criteria. Cut generic advice (“improve marketing”) unless rewritten as concrete actions with owner-type hints when the user named teams.

6. **Design intent** — Document tone, hierarchy, density, palette for the engine:
   - **Tone**: confident, neutral-analytic; suitable for skimming in a meeting. No hype, no apology essays.
   - **Hierarchy**: executive summary and KPI strip highest contrast; findings use severity as visual rank (critical before minor); recommendations as scannable cards or numbered blocks.
   - **Density**: bullets and short paragraphs; one idea per block; tables for metric grids; reserve narrative depth for summary and “so what” under each finding.
   - **Palette**: professional corporate (deep neutral text, one accent for deltas and severity); WCAG AA; charts limited to a coherent family (trend line, ranked bars, contribution)—no decorative dashboard clutter.

**Rules & constraints**

- **Decision thread**: every section answers “what should leadership do differently?”—cut interesting but non-actionable analysis.
- **Severity before volume**: fewer, ranked problems beat long undifferentiated lists.
- **Chart vs prose**: use chart intent when the insight is shape or comparison; use prose when causality or nuance needs words.
- **Honesty gate**: missing data → `NEEDS USER INPUT`; do not backfill with plausible numbers.
- **Audience fit**: weekly reports emphasize delta and actions; quarterly reports add trend context and initiative portfolios.

**Common failure modes**

- Data dump without verdict → add executive summary with explicit headline and ranked takeaways.
- Recommendations disconnected from findings → map each initiative to a finding ID.
- Vague weakness (“sales could be better”) → replace with metric, period, and threshold.
- Equal weight to every KPI → designate 4–7 primary metrics; demote the rest to appendix.
- Essay trends where a line chart tells the story → switch block to chart role + one interpretive sentence.
- Fabricated growth rates or benchmarks → remove or label as assumption pending user confirmation.
- Generic strategy boilerplate → rewrite as named program with actions and metrics.

**Quality checklist**

- [ ] Charter states audience, period, and primary question.
- [ ] Executive summary readable alone; takeaways match body findings.
- [ ] Each KPI has value, comparison, and one-line “so what.”
- [ ] Findings ordered by severity; impact stated for each.
- [ ] Every recommendation links to a finding and includes success metrics.
- [ ] Facts traceable to user inputs; gaps explicitly marked.
- [ ] Design intent specifies hierarchy for summary, KPIs, findings, and recommendations.

### Step 2. Use the content_and_design.md and k2f skill to generate a K2F file and export to an appropriate file format.

Follow **k2f** to instantiate sections and chart roles from the blueprint, apply design intent, verify, and export the executive-ready deliverable.
