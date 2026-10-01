---
name: k2f-event-poster
description: Turn event briefs into wall- or screen-ready posters with headline zone, hero visual intent, and scannable logistics—not programs, slide decks, or handout flyers. Use when the user asks for an event poster, concert poster, festival promo, exhibition poster, meetup banner, or large-format promo. Create event posters with AI-native design and the highest aesthetic standards, powered by the K2F format and engine. Purpose-built for AI agents, K2F lets them focus on semantic content and design intent while automatically handling layout, typography, and formatting complexity.
---

# K2F Event Poster — AI-Native Aesthetic Design

## Purpose

Teach agents to author `content_and_design.md` for **large-format event promotion**: content selection, distance-viewing layers, and aesthetic intent (tone, hierarchy, density, composition zones)—not K2F packaging.

## Built on K2F

This skill produces event posters as `.K2F` packages. Follow the **k2f** skill for authoring: unpack or init a workspace, edit the semantic tree, pack/verify, and export. K2F keeps agents on *meaning* (event identity, lineup, logistics, mood)—not pixel coordinates—while the engine compiles layout, typography, and theme into consistent output across PDF and other formats.

Before you build, install the core skill: `npx skills add suzheng/k2f --skill k2f`.

## When to Use

- Concerts, festivals, conferences, exhibitions, galas, workshops, sports, or activations on wall, screen, or social hero.
- Vertical (11:17, 2:3, A-series) or square/horizontal when the user names a channel.
- **One iconic visual + decisive type**, not a dense flyer or slide export.

## Workflow

### Step 1. Generate the content_and_design.md

One blueprint pairing **semantic content** with **design intent**. Reserve explicit **type zones** and **hero field** so the engine can compose without crowding the focal image.

**Sub-steps**

1. **Poster lock** — Event type, audience, distribution (print vs digital vs both), approximate aspect ratio if known, and brand constraints. State the **one impression**: what should someone remember after walking past at 3 m?

2. **Message stack** — Lock a single primary identity (event name or campaign line). Split content into viewing layers:
   - **Distance (3–5 s)**: event title or headliner, dominant visual concept, optional mark/logo.
   - **Approach (5–15 s)**: date/time stack, venue or city, 1–3 supporting names (lineup, speakers, hosts)—short labels, not bios.
   - **Near (optional)**: ticket URL, QR cue, sponsor row, age/policy line—footer band only.

3. **Content tiers** — Classify facts as **Hero** (title + visual story), **Support** (date, venue, lineup strip, price tier), **Footer** (sponsors, legal, social handle). Drop agendas, paragraphs, and duplicate URLs; link-level detail belongs elsewhere.

4. **Genre fit** — Tune emphasis without generic templates:
   - **Music/nightlife**: energy, headliner hierarchy, date stack, venue; avoid set-list essays.
   - **Conference/keynote**: authoritative tone, speaker name + theme hook, date/location ribbon.
   - **Exhibition/gallery**: artist or show title, dates, institution; calm negative space.
   - **Sports/community**: motion cue, matchup or cause, date/venue block.
   - **Corporate launch**: product or theme line, single proof point, registration CTA.

5. **Composition zones** — Top third: title/headliners; middle: hero or motif; bottom: logistics and CTA. Note balance, safe margins, and overlay/panel intent on busy imagery.

6. **Cut pass** — Remove mission statements, full schedules, multiple CTAs, and weak titles (“Annual Event 2026”). Sharpen the hook with specificity (genre, benefit, or named talent the user provided).

7. **Design intent** — Document for the engine:
   - **Tone**: one mood pair (e.g. neon-rave, gallery-minimal, stadium-intense)—aligned to venue and audience.
   - **Hierarchy**: display title > date/venue band > support names > footer; never three competing headline sizes.
   - **Density**: posters are **image-forward**; cap body copy; let type breathe in reserved zones.
   - **Palette & light**: 2–3 colors plus neutrals; direction of contrast (dark cinematic, pastel retro, high-key corporate).
   - **Style cue**: era or art direction only when it serves the event (not decorative keyword soup).
   - **Channel**: print vs social crop assumptions stated briefly.

**Rules & constraints**

- **One event, one poster**: split series nights or tracks into separate posters unless user requests a campaign set with shared style notes.
- **Typography zones**: always declare where title and logistics live relative to the hero; do not scatter dates in prose.
- **CTA singularity**: one primary action (Tickets, Register, RSVP); extras in footer cluster.
- **Logistics legibility**: date, time, place (or stream equivalent) must survive a mid-distance glance.
- **Honesty gate**: no invented performers, prices, venues, or sponsors; label placeholders.
- **Distance-first**: unreadable in a quick phone preview → shorten or promote to headline.

**Common failure modes**

| Failure | Fix |
|--------|-----|
| Flyer density on a poster | Enforce Hero/Support/Footer; move lists to a link or handout. |
| Text over busy hero | Specify overlay, panel, or split-field intent in design block. |
| Flat hierarchy | One display title; headliners secondary; date/venue as structured band. |
| Generic festival look | Anchor palette and motif to genre, city, or venue cues from the user. |
| Missing when/where | Add dedicated scan band even when art is minimal. |
| Slide-deck paste | Replace slide titles with poster title + single visual concept. |
| Competing focal points | One hero subject; lineup as type strip, not second illustration. |

**Quality checklist**

- [ ] Poster lock: event type, audience, channel, aspect intent, and one impression stated.
- [ ] Distance / approach / near layers populated; Hero/Support/Footer assigned.
- [ ] Composition zones (top / hero / bottom) and balance described.
- [ ] Design intent covers tone, hierarchy, density, palette, and channel.
- [ ] One primary CTA; logistics in scan band.
- [ ] Genre-appropriate emphasis; cut pass applied.
- [ ] No fabricated facts; placeholders labeled.

### Step 2. Use the content_and_design.md and k2f skill to generate a K2F file and export to an appropriate file format.

Follow **k2f** to instantiate the poster from the blueprint, map roles from design intent, verify, and export (typically PDF for print or PNG/PDF for digital).
