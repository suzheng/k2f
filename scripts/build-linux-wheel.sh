#!/usr/bin/env bash
set -euo pipefail

# Build a manylinux_2_28 x86_64 wheel inside Docker (Linux hosts or Mac with Docker).
# macOS-native wheels still need `maturin build` on the host or GHA publish-pypi.

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
IMAGE="${MANYLINUX_IMAGE:-quay.io/pypa/manylinux_2_28_x86_64}"
OUT_DIR="${WHEEL_OUT:-$ROOT/target/wheels}"

export K2F_ENGINE_COMMIT_SHA="$(git -C "$ROOT" rev-parse HEAD)"
mkdir -p "$OUT_DIR"

docker run --rm \
  --platform linux/amd64 \
  -e K2F_ENGINE_COMMIT_SHA \
  -v "$ROOT:/io" \
  -w /io \
  "$IMAGE" \
  bash -lc '
    set -euo pipefail
    curl --proto "=https" --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --profile minimal --default-toolchain stable
    source "$HOME/.cargo/env"
    /opt/python/cp312-cp312/bin/python -m pip install -q maturin
    cd /io/sdk/python
    /opt/python/cp312-cp312/bin/python -m maturin build --release \
      --out /io/target/wheels \
      --compatibility manylinux_2_28
  '

echo "Linux wheel(s) in $OUT_DIR:"
ls -1 "$OUT_DIR"/k2f-*.whl
