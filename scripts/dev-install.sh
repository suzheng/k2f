#!/usr/bin/env bash
set -euo pipefail
root="$(cd "$(dirname "$0")/.." && pwd)"
cd "$root/sdk/python"
python3 -m venv .venv
source .venv/bin/activate
pip install -U pip maturin
maturin develop --release
cd "$root"
bash scripts/build-sdk-js.sh
echo "dev install ok"
