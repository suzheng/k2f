---
name: k2f-event-program
description: Turn confirmed schedules, social slate decisions, and sponsor lists into attendee-facing programs with time-first wayfinding and publish-safe social copy—not posters, invitations, or slide decks. Use when the user asks for a conference program, festival booklet, gala run-of-show, performance playbill, or multi-day badgeholder agenda. Create event programs with AI-native design and the highest aesthetic standards, powered by the K2F format and engine. Purpose-built for AI agents, K2F lets them focus on semantic content and design intent while automatically handling layout, typography, and formatting complexity.
---

# K2F Event Program — AI-Native Aesthetic Design

## Purpose

Teach agents to author `content_and_design.md` for **published attendee programs**: content selection, wayfinding layers, and aesthetic intent (register, hierarchy, density, tracks)—not K2F packaging.

## Built on K2F

This skill produces event programs as `.K2F` packages. Follow the **k2f** skill for authoring: unpack or init a workspace, edit the semantic tree, pack/verify, and export. K2F keeps agents on *meaning* (days, slots, sessions, social occasions, sponsors)—not pixel coordinates—while the engine compiles layout, typography, and theme into consistent output.

Before you build, install the core skill: `npx skills add suzheng/k2f --skill k2f`.

## When to Use

- Conferences, festivals, galas, ceremonies, or performances where guests **carry or scroll a schedule** live.
- Print booklets, folds, or digital PDFs—not posters, invitations, VIP playbooks, or slide exports.
- When times, rooms, and **official** social occasions are decided or marked draft.

## Workflow

### Step 1. Generate the content_and_design.md

One blueprint pairing **publishable facts** with **design intent**. Document the programme; do not re-decide hospitality, VIP qualification, or grid placement—only what badgeholders should see.

**Sub-steps**

1. **Program lock** — Identity, dates, venue, audience, channel (print fold, booklet, digital), and **guest job**: find the next room in seconds. Note draft vs final; label gaps.

2. **Publish boundary** — Classify user inputs:
   - **Main grid**: keynotes, sessions, breaks, meals—clock spine only.
   - **Official social slate**: occasions the event runs; each row needs slot, place, **access line** (open, ticketed, family-inclusive, alcohol-free—in user terms).
   - **Invitation-only**: only if approved for publication—“invitation only,” no lists, door rules, or rank markers.
   - **Third-party events**: off-program or “permitted,” not on the official slate unless the user says so.

3. **Attendee layers** — Four tiers:
   - **Glance**: cover plus day opener—title, dates, venue, one-line day arc.
   - **Scan**: primary grid—time, room/track, title, speaker names (not bios).
   - **Read**: optional blurbs, panel order, thanks—user-supplied only.
   - **Reference**: maps, index, one-line bios, conduct or access pointer, sponsor tiers—back matter.

4. **Chronology spine** — Time leads each day; parallel tracks share clocks with declared **track keys**. Breaks, meals, and social rows are grid lines, not prose. If two official social options share an hour, name **which clears inclusion commitments** beside the slate.

5. **Social placement** — Main grid stays authoritative. Social rows use named slots; avoid pairing a public reception with an invite-only room in overlapping hours unless the user documents that split. No separate published room → fold into the general social line.

6. **Cut pass** — Drop mission essays, duplicate timetables, full CVs, slide URLs, backstage detail, and unordered sponsor logos. One grid per day plus optional opener.

7. **Design intent** (no coordinates):
   - **Register**: conference, community meetup, gala, or playbill—match stated culture without inventing tiers.
   - **Hierarchy**: time column or band first; title second; speakers third; room/track as cue; sponsors recede.
   - **Density**: arm’s-length scannable spacing; clear day dividers; no flyer walls.
   - **Tracks**: naming plus column or color-key intent for concurrent sessions.
   - **Discretion**: minimal invitation-only lines; no wristband legends or door lists.
   - **Palette**: neutral wayfinding base plus one accent (tracks or “now”); gala or performance contrast only from user cues.

**Rules & constraints**

- Times and rooms match the user grid; flag conflicts, do not silently fix.
- Official slate only; satellites labeled separately.
- Concurrent social options: identify which clears the user’s access set or narrow the slate.
- Record non-overlap requirements as facts; do not reschedule here.
- No invented speakers, rooms, sponsors, or invitation lists—mark placeholders.

**Common failure modes**

| Failure | Fix |
|--------|-----|
| Flyer density | Glance/scan/read/reference; marketing on cover only. |
| Slide export | Grid rows, not slide titles; short blurbs optional. |
| Hidden tracks | Track keys; aligned clocks across columns. |
| Tier leak | Invitation-only without lists or badge legends. |
| Fake choice slate | Label access-clearing option per hour or drop slot. |
| Evening clash | Slots so public and invite-only do not read as tier vs consolation. |

**Quality checklist**

- [ ] Program lock and draft/final noted.
- [ ] Publish boundary classified (grid, slate, invitation-only, third-party).
- [ ] Layers and chronology spine per day; track keys if needed.
- [ ] Inclusion label for concurrent social hours when applicable.
- [ ] Cut pass; design intent complete; non-overlap recorded; no fabricated facts.

### Step 2. Use the content_and_design.md and k2f skill to generate a K2F file and export to an appropriate file format.

Follow **k2f** to instantiate the program from the blueprint, map roles from design intent, verify, and export (typically PDF).
