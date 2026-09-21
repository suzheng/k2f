# K2F agent skill

The core **K2F** agent skill (workflows, catalog, scripts, schemas) lives in the **[k2f-skills](https://github.com/suzheng/k2f-skills)** repository:

[`skills/core/k2f/`](https://github.com/suzheng/k2f-skills/tree/main/skills/core/k2f)

## Install

```bash
npx skills add suzheng/k2f-skills --skill k2f
pip install k2f
```

Works with Cursor, Claude Code, Codex, and other agents that support [npx skills](https://github.com/vercel-labs/skills).

**Manual fallback:** copy the [`skills/core/k2f`](https://github.com/suzheng/k2f-skills/tree/main/skills/core/k2f) folder into your agent skills path as `k2f/` (for example `~/.cursor/skills/k2f/`).

Entry point: [SKILL.md](https://github.com/suzheng/k2f-skills/blob/main/skills/core/k2f/SKILL.md) on GitHub · [k2f.dev/skills/k2f](https://k2f.dev/skills/k2f) on the site.

Task-oriented skills (invoice, poster, CV, …) are added under the same [k2f-skills](https://github.com/suzheng/k2f-skills) repo.
