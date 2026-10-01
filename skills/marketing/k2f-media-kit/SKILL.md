---
name: k2f-media-kit
description: Turn verified company, product, and leadership inputs into a journalist-ready media kit with scannable facts, boilerplate, and asset guidance—not a pitch deck or sales one-pager. Use when the user asks for a media kit, press kit, PR kit, or creator resource pack. Create media kits with AI-native design and the highest aesthetic standards, powered by the K2F format and engine. Purpose-built for AI agents, K2F lets them focus on semantic content and design intent while automatically handling layout, typography, and formatting complexity.
---

# K2F Media Kit — AI-Native Aesthetic Design

## Purpose

Teach agents to author `content_and_design.md` for **press- and partner-facing media kits**: what belongs in the kit, how to tier information for grab-and-quote use, and aesthetic intent (tone, hierarchy, density)—not K2F packaging or export.

## Built on K2F

This skill produces media kits as `.K2F` packages. Follow the **k2f** skill for authoring: unpack or init a workspace, edit the semantic tree, pack/verify, and export. K2F keeps agents on *meaning* (sections, boilerplate, fact blocks, visual intent)—not coordinates—while the engine compiles layout, typography, and theme into export-ready output.

Before you build, install the core skill: `npx skills add suzheng/k2f --skill k2f`.

## When to Use

- Launch or refresh materials for **press, podcast hosts, analysts, or creators** who need facts and approved copy fast.
- Packaging boilerplate, leadership bios, product snapshots, milestones, and **asset inventory** in one coherent kit.
- **Modular reference** journalists can excerpt—not fundraising decks (**k2f-pitch-deck**), event promos (**k2f-flyer**), or single-page internal briefs (**k2f-one-pager**).

**Not for:** investor narratives or exhaustive brand bibles unless the user wants a hybrid (usage intent only).

## Workflow

### Step 1. Generate the content_and_design.md

Blueprint **semantic content** and **design intent** in labeled blocks reporters can scan and excerpt independently.

**Sub-steps**

1. **Kit lock** — Primary user (beat reporter, creator, partner comms), **kit job** (launch coverage, ongoing press room, product drop, executive profile), channel (downloadable PDF, emailed ZIP companion, on-site sections), and legal posture (what may be named vs anonymized).

2. **Kit archetype & section order** — Pick one skeleton; do not merge incompatible kits:
   - **Company / ongoing press**: cover identity → quick facts → short + long boilerplate → key messages (3–5) → products/services snapshot → milestones → leadership (short bios) → customers/partners (permitted only) → news highlights → awards → asset manifest → media contact.
   - **Product launch**: product hook → problem context (factual) → what’s new → availability/pricing (public only) → specs at outcome level → launch quotes (user-supplied) → boilerplate → assets → contact.
   - **Founder / executive**: headline role → bio (neutral third person) → expertise themes → select quotes → company tie-in → headshot intent → contact via comms.
   - **Creator pack**: brand one-liner → do/don’t talking points → deliverable expectations → disclosure rules → links/handles → asset list → contact.

3. **Information layers** — **Glance**: name, descriptor, founded, HQ, URL, category. **Grab-and-quote**: 50- and 150-word boilerplate, key messages, approved quotes. **Reference**: fact table, timelines, bios, product cards. **Operational**: asset manifest, usage notes, media email—never buried under marketing prose.

4. **Content tiers** — **Approved copy** (paste-ready), **Verified facts** (sourced or TBD), **Pointer** (link to press page, DAM, or “request via…”). Park website essays, roadmap fiction, and internal metrics in **omit** or appendix pointer—not inline.

5. **Honesty & permissions** — No invented revenue, customers, logos, awards, or quotes. Missing inputs → `TBD`, `Not provided`, or `NEEDS USER INPUT`. Logo/customer blocks only when user confirmed; else category-only or omit.

6. **Cut pass** — Remove pitch-deck slides as paragraphs, duplicate bios, feature catalogs, and multiple CTAs. One **primary media contact**; secondary routes in a single line. Replace adjectives with one verifiable fact per message.

7. **Design intent** (no coordinates):
   - **Tone**: editorial-neutral, third-person ready; confident only where proof exists—state one voice pair (e.g., “factual, approachable”).
   - **Hierarchy**: kit title and descriptor dominate; boilerplate and fact tables as distinct bands; bios and news as repeatable modules.
   - **Density**: facts in tables or labeled rows; bios ~80–120 words; key messages as parallel bullets; avoid walls of prose.
   - **Rhythm**: alternate **copy blocks** with **fact strips** and **visual intent** (hero product frame, headshot grid, logo row)—modules should feel extractable.
   - **Contrast**: calm ground; accent on labels and pull quotes; legible roles for boilerplate.

**Rules & constraints**

- Every section must be **excerpt-friendly**—no “see above” dependencies across modules.
- Boilerplate: two lengths (short/long) with identical facts; spelling of legal name and product names locked.
- Quotes: attributed, dated if possible, user-supplied only.
- Asset manifest: list type, description, and availability (attached / URL / on request)—no fake download links.
- Launch kits: separate **news angle** (what changed) from evergreen company boilerplate.

**Common failure modes**

| Failure | Fix |
|--------|-----|
| Pitch deck pasted in | Map to archetype; move ask/financials out; keep facts + boilerplate. |
| Marketing fluff, no paste-ready copy | Add short/long boilerplate and key messages first. |
| Missing fact layer | Quick-facts table before narrative. |
| Fabricated logos/quotes/stats | Stub, anonymize, or NEEDS USER INPUT. |
| One giant bio essay | Short bio + optional extended in subordinate block. |
| Asset section vague | Manifest rows: asset, format, use, source. |
| Sales CTA overload | Single media contact + optional press page URL. |

**Quality checklist**

- [ ] Lock: audience, kit job, archetype, channel, permissions.
- [ ] Section order matches archetype; modules stand alone.
- [ ] Glance / grab-and-quote / reference / operational layers present.
- [ ] Short + long boilerplate consistent; key messages parallel.
- [ ] Facts traceable or TBD; quotes and logos permitted only.
- [ ] Asset manifest complete or explicitly incomplete.
- [ ] Design intent: tone, hierarchy, density, rhythm, contrast.
- [ ] One primary media contact; no placeholder URLs.

### Step 2. Use the content_and_design.md and k2f skill to generate a K2F file and export to an appropriate file format.

Follow the **k2f** skill to build the `.K2F` package from the blueprint, verify, and export (typically PDF).
