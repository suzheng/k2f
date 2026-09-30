# K2F agent skills

Task-oriented [Agent Skills](https://docs.anthropic.com/en/docs/agents-and-tools/agent-skills) for [K2F](https://github.com/suzheng/k2f) — document jobs users actually ask for (posters, invoices, CVs, fillable forms, and more), built on the K2F format and Gallery.

## Core skill

```bash
npx skills add suzheng/k2f --skill k2f
pip install k2f
```

Source: [`skills/core/k2f/`](core/k2f/) — K2F engine / authoring when the user works with `.K2F`, the Gallery, or exports explicitly.

## Layers

| Layer | Role | Examples |
|-------|------|----------|
| **Core** | K2F format, toolchain, authoring | `k2f` |
| **Task skills** | User intent → polished K2F document → export | `k2f-research-poster`, `k2f-invoice`, `k2f-cv`, … |
| **Expert skills** | Deep domain workflows | `k2f-fillable-pdf`, `k2f-idml`, `k2f-template-finder` |

Task skills teach agents *how to do the job well* (layout, QA, failure modes), not just the file format.

## Layout

```
skills/
  registry.json
  core/k2f/          # shipped — see SKILL.md
  discovery/ …
  expert/ …
  academic/ …
  …
```

See [`registry.json`](registry.json) for the full index. Each skill is a folder with `SKILL.md` plus optional `references/`, `scripts/`, and assets.

## Install

Works with Cursor, Claude Code, Codex, and other agents that support [npx skills](https://github.com/vercel-labs/skills).

**Manual fallback:** copy the [`core/k2f`](core/k2f/) folder into your agent skills path as `k2f/` (for example `~/.cursor/skills/k2f/`).

Entry point: [SKILL.md](core/k2f/SKILL.md) · [k2f.dev/skills/k2f](https://k2f.dev/skills/k2f) on the site.

## Related

- [K2F](https://github.com/suzheng/k2f) — format, CLI, SDK
- [K2F Gallery](https://k2f.dev/gallery) — reusable templates
- [K2F Skill on k2f.dev](https://k2f.dev/skills/k2f) — install command and overview
