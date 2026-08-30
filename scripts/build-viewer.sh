#!/usr/bin/env bash
set -euo pipefail
root="$(cd "$(dirname "$0")/.." && pwd)"
bash "$root/scripts/build-sdk-js.sh"
bash "$root/scripts/build-viewer-only-js.sh"
