---
name: k2f-resume
description: Turn career history, notes, or an existing CV into a tailored, achievement-led resume with clear hierarchy and scannable proof—not a generic duty list or keyword-stuffed template. Use when the user asks for a resume, CV, job application document, or role-targeted career summary (including refresh for a specific job posting). Create resumes with AI-native design and the highest aesthetic standards, powered by the K2F format and engine. Purpose-built for AI agents, K2F lets them focus on semantic content and design intent while automatically handling layout, typography, and formatting complexity.
---

# K2F Resume — AI-Native Aesthetic Design

## Purpose

Teach agents to blueprint **resumes** in `content_and_design.md` before layout: what to include and cut, how to tier information for recruiter scan, achievement bullets with honest metrics, and aesthetic intent (tone, hierarchy, density)—without inventing roles, dates, or results the user did not supply.

## Built on K2F

This skill produces resumes as `.K2F` packages. Follow the **k2f** skill for authoring: unpack or init a workspace, edit the semantic tree, pack/verify, and export. K2F keeps agents on *meaning* (sections, roles, claims)—not coordinates—while the engine handles layout and typography.

Before you build, install the core skill: `npx skills add suzheng/k2f --skill k2f`.

## When to Use

- Drafting or rebuilding a resume from notes, LinkedIn export, or an outdated CV.
- Tailoring content to a **target role** or job description (emphasis and keywords, not fabrication).
- Career changers, recent grads, or executives who need different section weighting—not cover letters, portfolios, or multi-page bios.

## Workflow

### Step 1. Generate the content_and_design.md

Single blueprint for semantic content and design intent. Use labeled blocks per section (header, summary, each role, education, skills) that the **k2f** skill can map later.

**Sub-steps**

1. **Resume frame** — Target role, seniority, industry, geography, and page budget (typically **one page** until ~10+ years or heavy publication lists; two pages max when justified). Note if user supplied a job description for tailoring.

2. **Input audit** — Inventory dates, employers, titles, projects, metrics, education, certifications, and links. Flag `NEEDS USER INPUT` for gaps; never fabricate revenue, headcount, or tenure.

3. **Section architecture (include / cut)** — Default stack:
   - **Contact** → **Professional summary** (optional) → **Experience** (reverse chronological) → **Education** → **Skills** → (optional) **Certifications / Projects**.
   - Use **recognizable section labels** (e.g., Work Experience, Education, Skills)—not clever nicknames that obscure structure.
   - **Cut** roles older than ~10–15 years unless highly relevant; redundant soft-skill essays; hobbies; full street address; every duty from early careers.
   - **Merge or shorten** thin roles to one line; **expand** only recent roles aligned with the target job (typically **2–4 bullets** each).

4. **Scan layers** — **Glance** (top third): name, role headline or target title, contact line, summary hook. **Scan**: employer + title + dates, then bullet achievements. **Detail**: tools, coursework, certifications—subordinate, grouped, not competing with impact bullets.

5. **Achievement bullets** — Outcomes, not duties. Use **X–Y–Z**: result + metric + how; one strong verb; **1–2 lines max**. Add numbers only from user input, conservative estimates they accept, or `metric TBD`—never invent.

6. **Tailoring pass** (when a JD exists) — Extract must-have skills and role phrases; place exact terms in summary, skills, and bullets only where truthful. Avoid keyword repetition or synonym lists in Skills.

7. **Design intent** — Tone, hierarchy, density for the engine:
   - **Tone**: confident, factual, achievement-led; no hype without proof. Match industry (creative may allow slightly more voice; finance/legal stay crisp).
   - **Hierarchy**: name and target headline dominate; job titles and employers next; dates and locations quieter; bullets as readable body; skills/education visually lighter than recent experience.
   - **Density**: generous whitespace mentally—no paragraph blocks under roles; **~5 bullets max** per recent job; skills as compact grouped lists, not walls of comma-separated keywords.
   - **Rhythm**: parallel bullets per role; consistent dates; one accent on name or rules, neutral body, readable contrast.

**Rules & constraints**

- **Truth gate**: every employer, title, date, and metric traces to user input or an explicit stub.
- **Achievement gate**: no “responsible for” or “helped with” as the lead; replace with owned outcomes or cut the bullet.
- **One-page gate**: overflow → cut oldest roles and weak bullets in the blueprint, not via tiny type.
- **Tailoring gate**: keywords appear because the user actually did the work—not because the JD asked for it.
- **Consistency gate**: verb tense (present for current role, past for others), date style, and company names identical everywhere.

**Common failure modes**

- Duty laundry list with no metrics → rewrite with X–Y–Z or ask user for scale/outcomes.
- Generic summary that could fit any candidate → tie to target role + 2–3 proof themes from experience.
- Skills section as keyword stuffing → group by category; mirror JD terms only where accurate.
- Equal weight on every job → emphasize last 2–3 roles; demote or omit ancient history.
- Creative section names or buried contact → standard labels; contact in the main flow, not an afterthought.
- Two pages of old tasks for a mid-level role → enforce page budget and cut pass.
- Mismatched dates or titles vs user source → reconcile before handoff.

**Quality checklist**

- [ ] Frame: target role, seniority, page budget, JD tailoring noted if applicable.
- [ ] Section order and labels are standard; cut list documented for omitted content.
- [ ] Glance layer delivers name, reachability, and positioning within top third.
- [ ] Each recent role has achievement bullets with verbs, outcomes, and metrics or honest TBD.
- [ ] Summary (if present) is 3–4 sentences and JD-aligned without stuffing.
- [ ] Skills grouped; critical JD terms appear in truthful locations.
- [ ] Design intent states tone, hierarchy, density, and accent usage for the full page.

### Step 2. Use the content_and_design.md and k2f skill to generate a K2F file and export to an appropriate file format.

Follow **k2f** to instantiate sections from the blueprint, apply roles from design intent, verify, and export the application-ready resume (typically PDF).
