# Contributing to K2F

Thank you for helping. We care about **deterministic layout**, **honest integrity reporting**, and keeping the **spec, schemas, and engine** aligned.

**Repository:** [github.com/suzheng/k2f](https://github.com/suzheng/k2f) · **Community hub:** [k2f.dev/community](https://k2f.dev/community) · **Conformance (golden runner):** [k2f.dev/conformance](https://k2f.dev/conformance)

## Ways to contribute

| Area | Where to work | What helps |
| --- | --- | --- |
| **Bugs and ideas** | [GitHub Issues](https://github.com/suzheng/k2f/issues) (use the templates) | Minimal `.K2F`, repro steps, spec impact for format changes |
| **Author docs and spec** | [`docs/`](docs/README.md) in this repo | Guides, spec (`docs/spec/`), architecture notes for contributors |
| **Agent skill** | [k2f](https://github.com/suzheng/k2f) — [`skills/core/k2f/`](https://github.com/suzheng/k2f/tree/main/skills/core/k2f) | Workflows, catalog (`ex_*.json`), skill schemas, scripts |
| **Engine, CLI, exporters, viewer** | This repo (`engine/`, `cli/`, `export/`, `sdk/`) | Rust/Python/JS changes with tests |

Security vulnerabilities: [SECURITY.md](SECURITY.md) — not public issues.

## Prerequisites

- **Rust** (latest stable) — [rustup.rs](https://rustup.rs/)
- **Python 3.9+** — `k2f_py` / maturin when testing the Python SDK
- **Node.js 18+** — when changing `sdk/js` or running JS tests

See [COMPATIBILITY.md](COMPATIBILITY.md) for engine version matching and supported runtimes.

## Development setup

Clone and build the toolchain used by contributors and CI (no PyPI install required first):

```bash
git clone https://github.com/suzheng/k2f.git
cd k2f
bash scripts/dev-install.sh
bash scripts/test-getting-started.sh
```

`dev-install.sh` creates a venv under `sdk/python/.venv`, builds the Python package with maturin, and compiles JS WASM bindings. The smoke script writes a package, exports PDF, checks `PDF_IS_NOT_A_SOURCE`, and runs `k2f verify`.

On Windows, use Git Bash or WSL for the bash scripts. Rust is required.

```bash
cargo build
cargo test
```

## Documentation (single sources of truth)

| Content | Edit here | Published as |
| --- | --- | --- |
| Human docs (guides, authoring, spec) | [`docs/`](docs/README.md) | Copied into [`sdk/js/public/`](sdk/js/public/) via symlinks; shipped in `@openk2f/k2f` and shown on [k2f.dev/docs](https://k2f.dev/docs) |
| Agent skill (catalog, `SKILL.md`, skill `schema/`) | [`skills/core/k2f/`](skills/core/k2f/) in this repo | `npx skills add suzheng/k2f --skill k2f` · [k2f.dev/skills/k2f](https://k2f.dev/skills/k2f) |
| Install pointer | [`skills/README.md`](skills/README.md) | Registry and install overview |
| Format JSON Schema | [`schema/`](schema/) at repo root | Validators in Rust; skill schemas should stay in sync |

Do **not** edit files under `sdk/js/public/` by hand — in a git checkout they symlink to `docs/`, `schema/`, `skills/README.md`, and `meta/`; `npm pack` materializes copies into the tarball.

After changing markdown under `docs/` or repo meta files:

```bash
bash scripts/check-doc-links.sh
```

## Build and test

### Golden suite

Lock comparison is **geometry + render plan + content hash**. Engine commit SHA and `appearance_hash` are not golden-equal (they differ between debug `UNKNOWN` and release git SHA).

**Local (fast):** one compile per case, no PNG. Default when `CI` is unset:

```bash
cargo run -p k2f-test-runner -- run
cargo test -p k2f_sdk -p k2f_package -p k2f
```

**CI:** five compiles per case (determinism) plus a small PNG appearance gate (`elevation_shadows`, `card_variants`, `running_footer_page_numbers`). Published examples (`examples/published/*.K2F`) are not PNG-golden.

**Visual-gate PNG** (optional locally):

```bash
cargo run -p k2f-test-runner -- run --check-png --dataset-runs 1
```

Cases without committed `page_*.png` skip the pixel check. To refresh gate PNGs: `--update --check-png`.

Default `cargo test -p k2f_paint` still renders those scenes and checks PNG magic bytes, but does not byte-compare goldens (macOS vs Linux raster bytes can differ). Linux CI compares via `CHECK_PAINT_GOLDENS=1` and the `--check-png` job. Refresh paint-crate goldens with `UPDATE_PAINT_GOLDENS=1`.

**Nightly:** same determinism + PNG gate; see [`.github/workflows/golden-nightly.yml`](.github/workflows/golden-nightly.yml).

### JS / WASM (viewer changes)

```bash
bash scripts/build-sdk-js.sh
cd sdk/js && npm test
```

### Doc and spec checks

```bash
bash scripts/check-spec-paths.sh
bash scripts/check-doc-links.sh
```

## Official pixels

Never use browser `fillText` or CSS flow for document body text. The web viewer paints from the WASM lock executor only. Regression test: `sdk/js/test/no-fill-text.mjs`.

## Format schema changes

Repo-root [`schema/`](schema/) is the **format** contract (manifest, nodes, styles, visual_primitives, signatures). Agent dialects live under `engine/k2f_sdk/profiles/`; MCP catalog at `engine/k2f_mcp/mcp_tools.json`.

When changing format `schema/*.json`:

1. Update Rust validators in the same PR
2. Update [`docs/spec/k2f-v0.3.md`](docs/spec/k2f-v0.3.md)
3. Regenerate committed examples if needed (`bash scripts/build-published-examples.sh`)
4. Keep [k2f](https://github.com/suzheng/k2f) skill `schema/` aligned when the format contract changes

## Pull request checklist

- [ ] Tests pass locally (`cargo run -p k2f-test-runner -- run` plus crate tests above)
- [ ] Spec updated if format or verify codes changed
- [ ] Doc links check if you touched `docs/` or meta markdown
- [ ] No secrets, keys, or `.env` files committed
- [ ] No `fillText` / CSS-flow regression in the viewer

## Releasing (maintainers)

**Do not** publish from a laptop using `scripts/archive/local-publish/`.

**Do** release through GitHub Actions ([`.github/workflows/publish.yml`](.github/workflows/publish.yml)):

1. Work on feature branches; **squash-merge** into **`main`**, then `git checkout main && git pull` before bumping or tagging.
2. Bump versions and update `CHANGELOG.md`.
3. `bash scripts/publish-preflight.sh`
4. Optional: `bash scripts/trigger-publish-dry-run.sh` (wheels only, no registry upload)
5. `git tag vX.Y.Z && git push origin vX.Y.Z` — publishes crates.io, npm, and PyPI
6. `gh release create vX.Y.Z` with notes from `CHANGELOG.md` (tag push does not create a GitHub Release)
7. **Website:** bump `dependencies.k2f` (`npm:@openk2f/k2f@^…`) in **k2f-site**, `npm install`, test, and deploy — the tag workflow does not update [k2f.dev](https://k2f.dev)

Full maintainer checklist (including Linux smoke and site steps): `bash scripts/publish-all.sh`. Background: [scripts/archive/local-publish/README.md](scripts/archive/local-publish/README.md).

## Code of conduct

See [CODE_OF_CONDUCT.md](CODE_OF_CONDUCT.md).
