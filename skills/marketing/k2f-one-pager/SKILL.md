---
name: k2f-one-pager
description: Turn notes, specs, or release details into a single-page brief that aligns stakeholders on one story—problem, solution, proof, scope, or changelog—not a deck, flyer, or multi-page PRD. Use when the user asks for a one-pager, product brief, executive summary page, solution overview, initiative brief, or printable one-page spec. Create one pagers with AI-native design and the highest aesthetic standards, powered by the K2F format and engine. Purpose-built for AI agents, K2F lets them focus on semantic content and design intent while automatically handling layout, typography, and formatting complexity.
---

# K2F One Pager — AI-Native Aesthetic Design

## Purpose

Teach agents to author `content_and_design.md` for **single-page decision and narrative documents**: what belongs on one page, how to tier information for executive scan, and aesthetic intent (tone, hierarchy, density)—not K2F packaging or export.

## Built on K2F

This skill produces one pagers as `.K2F` packages. Follow the **k2f** skill for authoring: unpack or init a workspace, edit the semantic tree, pack/verify, and export. K2F keeps agents on *meaning* (sections, claims, evidence shapes, CTA)—not coordinates—while the engine compiles layout, typography, and theme into export-ready output.

Before you build, install the core skill: `npx skills add suzheng/k2f --skill k2f`.

## When to Use

- Product or feature briefs, initiative proposals, solution/company overviews, partnership summaries, or release summaries on **one page**.
- Alignment before detailed design, eng handoff, or launch comms.
- **One scannable artifact**—not pitch decks, flyers, posters, full PRDs, or multi-page reports.

**Not for:** deep technical design, research decks, or event promos centered on date/venue and one bold CTA (**k2f-flyer**).

## Workflow

### Step 1. Generate the content_and_design.md

Blueprint **semantic content** plus **design intent** in labeled section blocks for a single canvas.

**Sub-steps**

1. **One-pager lock** — The **one job** (approval, release summary, solution sell). Audience, channel (print PDF, email, wiki), and **genre**—pick one archetype; do not merge incompatible skeletons.

2. **Archetype & section order** — Fixed vertical stack with headings:
   - **Decision / product**: thesis → problem (pain + why now) → solution (what, not UI) → users → metrics (baseline + target) → in/out scope → constraints → open questions → optional owner/next step.
   - **Release / changelog**: version, date, summary → Added → Fixed → Breaking → Known issues → Upgrade note → CTA only with a real destination.
   - **Solution / company**: audience → one-line offer → three differentiators → proof → optional pricing → one next step.
   - **Initiative pitch**: problem → approach → impact → resources/timeline → ask → risks.

3. **Scan layers** — **Glance**: title, thesis, hero metric or version. **Scan**: 2–4 bullet blocks. **Detail**: sources and fine print—subordinate to thesis.

4. **Content tiers** — **Thesis** (page story), **Evidence** (numbers, quotes, lists), **Boundary** (scope, constraints, explicit “none”). Second-page material → cut or appendix link.

5. **Honesty** — No invented metrics, dates, customers, or breaking changes. Missing input → `None`, `No … provided`, or `TBD`; unknown version/date → `—` with label.

6. **Cut pass** — Drop essays, feature catalogs, APIs, duplicate metrics. Replace vague goals with measurable outcomes or TBD. One primary CTA; extras in a contact line.

7. **Design intent** (no coordinates):
   - **Tone**: evidence-led for decisions; factual for releases; confident solution voice only with proof—one adjective pair.
   - **Hierarchy**: thesis largest; consistent section labels; hero metric/version as anchor; boundaries smallest.
   - **Density**: ~250–450 words unless user overrides; bullets over prose; ~5 bullets max per block; short parallel columns for in/out scope only.
   - **Rhythm**: mix bullets with one visual intent (metric row, logo band, timeline chips)—avoid bullet walls.
   - **Contrast**: light ground, accent on thesis/metrics/CTA; readable brand colors.

**Rules & constraints**

- One page, one thesis—split combined angles into separate one-pagers.
- Changelog: every standard section present; empty → explicit None-style copy.
- Capabilities at outcome level, not screen specs.
- Metrics: baseline + target or TBD—no decoration.
- Crisp in/out scope; MVP vs later when relevant.
- CTA only with real destinations—no placeholders.

**Common failure modes**

| Failure | Fix |
|--------|-----|
| Deck/PRD paste | Collapse to archetype; one idea per bullet; park depth in out-of-scope. |
| Flyer tone on product brief | Problem + metrics before slogans. |
| Skipped release sections | Labeled blocks with None-style text. |
| Vague problem | Pain + signal or TBD. |
| Scope creep | Boundary tier + out-of-scope list. |
| Two theses | One job per page. |
| Fabricated proof | Stub or NEEDS USER INPUT. |

**Quality checklist**

- [ ] Lock: job, audience, genre, channel.
- [ ] Archetype order; thesis in glance layer.
- [ ] Thesis / Evidence / Boundary; word budget noted.
- [ ] Claims traceable or stubbed; scope/changelog sections complete.
- [ ] Design intent: tone, hierarchy, density, rhythm, contrast.
- [ ] One primary next step; no fake CTAs.

### Step 2. Use the content_and_design.md and k2f skill to generate a K2F file and export to an appropriate file format.

Follow **k2f** to build the single page from the blueprint, apply design roles, verify, and export (usually PDF).
