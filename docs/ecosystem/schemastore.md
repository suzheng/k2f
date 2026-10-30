# SchemaStore

[K2F](https://k2f.dev) format JSON Schemas are published to [SchemaStore](https://www.schemastore.org/) so editors and language servers can validate author JSON. The **source of truth** remains [`schema/`](../schema/) in this repository (with `k2f://` `$id` / `$ref` for the engine). Vendored copies for SchemaStore live under [`schema/schemastore/`](../schema/schemastore/).

## SchemaStore pull request

Submission: [SchemaStore/schemastore#6567](https://github.com/SchemaStore/schemastore/pull/6567) (pending merge). After merge, the catalog URLs below are live on schemastore.org.

## Catalog URLs (spec 0.3)

| File in an author directory | SchemaStore URL |
| --- | --- |
| `manifest.json` | `https://www.schemastore.org/k2f-manifest.json` |
| `content/root.json` and other semantic node JSON | `https://www.schemastore.org/k2f-nodes.json` |
| `styles/theme.json` | `https://www.schemastore.org/k2f-styles.json` |
| `signatures/v1.json` (packed packages) | `https://www.schemastore.org/partial-k2f-signatures.json` |

Subschema (referenced by theme/nodes schemas, not usually set as top-level `$schema`):

- `https://www.schemastore.org/partial-k2f-visual-primitives.json`

## Recommended `$schema` in author JSON

After the SchemaStore PR is merged (see above), add at the top of each file you edit:

```json
{
  "$schema": "https://www.schemastore.org/k2f-manifest.json",
  "title": "…"
}
```

```json
{
  "$schema": "https://www.schemastore.org/k2f-nodes.json",
  "id": "doc.root",
  "role": "…"
}
```

```json
{
  "$schema": "https://www.schemastore.org/k2f-styles.json",
  "palette": { }
}
```

Until merge, point `$schema` at a local path inside a checkout, for example `../../schema/schemastore/k2f-nodes.json`, or use workspace settings in [`.vscode/settings.json`](../.vscode/settings.json).

## Regenerate vendored copies

When [`schema/*.schema.json`](../schema/) changes on a spec bump:

```bash
./scripts/export-schemastore.sh
```

Then open a PR to [SchemaStore/schemastore](https://github.com/SchemaStore/schemastore) copying `schema/schemastore/*.json` into `src/schemas/json/`, update `src/api/json/catalog.json`, add positive tests under `src/test/k2f-*`, and register draft 2020-12 in `src/schema-validation.jsonc` (`highSchemaVersion`, `missingCatalogUrl` for `partial-k2f-visual-primitives.json`, and `options.externalSchema` for cross-file `$ref`). Run `npm clean-install && node cli.js check --schema-name=k2f-manifest.json` (and siblings) in the clone before opening the PR.

A ready-made branch with these edits may live in a local clone at `../schemastore` next to this repo (not committed into k2f).

Normative field rules: [Format spec 0.3](../spec/k2f-v0.3.md) and [EBNF appendix](../spec/k2f-v0.3.ebnf).

## License

Schemas are Apache-2.0, same as the [k2f repository](https://github.com/suzheng/k2f).
