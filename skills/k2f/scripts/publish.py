#!/usr/bin/env python3
"""Preflight and POST a .K2F package to POST /api/publish.

Avoids fragile hand-rolled curl: binary body, application/zip, lock hash check, 8 MiB cap.
"""

from __future__ import annotations

import argparse
import json
import os
import re
import sys
import urllib.error
import urllib.request
import zipfile
from pathlib import Path

MAX_PACKAGE_BYTES = 8 * 1024 * 1024
HASH_RE = re.compile(r"^[0-9a-f]{64}$")
LOCK_NAME = "document.K2F.lock"


def resolve_origin(args_origin: str | None) -> str:
    origin = args_origin or os.environ.get("K2F_PUBLISH_ORIGIN")
    if not origin:
        raise SystemExit(
            "publish origin required.\n"
            "Pass --origin https://your-k2f-site.example or set K2F_PUBLISH_ORIGIN.\n"
            "Embedding with <k2f-viewer> does not require publish."
        )
    return origin.rstrip("/")


def appearance_hash_from_package(path: Path) -> str:
    data = path.read_bytes()
    if len(data) == 0 or len(data) > MAX_PACKAGE_BYTES:
        raise SystemExit(
            f"invalid package size: {len(data)} bytes (max {MAX_PACKAGE_BYTES})"
        )
    if len(data) < 2 or data[0] != 0x50 or data[1] != 0x4B:
        raise SystemExit("expected ZIP package (PK magic)")

    try:
        with zipfile.ZipFile(path, "r") as zf:
            try:
                lock_raw = zf.read(LOCK_NAME)
            except KeyError as e:
                raise SystemExit(f"zip missing {LOCK_NAME}") from e
    except zipfile.BadZipFile as e:
        raise SystemExit(f"expected ZIP package: {e}") from e

    try:
        lock = json.loads(lock_raw.decode("utf-8"))
    except (UnicodeDecodeError, json.JSONDecodeError) as e:
        raise SystemExit(f"package lock missing appearance_hash: {e}") from e

    hash_val = lock.get("appearance_hash")
    if not isinstance(hash_val, str) or not HASH_RE.match(hash_val):
        raise SystemExit("package lock missing appearance_hash")
    return hash_val


def publish(path: Path, origin: str) -> dict:
    body = path.read_bytes()
    url = f"{origin}/api/publish"
    req = urllib.request.Request(
        url,
        data=body,
        method="POST",
        headers={"Content-Type": "application/zip"},
    )
    try:
        with urllib.request.urlopen(req) as resp:
            raw = resp.read().decode("utf-8")
            status = resp.status
    except urllib.error.HTTPError as e:
        err_body = e.read().decode("utf-8", errors="replace")
        print(f"publish HTTP {e.code}: {err_body}", file=sys.stderr)
        raise SystemExit(1) from e
    except urllib.error.URLError as e:
        print(f"publish request failed: {e}", file=sys.stderr)
        raise SystemExit(1) from e

    if status < 200 or status >= 300:
        print(f"publish HTTP {status}: {raw}", file=sys.stderr)
        raise SystemExit(1)

    try:
        return json.loads(raw)
    except json.JSONDecodeError as e:
        print(f"publish response json: {e}\n{raw}", file=sys.stderr)
        raise SystemExit(1) from e


def main() -> None:
    parser = argparse.ArgumentParser(
        description="Publish a locked .K2F package to POST /api/publish"
    )
    parser.add_argument("package", type=Path, help="Path to .K2F ZIP")
    parser.add_argument(
        "--origin",
        default=None,
        help="Publish API origin (required unless --dry-run; or set K2F_PUBLISH_ORIGIN)",
    )
    parser.add_argument(
        "--dry-run",
        action="store_true",
        help="Preflight only: print appearance_hash, do not POST",
    )
    args = parser.parse_args()

    path = args.package
    if not path.is_file():
        raise SystemExit(f"file not found: {path}")

    appearance_hash = appearance_hash_from_package(path)
    if args.dry_run:
        print(appearance_hash)
        return

    origin = resolve_origin(args.origin)
    result = publish(path, origin)
    print(json.dumps(result, indent=2))
    rel = result.get("url")
    if isinstance(rel, str) and rel.startswith("/"):
        print(f"absolute: {origin}{rel}")


if __name__ == "__main__":
    main()
