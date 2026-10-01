---
name: k2f-conference-handout
description: Turn an accepted talk, slide outline, or session notes into a printable attendee handout with verb-led takeaways, a condensed talk spine, and follow-up resources—not a slide dump or speaker script. Use when the user asks for a conference handout, session handout, talk companion, or printable PDF for attendees after a presentation. Create conference handouts with AI-native design and the highest aesthetic standards, powered by the K2F format and engine. Purpose-built for AI agents, K2F lets them focus on semantic content and design intent while automatically handling layout, typography, and formatting complexity.
---

# K2F Conference Handout — AI-Native Aesthetic Design

## Purpose

Teach agents to author `content_and_design.md` for **attendee reference**: what to keep from the talk, how to layer it for reading on paper or PDF, and how to express design intent (tone, hierarchy, density)—not how to build or export K2F packages.

## Built on K2F

This skill produces conference handouts as `.K2F` packages. Follow the **k2f** skill for authoring: unpack or init a workspace, edit the semantic tree, pack/verify, and export. K2F keeps agents on *meaning* (sections, takeaways, figures)—not pixel coordinates—while the engine compiles layout, typography, and theme into consistent output across PDF and other formats.

Before you build, install the core skill: `npx skills add suzheng/k2f --skill k2f`.

## When to Use

- After a talk is outlined or accepted; before or after slides exist.
- When the user wants something **attendees keep**, not a CFP, not full slide export, not presenter notes.
- Print or PDF distribution at workshops, meetups, or conference sessions.

## Workflow

### Step 1. Generate the content_and_design.md

One blueprint pairing **semantic content** with **aesthetic intent**. Use labeled blocks the k2f skill can map to pages later.

**Sub-steps**

1. **Session lock** — Event, session title, slot length, audience job and situation (not “developers”), vendor-neutral vs vendor-run tone, and whether product names are allowed. State what the handout must do: follow-up reference, not marketing.

2. **Source spine** — List the talk in running order with minute estimates (two levels). Mark each beat **Keep**, **Compress**, or **Drop**. Handouts mirror the **arc**, not the slide count.

3. **Attendee layers** — Structure content in four tiers:
   - **Glance (10 s)**: title, speaker, event/date, optional QR or URL.
   - **Scan (1 min)**: 2–4 **verb-first takeaways** scoped to slot length (lightning: one idea, one takeaway; standard: one problem, two or three actions).
   - **Read (5–10 min)**: condensed sections aligned to the spine—hooks, diagrams, tables, code the room could not absorb live.
   - **Later**: links, repos, reading, prerequisites, contact—short, sourced from the user.

4. **Progressive beats** — Where the live talk used reveals or demos, specify the **static equivalent** in the design doc (numbered steps, before/after pair, screenshot description, final code state). Never leave a section that only made sense animated.

5. **Cut pass** — Remove pitch-first openings, book-sized scope, speaker-journey framing, and “understand/appreciate” bullets. Replace with attendee-side sentences and actions they can perform Monday.

6. **Design intent** — Document tone, hierarchy, density, and palette for the engine (no JSON):
   - **Tone**: attendee-facing; problem → approach → payoff; evidence and numbers the speaker can source.
   - **Hierarchy**: takeaways and section titles carry the story; body supports; footnotes and links recede.
   - **Density**: tighter than slides, looser than a paper—bullets and callouts over long paragraphs; cap ~800–1,200 words unless the user sets a limit.
   - **Palette**: calm professional base, one accent for takeaways and warnings; high contrast for print; code and figures called out as full-width or inline.

**Rules & constraints**

- **One angle per handout**: one problem and one through-line; split multi-topic talks into separate handouts.
- **Takeaway test**: every takeaway starts with an action verb and fits the slot; eight takeaways in twenty-five minutes fails the test.
- **Attendee voice**: rewrite “I’ll show our journey” into what the reader gains.
- **Honesty gate**: no invented benchmarks, customers, or citations; mark gaps the talk skipped.
- **Static-only assets**: video, iframe, or live demo → screenshot, steps, or repo link in the blueprint.

**Common failure modes**

| Failure | Fix |
|--------|-----|
| Slide dump | Keep spine labels; drop title-only slides; expand only high-value beats. |
| Presenter notes pasted | Separate optional “facilitator” material; default doc is attendee-only. |
| Missing reveal content | Add final-state diagram, code, or numbered list for each progressive beat. |
| Pitch in disguise | Lead with the problem; product is the setting, not the headline. |
| Interchangeable title | Add system, scale, or tension from the talk. |
| Wall of code | One exemplar snippet plus link; annotate lines that matter. |
| Generic takeaways | Verb + object + context (“Audit egress before enabling cross-region replication”). |

**Quality checklist**

- [ ] Session lock and audience are explicit; tone matches event type.
- [ ] Spine mapped with Keep/Compress/Drop; one clear angle.
- [ ] 2–4 verb-first takeaways match slot length.
- [ ] Every live-only beat has a static substitute described.
- [ ] No pitch-first or speaker-centric openings; at least one sourced number or concrete system.
- [ ] Word budget stated; links and prerequisites in the Later layer.
- [ ] Design intent covers tone, hierarchy, density, and palette.
- [ ] All claims traceable to user-supplied material.

### Step 2. Use the content_and_design.md and k2f skill to generate a K2F file and export to an appropriate file format.

Follow **k2f** to instantiate pages from the blueprint, apply roles from design intent, verify, and export the attendee-ready handout (typically PDF for print or sharing).
