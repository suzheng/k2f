#!/usr/bin/env bash
set -euo pipefail

# Trigger publish workflow in wheel-build-only mode (no PyPI/crates/npm upload).
# Requires: gh auth login, push access to suzheng/k2f

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

if ! command -v gh >/dev/null 2>&1; then
  echo "gh CLI required: https://cli.github.com/" >&2
  exit 1
fi

gh workflow run publish.yml \
  --repo suzheng/k2f \
  -f upload_pypi=false \
  -f publish_crates=false \
  -f publish_npm=false

echo "Triggered publish.yml (wheels only)."
echo "Watch: gh run list --workflow=publish.yml --repo suzheng/k2f"
echo "Artifacts: wheels-<runner>-<target> under the completed run."
