---
name: k2f-cover-letter
description: Turn a job posting, resume, and candidate notes into a tailored one-page cover letter with a sharp hook, requirement-matched proof, and company-specific fit—not a resume recap or generic application template. Use when the user asks for a cover letter, application letter, referral letter, or role-specific letter for job applications (including career change or entry-level). Create cover letters with AI-native design and the highest aesthetic standards, powered by the K2F format and engine. Purpose-built for AI agents, K2F lets them focus on semantic content and design intent while automatically handling layout, typography, and formatting complexity.
---

# K2F Cover Letter — AI-Native Aesthetic Design

## Purpose

Teach agents to blueprint **cover letters** in `content_and_design.md` before layout: what belongs in the letter versus the resume, how to sequence persuasion (hook → proof → fit → close), and aesthetic intent (tone, hierarchy, letter density)—without inventing employers, referrals, metrics, or company facts the user did not supply.

## Built on K2F

This skill produces cover letters as `.K2F` packages. Follow the **k2f** skill for authoring: unpack or init a workspace, edit the semantic tree, pack/verify, and export. K2F keeps agents on *meaning* (letter blocks, salutation, narrative)—not coordinates—while the engine handles layout and typography.

Before you build, install the core skill: `npx skills add suzheng/k2f --skill k2f`.

## When to Use

- Writing or rewriting a cover letter for a **specific role** and company (with or without a full job description).
- Referral introductions, internal transfers, speculative applications, or **career-change** narratives that need framing beyond the resume.
- Adjusting tone for startup vs enterprise or creative vs formal contexts—not resumes, CVs, thank-you notes, or negotiation emails.

## Workflow

### Step 1. Generate the content_and_design.md

Single blueprint for letter content and design intent. Use labeled blocks (letterhead, date/recipient, opening, body paragraphs, closing, sign-off) the **k2f** skill can map later.

**Sub-steps**

1. **Letter frame** — Role title, company, application type (standard, referral, career change, entry-level, return-to-work), target length (**250–400 words**, **one page**, typically **3–4 body paragraphs**), and tone band (formal, professional-warm, conversational-confident). Note if a job description and resume are attached.

2. **Requirement map** — From the JD (or user brief), list **top 3–5 must-haves**. Mark **strengths to prove** (with user-supplied metrics) and **gaps** to address honestly or omit. Pick **one primary match** for body paragraph 1 and **one secondary theme** (extra value, gap reframe, or company fit) for body paragraph 2.

3. **Company research layer** — Capture **one specific, verifiable hook** the user provides or confirms: product, launch, mission line, values, or challenge from the posting. If unknown, flag `NEEDS USER INPUT`—do not invent news or culture claims.

4. **Include / cut** — **Include**: hook, role name, two proof paragraphs (outcomes not on the resume), why-here/why-now, active close. **Cut**: resume summaries, salary, negativity, hobbies, duplicate CV bullets, generic work ethic, “To whom it may concern” when avoidable.

5. **Opening** — Pick one hook type in the blueprint: company insight, referral, JD problem-solver, achievement, or industry angle—not “I am writing to apply” or hollow self-praise.

6. **Body** — P1: **[their need] + [your experience] + [result]** from user data. P2: extra value, honest gap reframe, or deeper company tie-in. One quantified outcome when numbers exist; else `metric TBD`—never invent.

7. **Close & recipient** — Specific enthusiasm, discussion ask, brief thanks (not passive “hearing from you”). Name hiring manager if known; else `Dear Hiring Manager` or team salutation. Dates and addresses from user input only.

8. **Design intent** — **Tone**: employer-centric, confident, industry-matched (formal vs startup-direct). **Hierarchy**: hook and role scannable first; body readable; letterhead quiet. **Density**: one idea per short paragraph; body dominates the page. **Rhythm**: parallel proof sentences; one honorific and sign-off style throughout.

**Rules & constraints**

- **Truth gate**: employers, dates, referrals, metrics, and company facts trace to user input or explicit stubs.
- **Non-resume gate**: every paragraph must add **context, narrative, or motivation** not obvious from the CV.
- **Specificity gate**: company and role names appear correctly; no placeholder company left in final text.
- **Length gate**: over 400 words → cut weakest proof or merge fit into opening, not micro-type.
- **Gap gate**: acknowledge gaps with evidence and learning path—never fake credentials.
- **Employer-focus gate**: more sentences about their needs and your fit than about what you want from the job.

**Common failure modes**

- Generic opening or resume recap → hook + narrative the CV cannot carry.
- Missing or invented company detail → one verified beat or `NEEDS USER INPUT`.
- JD keyword stuffing, wrong names, passive close, apologetic career-change tone, or unconfirmed referral → fix against user sources before handoff.

**Quality checklist**

- [ ] Frame, requirement map (strengths/gaps), and hook type documented.
- [ ] Two body beats tie their needs to user evidence; metric or honest TBD.
- [ ] Verified why-here/why-now; active close; cut list for resume-only content.
- [ ] Design intent covers tone, hierarchy, density, sign-off for the full letter.

### Step 2. Use the content_and_design.md and k2f skill to generate a K2F file and export to an appropriate file format.

Follow **k2f** to instantiate letter blocks from the blueprint, apply design intent, verify, and export the application-ready letter (typically PDF or DOCX).
