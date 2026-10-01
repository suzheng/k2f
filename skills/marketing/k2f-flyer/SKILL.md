---
name: k2f-flyer
description: Turn event details, offers, or brand notes into a single-page flyer with a scannable headline, one primary call to action, and mood-matched visual intent—not a brochure, slide deck, or poster wall. Use when the user asks for a flyer, leaflet, handout, promo sheet, open-house sheet, or print/digital one-pager for events, sales, restaurants, fitness, real estate, hiring, or community campaigns. Create flyers with AI-native design and the highest aesthetic standards, powered by the K2F format and engine. Purpose-built for AI agents, K2F lets them focus on semantic content and design intent while automatically handling layout, typography, and formatting complexity.
---

# K2F Flyer — AI-Native Aesthetic Design

## Purpose

Teach agents to author `content_and_design.md` for **single-page promotion**: what belongs on a flyer, how to prioritize for a three-second stop, and how to express aesthetic intent (tone, hierarchy, density)—not how to build or export K2F packages.

## Built on K2F

This skill produces flyers as `.K2F` packages. Follow the **k2f** skill for authoring: unpack or init a workspace, edit the semantic tree, pack/verify, and export. K2F keeps agents on *meaning* (headline, offer, logistics, CTA)—not pixel coordinates—while the engine compiles layout, typography, and theme into consistent output across PDF and other formats.

Before you build, install the core skill: `npx skills add suzheng/k2f --skill k2f`.

## When to Use

- Events, concerts, fundraisers, grand openings, seasonal sales, gym drives, restaurant specials, property open houses, job fairs, or service promos.
- Print handouts (letter, A4, A5) or square/digital variants when the user names a channel.
- When the user wants **one decisive page**, not multi-page brochures, full menus, or large-format posters.

## Workflow

### Step 1. Generate the content_and_design.md

One blueprint pairing **semantic content** with **design intent**. Use labeled blocks the k2f skill can map to a single canvas later.

**Sub-steps**

1. **Flyer lock** — Purpose (what is being promoted), distribution (print vs digital vs both), approximate format if known, audience situation, and brand constraints (colors, logo, voice). State the **one job** of the flyer: stop the scroll or earn a second glance.

2. **Message stack** — Lock a single primary message (event name, offer, or hook). Everything else supports or routes to action. Split supporting facts into layers:
   - **Glance (1–3 s)**: headline or offer line, hero visual direction, optional logo.
   - **Scan (5–10 s)**: when/where (or price/deadline), 2–4 proof bullets or lineup items—not paragraphs.
   - **Act**: one primary CTA (RSVP, Visit, Call, Scan QR, Register) plus minimal contact (one URL, phone, or address block).

3. **Content tiers** — Classify every user-supplied fact as **Hero**, **Support**, or **Footer**. Hero carries the story; Support is icons, bullets, or short labels; Footer is legal, sponsors, fine print. Move anything that needs a second page to a separate asset or drop it.

4. **Genre fit** — Align emphasis to flyer type without generic templates:
   - **Event/nightlife**: energy, date stack, talent or agenda strip, venue.
   - **Retail/sale**: discount or urgency, dates, store identity—avoid catalog lists.
   - **Local business/services**: trust + offer, 3 services max, contact band.
   - **Real estate**: property hook, open-house banner, beds/baths/sqft as scan labels, agent block.
   - **Food**: appetite cue, one special, location/hours—not full menu.
   - **Fitness/recruitment/community**: motivational or professional tone, one tiered list, QR/sign-up cue.

5. **Cut pass** — Remove mission essays, duplicate contact lines, every secondary CTA, and jargon the audience will not read at arm’s length. Replace weak headlines (“Join us for…”) with specific tension, benefit, or number.

6. **Design intent** — Document tone, hierarchy, density, and palette for the engine (no layout coordinates):
   - **Tone**: match occasion (luxurious, urgent, rustic, corporate-warm, festive)—one adjective pair, not a mood board essay.
   - **Hierarchy**: headline largest story; date/offer/price as second band; body bullets tertiary; contact/CTA visually anchored (often bottom or side).
   - **Density**: flyers breathe—one focal image or graphic field, generous margins; cap body copy (~80–180 words unless user demands more).
   - **Contrast & brand**: readable text on imagery (describe overlay or panel intent); 2–3 colors plus neutrals; consistent type roles (display vs detail).
   - **Channel note**: print favors high contrast and bleed-safe margins; digital favors bold type at smaller physical size—state which assumptions apply.

**Rules & constraints**

- **One page, one angle**: one event, one offer, or one listing; split campaigns into separate flyers.
- **CTA singularity**: one primary action; secondary links live in a single contact cluster.
- **Logistics legibility**: date, time, place (or equivalent) must survive a quick scan—never buried in prose.
- **Honesty gate**: no invented prices, dates, performers, or addresses; mark placeholders clearly.
- **Audience-first copy**: benefits and specifics over brand history; verbs on CTAs (“Register”, “Order”, “Tour”).

**Common failure modes**

| Failure | Fix |
|--------|-----|
| Brochure creep | Enforce Hero/Support/Footer; drop sections or link out. |
| Weak headline | Lead with offer, date, or outcome; subtitle carries brand. |
| Wall of text | Convert to 3–5 bullets or labeled chips; one testimonial max. |
| Competing CTAs | Pick primary; demote others to footer text only. |
| Generic stock mood | Tie palette and imagery to venue, cuisine, or neighborhood cues from the user. |
| Missing when/where | Add a dedicated scan block even if design is image-heavy. |
| Slide-deck paste | Replace slide titles with flyer headline + one visual concept. |

**Quality checklist**

- [ ] Flyer lock: purpose, audience, channel, and single job stated.
- [ ] One primary message and one primary CTA; logistics in scan layer.
- [ ] Hero/Support/Footer assignment; word budget noted.
- [ ] Genre-appropriate emphasis (event, sale, food, property, etc.).
- [ ] Design intent covers tone, hierarchy, density, contrast, and channel.
- [ ] No fabricated facts; placeholders labeled.
- [ ] All claims traceable to user-supplied material.

### Step 2. Use the content_and_design.md and k2f skill to generate a K2F file and export to an appropriate file format.

Follow **k2f** to instantiate the single-page flyer from the blueprint, apply roles from design intent, verify, and export (typically PDF for print or sharing).
