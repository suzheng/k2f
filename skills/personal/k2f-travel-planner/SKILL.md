---
name: k2f-travel-planner
description: Blueprint a printable, trip-ready travel planner from destination, dates, budget, and must-dos—day-by-day flow, budget roll-up, packing, culture notes, and prep timeline with scannable hierarchy. Use when the user asks for a travel planner, trip itinerary document, vacation plan booklet, or multi-day travel guide to print or share. Create travel planners with AI-native design and the highest aesthetic standards, powered by the K2F format and engine. Purpose-built for AI agents, K2F lets them focus on semantic content and design intent while automatically handling layout, typography, and formatting complexity.
---

# K2F Travel Planner — AI-Native Aesthetic Design

## Purpose

Teach agents to author **travel planners** in `content_and_design.md`: trip content, information layering, and design intent so the result reads like a field guide—not a chat log or destination encyclopedia.

## Built on K2F

This skill produces travel planners as `.K2F` packages. Follow the **k2f** skill for authoring, pack/verify, and export. K2F keeps agents on trip semantics and visual rhythm while the engine handles layout and typography.

Before you build, install the core skill: `npx skills add suzheng/k2f --skill k2f`.

## When to Use

- Turning trip constraints into a **shareable planner** (couple, family, or group) with days, costs, and checklists in one document.
- Merging user notes, bookings, and research into one coherent structure.
- **Not** for destination-only chat or live expense tracking unless the deliverable is still a planner blueprint.

## Workflow

### Step 1. Generate the content_and_design.md

One blueprint: semantic blocks per planner section plus explicit design intent. Label sections so the **k2f** skill can map them later.

**Sub-steps**

1. **Trip frame** — Destination(s), dates, duration, travelers, purpose, budget (total or daily), currency, pace, accommodation base(s), must-sees. Note routing (linear, loop, hub-and-spoke) and transit or jet-lag recovery days.

2. **Input audit** — List confirmed bookings, dietary or accessibility needs, interests to weight, and gaps. Mark `NEEDS USER INPUT` for unknown flight times, hotel address, or ticket status; do not invent opening hours, prices, or visa rules—use `verify before travel` stubs.

3. **Section architecture (include / cut / merge)** — Default stack (reorder when the frame demands):
   - **Trip at a glance** (dates, party, budget headline, base neighborhoods) → **Essentials** (documents, emergency, money, connectivity, key transport) → **Day-by-day itinerary** → **Budget summary** → **Packing checklist** → **Culture & safety** (compact) → **Pre-trip countdown**.
   - **Cut** long destination essays, duplicate restaurant lists, generic packing unrelated to climate/activities, and hour-by-hour filler when pace is relaxed.
   - **Merge** overlapping transport tips into Essentials; **split** multi-city trips into labeled day ranges with transit days called out explicitly.

4. **Itinerary build** — **Theme per day**; geo-cluster stops; time blocks with duration, transit, meals, cost, booking notes. Pacing: short trips 2–3 majors/day; week-long 2 major + 1 minor plus one light day; buffers and an optional rain/tired swap every 2–3 active days.

5. **Budget reconciliation** — Fixed costs (flights, lodging, insurance, visas) plus daily buckets (food, local transport, activities, misc); 10% contingency; show total, per-person, and per-day; line items must trace to itinerary activities or stated assumptions.

6. **Supporting modules** — Packing grouped Documents / Clothing (climate + dress codes) / Activity gear; culture as short do/don’t and dining timing; prep timeline working backward from departure (bookings that sell out first at the top).

7. **Scan layers** — **Glance**: where, when, how much headroom. **Scan**: day titles and time leaders. **Detail**: addresses, confirmation hints, costs—subordinate in design intent.

8. **Design intent** — **Tone**: practical, warm, local-aware; imperative checklists for actions. **Hierarchy**: title and dates dominate; day + theme anchors; times and places beat prose; costs and must-book flags scannable. **Density**: airy overview; consistent day blocks; checkbox lists; culture as callouts. **Palette**: calm journal feel; one accent on day headers; strong contrast on times and bookings.

**Rules & constraints**

- **Truth gate**: prices, hours, and requirements trace to user input or labeled stubs—not guessed from memory.
- **Pace gate**: total planned hours fit the stated pace; include meals and transit in every day.
- **Budget gate**: category totals reconcile with headline budget or show explicit overrun note.
- **Routing gate**: minimize backtracking; high-energy days not stacked after red-eye arrival without recovery.
- **Planner boundary**: if the user only wants a spreadsheet tracker, note in blueprint that this skill targets a designed planner artifact.

**Common failure modes**

- Attraction list without time or geography → themed days with clustered blocks.
- Packed schedule with no rest → pacing table and one light day per week-long trip.
- Budget section disconnected from days → tie line items to named activities.
- Packing encyclopedia → climate + activity-filtered checklist only.
- Culture section as generic country essay → 5–8 actionable do/don’t plus dining timing.
- Prep timeline after departure or missing sell-out bookings → reverse chronological from departure date.
- Repeating the same practical tip in every day → lift once to Essentials.

**Quality checklist**

- [ ] Frame: destination, dates, party, budget, pace, routing, and must-sees documented.
- [ ] Section stack fits trip length; cuts and multi-city splits explained.
- [ ] Each day has theme, geo-logical flow, meals, transport, costs, and booking notes or stubs.
- [ ] Budget reconciles (or flags gap) with contingency stated.
- [ ] Packing and prep match climate, activities, and departure date.
- [ ] Glance layer answers where/when/how much without reading full days.
- [ ] Design intent covers tone, hierarchy, density, checklist rhythm, and accent.

### Step 2. Use the content_and_design.md and k2f skill to generate a K2F file and export to an appropriate file format.

Follow **k2f** to instantiate planner sections from the blueprint, apply roles from design intent, verify, and export (typically PDF or print-ready format).
