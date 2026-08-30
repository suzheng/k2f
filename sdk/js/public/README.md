# `k2f/public` exports

Stable package paths for docs and contracts. In a git checkout these are **relative symlinks** to the single source of truth elsewhere in the repo (not copies).

| Export | Points to |
|--------|-----------|
| `k2f/public/architecture/*` | `docs/architecture/` |
| `k2f/public/spec/*` | `docs/spec/` |
| `k2f/public/guide/*` | `docs/guide/` |
| `k2f/public/docs-readme.md` | `docs/README.md` |
| `k2f/public/instructions/*` | public authoring docs under `docs/instructions/` |
| `k2f/public/skills/*` | `skills/` |
| `k2f/public/mcp_tools.json` | `engine/k2f_mcp/mcp_tools.json` |
| `k2f/public/schema/*` | `schema/` |
| `k2f/public/examples/*` | `examples/published/` |
| `k2f/public/contracts/*` | generated from Rust by `scripts/emit-js-contracts.mjs` |
| `k2f/public/meta/*` | root SECURITY / CONTRIBUTING / … |
| `k2f/public/fixtures/*` | selected test fixtures |

Do not add `docs/plans`, `docs/prompts`, or `experiments/` here.
