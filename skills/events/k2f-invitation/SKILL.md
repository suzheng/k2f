---
name: k2f-invitation
description: Turn event facts and tone notes into print or digital invitations with ceremonial copy, layered logistics, and suite-consistent visual intent—not posters, programs, or slide decks. Use when the user asks for a wedding invitation, party invite, gala card, save-the-date, RSVP insert, or mobile e-invite. Create invitations with AI-native design and the highest aesthetic standards, powered by the K2F format and engine. Purpose-built for AI agents, K2F lets them focus on semantic content and design intent while automatically handling layout, typography, and formatting complexity.
---

# K2F Invitation — AI-Native Aesthetic Design

## Purpose

Teach agents to author `content_and_design.md` for **ceremonial event invitations**: what to include, how to tier information for guests, and how to express aesthetic intent (formality, hierarchy, density, motif)—not K2F packaging.

## Built on K2F

This skill produces invitations as `.K2F` packages. Follow the **k2f** skill for authoring: unpack or init a workspace, edit the semantic tree, pack/verify, and export. K2F keeps agents on *meaning* (hosts, occasion, schedule, RSVP)—not pixel coordinates—while the engine compiles layout, typography, and theme into consistent output across PDF and other formats.

Before you build, install the core skill: `npx skills add suzheng/k2f --skill k2f`.

## When to Use

- Weddings, engagements, anniversaries, galas, dinners, baptisms, cultural celebrations, or corporate hosted events.
- Print cards (e.g. 5×7 portrait, 4×6 save-the-date) or digital shares (square, 9:16 story).
- Main invite plus matching suite pieces when named (RSVP, details, program cover)—each gets a blueprint or shared **suite style** block.

## Workflow

### Step 1. Generate the content_and_design.md

One blueprint pairing **guest-facing copy** with **design intent**. Declare **piece type** and **formality** so tone and date phrasing stay consistent.

**Sub-steps**

1. **Invitation lock** — Occasion, hosts or honorees, audience relationship (family, colleagues, public), channel (print, email, social), piece type (invite, save-the-date, RSVP, details), and cultural or religious context. State the **guest takeaway**: why they are invited and what they must remember first.

2. **Message stack** — Split content into reading layers:
   - **Opening (ceremonial)**: host line or couple names, invitation phrasing (“request the pleasure of your company”), event title if not the names.
   - **Core (must-read)**: date, day of week, time, venue name and city (full street address on details card or footer—not crammed into the hero).
   - **Support**: dress code, reception note, gift preference, parking, multi-event schedule (Mehendi, Sangeet, etc.) as labeled blocks—not paragraphs.
   - **Action**: RSVP deadline, method (card, URL, phone), meal or guest-count fields for RSVP pieces only.

3. **Content tiers** — **Hero** (names or occasion + emotional hook), **Ceremony** (when/where stack), **Courtesy** (host parents, religious wording, bilingual lines), **Footer** (website, registry note, map cue). Drop backstory, long bios, and duplicate URLs. Save-the-dates omit full ceremony copy; RSVP cards omit decorative essays.

4. **Formality fit** — Match copy shape to style:
   - **Formal/traditional**: spelled-out dates and times, full legal names, third-person host lines.
   - **Modern/casual**: first names, numeric dates, conversational invite line.
   - **Multi-event/cultural**: ordered sub-events with date-time-venue triplets; note symbols, language, and color symbolism in design intent—not as unexplained decoration.

5. **Suite alignment** — If multiple pieces: one shared palette, motif, and type roles; per-piece content only what that piece must carry. Save-the-date leads with date/location tease; main invite completes the story; RSVP is field-focused.

6. **Cut pass** — Remove marketing slogans, full agendas, three fonts worth of quotes, and every extra CTA. One RSVP path; registry and website in footer cluster.

7. **Design intent** — Document for the engine (no coordinates):
   - **Tone**: one style family (classic elegant, modern minimal, floral romantic, rustic organic, destination relaxed, ornate cultural)—aligned to venue and dress code.
   - **Hierarchy**: display names or occasion largest; date/venue as second band; supporting lines smallest; never three competing display lines.
   - **Density**: invitations **breathe**—generous margins; cap to essential lines; motif frames content, not competes with it.
   - **Palette & motif**: 2–4 harmonious colors; botanical, geometric, cultural, or typographic motif—stated as intent, not stock-keyword lists.
   - **Typography roles**: at most three roles (e.g. script display, serif ceremony, sans detail).
   - **Channel**: print texture or foil *intent* vs digital legibility at phone scale (bold date band, high contrast).

**Rules & constraints**

- **One piece, one job**: do not merge RSVP fields onto a save-the-date unless the user asks.
- **When/where stack**: date, time, and place appear as a scannable block—never buried in flowing prose.
- **Honesty gate**: no invented venues, times, hosts, or traditions; label placeholders.
- **Cultural respect**: symbols and bilingual text only when user supplies or requests; do not invent ritual language.
- **Suite consistency**: repeating pieces share motif and palette notes in a **Suite style** subsection.

**Common failure modes**

| Failure | Fix |
|--------|-----|
| Poster/flyer density | Enforce Hero/Ceremony/Footer; move maps and schedules to details card. |
| Cluttered ornament | Motif at edges or watermark; center field for type. |
| Flat hierarchy | One display line for names; structured date band below. |
| Format clash | Formal spelled-out date with ultra-casual body—pick one lane. |
| Missing RSVP clarity | Dedicated action line with deadline and method. |
| Generic wedding template | Anchor palette and motif to user venue, season, or culture. |
| Digital unreadable | Promote date/venue; simplify script at small sizes in intent. |

**Quality checklist**

- [ ] Invitation lock: occasion, piece type, channel, formality, guest takeaway.
- [ ] Opening / core / support / action layers populated; tiers assigned.
- [ ] Formality-appropriate date, time, and name treatment.
- [ ] Design intent covers tone, hierarchy, density, palette, type roles, channel.
- [ ] One primary RSVP or attendance path; footer for secondary links.
- [ ] Suite style noted if multiple pieces; cut pass applied.
- [ ] No fabricated facts; placeholders labeled.

### Step 2. Use the content_and_design.md and k2f skill to generate a K2F file and export to an appropriate file format.

Follow **k2f** to instantiate the invitation from the blueprint, map roles from design intent, verify, and export (typically PDF for print or PNG for digital sharing).
