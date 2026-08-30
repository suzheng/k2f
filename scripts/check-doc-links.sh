#!/usr/bin/env bash
set -euo pipefail
root="$(cd "$(dirname "$0")/.." && pwd)"
cd "$root"

python3 - <<'PY'
import re
import sys
from pathlib import Path

root = Path(".").resolve()
files = sorted(
    p
    for p in [
        root / "README.md",
        root / "CONTRIBUTING.md",
        root / "docs/README.md",
        * (root / "docs/spec").rglob("*.md"),
        * (root / "docs/guide").rglob("*.md"),
        * (root / "docs/architecture").rglob("*.md"),
        * (root / "docs/instructions").rglob("*.md"),
        * (root / "skills").rglob("*.md"),
    ]
    if p.is_file()
)
link_re = re.compile(r"\[[^\]]*\]\(([^)]+)\)")
for path in files:
    text = path.read_text(encoding="utf-8")
    for link in link_re.findall(text):
        if link.startswith(("http://", "https://", "mailto:")) or link.startswith("#"):
            continue
        link = link.split("#", 1)[0]
        if not link:
            continue
        # Skip placeholder / illustrative targets (e.g. `[t](url)` in mapping tables)
        if "/" not in link and "\\" not in link and not link.startswith("..") and "." not in link:
            continue
        target = (path.parent / link).resolve()
        if not target.exists():
            print(f"broken link in {path.relative_to(root)} -> {link}")
            sys.exit(1)
print("doc links ok")
PY
