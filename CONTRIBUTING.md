# Contributing to K2F

Thank you for contributing. This project values deterministic layout, honest integrity reporting, and spec-code alignment.

## Prerequisites

- **Rust** (latest stable) — [rustup.rs](https://rustup.rs/)
- **Python 3.9+** — for `k2f_py` / maturin when testing the Python SDK
- **Node.js 18+** — when changing `sdk/js` or running JS tests

See [COMPATIBILITY.md](COMPATIBILITY.md) for engine version matching and release expectations.

## Releasing

**Do not** run `scripts/archive/local-publish/` from your machine. That folder holds retired laptop publish scripts (including the old `publish-all.sh` flow).

**Do** publish via GitHub Actions ([`.github/workflows/publish.yml`](.github/workflows/publish.yml)):

1. `bash scripts/publish-preflight.sh`
2. `bash scripts/trigger-publish-dry-run.sh` (wheels only, no registry upload)
3. `git tag vX.Y.Z && git push origin vX.Y.Z` — publishes crates.io, npm, and PyPI
4. Optionally `gh release create vX.Y.Z` for release notes on GitHub

Details: [scripts/archive/local-publish/README.md](scripts/archive/local-publish/README.md)

## Reporting issues

Use the GitHub issue templates (`.github/ISSUE_TEMPLATE/`):

- **Bug report** — layout, verify codes, viewer behavior; attach a minimal `.K2F` or SDK snippet when possible
- **Feature request** — format/SDK/tooling ideas; note spec impact and backward compatibility

Do **not** use public issues for security vulnerabilities — see [SECURITY.md](SECURITY.md).

## Documentation

Edit the **source** files:

- `docs/` — specification, guides, architecture, instructions
- `skills/` — agent skill playbooks

Do not edit copies under `sdk/js/public/` by hand. In a git checkout those paths are symlinks to the sources above; npm pack materializes them into the tarball.

After doc changes, run:

```bash
bash scripts/check-doc-links.sh
```

## Build and test

```bash
cargo build
cargo test
```

### Golden suite

Lock comparison is **geometry + render plan + content hash**. Engine commit SHA and
`appearance_hash` are not golden-equal (they differ between debug `UNKNOWN` and
release git SHA).

**Local (fast):** one compile per case, no PNG. Default when `CI` is unset:

```bash
cargo run -p k2f-test-runner -- run
cargo test -p k2f_sdk -p k2f_package -p k2f
```

**CI:** five compiles per case (determinism) plus a small PNG appearance gate
(`elevation_shadows`, `card_variants`, `running_footer_page_numbers`).
Published examples (`examples/published/*.K2F`) are not PNG-golden.

**Visual-gate PNG** (optional locally):

```bash
cargo run -p k2f-test-runner -- run --check-png --dataset-runs 1
```

Cases without committed `page_*.png` skip the pixel check. To refresh gate PNGs:
`--update --check-png`.

**Nightly:** same determinism + PNG gate; see `.github/workflows/golden-nightly.yml`.

JS / WASM (when changing the viewer):

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

## Schema changes

`schema/*.json` is the **format** contract only (manifest, nodes, styles, visual_primitives, signatures). Agent dialects live under `engine/k2f_sdk/profiles/`; MCP catalog at `engine/k2f_mcp/mcp_tools.json`.

When changing format `schema/*.json`:

1. Update Rust validators in the same PR
2. Update `docs/spec/k2f-v0.1.md`
3. Regenerate committed examples if needed (`bash scripts/build-published-examples.sh`)

## Pull request checklist

- [ ] Tests pass locally (`cargo run -p k2f-test-runner -- run` plus crate tests above)
- [ ] Spec updated if format or verify codes changed
- [ ] No secrets, keys, or `.env` files committed
- [ ] No `fillText` / CSS-flow regression in viewer

## Code of conduct

See [CODE_OF_CONDUCT.md](CODE_OF_CONDUCT.md).
