#!/usr/bin/env bash
set -euo pipefail
root="$(cd "$(dirname "$0")/.." && pwd)"
cd "$root"
cargo build -p k2f -q
k2f="$root/target/debug/k2f"
mkdir -p examples/published
"$k2f" pack examples/invoice -o examples/published/invoice.K2F
"$k2f" compile examples/published/invoice.K2F
"$k2f" pack examples/contract -o examples/published/contract.K2F
"$k2f" compile examples/published/contract.K2F
echo "published examples ok"
