#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

echo "=== Phase 2 preflight ==="
cargo test -p k2f_sdk -p k2f_package -p k2f
bash scripts/check-vendored-schema.sh
bash scripts/check-cargo-package.sh

bash scripts/build-sdk-js.sh
bash scripts/build-viewer-only-js.sh
bash scripts/check-js-wasm.sh
if grep -R --include='*.rs' -l 'include_dir!' engine/k2f_sdk/src >/dev/null 2>&1; then
  echo "k2f_sdk must not use include_dir! (that embedded k2f/templates into WASM)" >&2
  exit 1
fi
if grep -q 'include_dir' engine/k2f_sdk/Cargo.toml; then
  echo "k2f_sdk Cargo.toml must not depend on include_dir" >&2
  exit 1
fi
cd sdk/js
npm test
npm pack --dry-run
cd "$ROOT"

cd sdk/python
export K2F_ENGINE_COMMIT_SHA="$(git -C "$ROOT" rev-parse HEAD)"
# Host-only wheel here (macOS on Mac, Linux on Linux). manylinux wheels:
#   bash scripts/build-linux-wheel.sh
#   or GHA publish-pypi workflow (maturin-action).
if [[ -x .venv/bin/maturin ]]; then
  .venv/bin/maturin build --release
  .venv/bin/pip install -q ../../target/wheels/k2f-*.whl
  .venv/bin/pytest -q
else
  python3 -m pip install -q maturin pytest
  python3 -m maturin build --release
  pip install -q ../../target/wheels/k2f-*.whl
  pytest -q
fi
cd "$ROOT"

echo "preflight OK"
