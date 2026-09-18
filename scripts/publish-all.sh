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
     (preflight refuses SDK WASM that still exports official_templates / openTemplate,
      and k2f_sdk that include_dir-embeds k2f/templates/)
  3. bash scripts/trigger-publish-dry-run.sh          # optional: build wheels only
  4. git tag vX.Y.Z && git push origin vX.Y.Z       # publishes crates.io, npm, PyPI

  npm @openk2f/k2f must not embed templates/. Agents create packages with
  init_package.py / unpack / Gallery; the SDK opens dirs or .K2F bytes.

  Optional: gh release create vX.Y.Z --notes-file ...

  4. Modal — Linux SDK/CLI smoke (before/after tag; k2f-private, source .env):
       ../k2f-private/scripts/pypi-linux-smoke.sh --build-wheel
     Docs: ../k2f-private/scripts/modal-smoke-tests.md

  *** Modal billing ***
  pypi-linux-smoke.sh uses `modal run` and stops when the script exits.
  If you started a long-lived Modal Sandbox for manual testing, Terminate it
  in the Modal dashboard (Sandboxes) when finished — or it keeps billing.

Legacy local upload scripts: scripts/archive/local-publish/ (retired)
EOF
