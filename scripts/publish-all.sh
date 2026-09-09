#!/usr/bin/env bash
# Release cheat sheet — does not upload anything. Prints the supported publish flow.
#
# Full docs: scripts/archive/local-publish/README.md
# Workflow:  .github/workflows/publish.yml
set -euo pipefail

cat <<'EOF'
K2F release (GitHub Actions — do not publish from a laptop)

  1. Bump versions + CHANGELOG.md
  2. bash scripts/publish-preflight.sh
  3. bash scripts/trigger-publish-dry-run.sh          # optional: build wheels only
  4. git tag vX.Y.Z && git push origin vX.Y.Z       # publishes crates.io, npm, PyPI

  Optional: gh release create vX.Y.Z --notes-file ...

  Linux wheel smoke (Modal): k2f-private/scripts/pypi-linux-smoke.sh

Legacy local upload scripts: scripts/archive/local-publish/ (retired)
EOF
