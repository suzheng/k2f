# K2F agent skill

**One skill.** Create, edit, convert, export, publish, and embed K2F documents using **published** Python/CLI/npm tools — no source checkout required.

## Install

```bash
npx skills add suzheng/k2f --skill k2f
```

Works with Cursor, Claude Code, Codex, and other agents that support [npx skills](https://github.com/vercel-labs/skills).

**Manual fallback:** copy the **`k2f/`** directory into your agent skills path (e.g. Cursor `~/.cursor/skills/k2f/` or project `.cursor/skills/k2f/`), or use [skills/k2f on GitHub](https://github.com/suzheng/k2f/tree/main/skills/k2f).

## Install tools (before workflows)

```bash
pip install k2f              # CLI on PATH + Python SDK — no need to install Cargo
npm i @openk2f/k2f           # only for embedding <k2f-viewer>
# cargo install k2f          # optional: CLI without Python
```

MCP is not published on registries; this skill does not depend on MCP.

## Workflows

| Workflow | Use when |
|----------|----------|
| [writing](k2f/references/writing.md) | Create or update `.K2F` (JSON + pack; starter + catalog + schema lookup) |
| [converting-markdown](k2f/references/converting-markdown.md) | Convert Markdown ↔ K2F |
| [exporting-pdf](k2f/references/exporting-pdf.md) | Export PDF from the published lock |
| [exporting-pptx](k2f/references/exporting-pptx.md) | Export PPTX from the published lock |
| [embedding-viewer](k2f/references/embedding-viewer.md) | Embed `<k2f-viewer>` in a web app |
| [publishing](k2f/references/publishing.md) | Publish permanent `/v/{appearance_hash}` (requires your site origin) |

Entry point: [k2f/SKILL.md](k2f/SKILL.md).

## Bundled in the skill folder

| Path | Purpose |
|------|---------|
| `k2f/schema/` | Read-only format JSON Schemas (lookup — never copy into author dirs) |
| `k2f/starter/` | Empty package copied by `init_package.py` |
| `k2f/catalog/` | Golden `ex_*.json` shape reference ([index](k2f/catalog/README.md)) |
| `k2f/looks/` | Example design skins — theme + composition guide; fallback when the user did not name a style ([index](k2f/looks/README.md)) |
| `k2f/scripts/` | `init_package.py`, `pack_verify.py`, `edit_and_verify.py`, … |
