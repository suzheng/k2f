# K2F documentation

K2F is a document format: you write **meaning** as JSON, and one engine turns that into the same pixels everywhere.

A `.K2F` file is a ZIP with three layers:

- **Content** (`content/root.json`) — semantic tree. Every node has a stable `id`, a `role`, and `content`. No coordinates, colors, or font sizes.
- **Theme** (`styles/theme.json`) — appearance keyed by role (and optional variant). Fonts live here, plus `assets/fonts/`.
- **Lock** (`document.K2F.lock`) — compiled geometry. Viewers and PDF paint this. Do not hand-edit it.

`k2f pack` / `compile` is the compiler. Edit the tree or the theme, then relock.

## Get started

- [First document](guide/getting-started.md) — install, init a package, pack, verify
- [Format spec](spec/k2f-v0.1.md) — normative contract (paths, fields, verify codes)

## Authoring

Copy shapes from the [catalog](../skills/k2f/catalog/README.md) into `content/root.json` children. Look up allowed keys in [fields.md](../skills/k2f/references/writing/fields.md), then the matching file under [`schema/`](../skills/k2f/schema/).

- [Text](authoring/text.md) — paragraphs, line breaks, headings, emphasis, lists
- [Images](authoring/images.md) — embedded PNG/JPEG
- [Tables](authoring/tables.md) — inline tables
- [Layout](authoring/layout.md) — stack and grid
- [Theme and fonts](authoring/theme.md) — roles, variants, embedded fonts

## Reference

- [Catalog](../skills/k2f/catalog/README.md) — golden `ex_*.json` shapes
- [Allowed keys](../skills/k2f/references/writing/fields.md) — node, theme, and manifest fields
- [Format spec](spec/k2f-v0.1.md)

## Guides

- [Markdown conversion](../skills/k2f/references/converting-markdown.md)
- [Web viewer](../skills/k2f/references/embedding-viewer.md)

## Also

- [Agent skill](../skills/k2f/SKILL.md) — task playbooks that call the CLI
- [Project status](guide/status.md)
- [Desktop reader (in development)](../desktop/k2f_reader/README.md)
- [Contributing](../CONTRIBUTING.md)
- [Security](../SECURITY.md)

Contributor internals (engine design, crate map) live under [`architecture/`](architecture/README.md). They are not the authoring docs.
