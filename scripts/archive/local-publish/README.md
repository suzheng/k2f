# Archived: local registry publish scripts

**Do not use these scripts.** They upload directly to crates.io, npm, and PyPI from a developer machine. That workflow is retired.

## Supported release path (best practice)

Publish through **GitHub Actions** — [`.github/workflows/publish.yml`](../../../.github/workflows/publish.yml):

1. Bump versions and update `CHANGELOG.md`.
2. Run preflight locally: `bash scripts/publish-preflight.sh`
3. Optional wheel-only dry run: `bash scripts/trigger-publish-dry-run.sh`
4. Optional Linux install smoke on **Modal** (verify SDK / `k2f` CLI on fresh Linux before or after tag):
   - `k2f-private/scripts/pypi-linux-smoke.sh` (`--build-wheel`, `--from-wheel`, or `--from-pypi`)
   - Docs: `k2f-private/scripts/modal-smoke-tests.md`
   - **Billing:** `modal run` smokes end when the script exits. If you opened a long-lived **Modal Sandbox** for manual testing, **Terminate** it in the Modal dashboard when finished — or it keeps billing.
5. Tag and push — the workflow publishes all registries:

   ```bash
   git tag v0.2.1
   git push origin v0.2.1
   ```

   Or use `workflow_dispatch` in the GitHub Actions UI / `gh workflow run publish.yml` with `upload_pypi`, `publish_crates`, and `publish_npm` set to `true`.

6. Optional: create a [GitHub Release](https://docs.github.com/en/repositories/releasing-projects-on-github) for the same tag so release notes and wheel artifacts are visible on the Releases page. Registry upload still happens in the workflow; the GitHub Release is for changelog and discovery.

### Why not local publish?

| Problem | Local scripts | GitHub Actions |
|---------|---------------|----------------|
| PyPI wheels | Host OS only (e.g. macOS arm64 on a Mac) | Linux / macOS / Windows matrix via maturin-action |
| `K2F_ENGINE_COMMIT_SHA` | Easy to forget | Set from `github.sha` in CI |
| Secrets | Laptop `.env` | GitHub repository secrets |
| Reproducibility | One machine | Same workflow every release |

`publish-pypi-local.sh` in this folder was the main reason PyPI only had macOS ARM wheels.

## What remains in `scripts/` (not archived)

| Script | Purpose |
|--------|---------|
| `publish-all.sh` | Prints the release cheat sheet (no upload) |
| `publish-preflight.sh` | Local validation before tagging |
| `trigger-publish-dry-run.sh` | Trigger GHA wheel build without uploading |
| `build-linux-wheel.sh` | Local manylinux wheel for Modal smoke tests |

## CI-only use of archived scripts

`publish.yml` still invokes `publish-crates.sh` and `publish-npm.sh` from this directory during tagged releases. **Contributors should not run them by hand** — only the workflow should call them.
