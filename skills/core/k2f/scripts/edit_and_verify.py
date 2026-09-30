#!/usr/bin/env python3
"""Deterministic Editor open → mutate → save → k2f verify (expect UNSIGNED)."""
from __future__ import annotations

import argparse
import json
import os
import shutil
import subprocess
import sys
from pathlib import Path


def verify_unsigned(cli: str, path: Path) -> tuple[int, str, str]:
    proc = subprocess.run(
        [cli, "verify", str(path)],
        capture_output=True,
        text=True,
    )
    return proc.returncode, proc.stdout.strip(), proc.stderr.strip()


def k2f_candidates() -> list[str]:
    """Prefer K2F_CLI, then `k2f` on PATH."""
    found: list[str] = []
    env_cli = os.environ.get("K2F_CLI")
    if env_cli:
        found.append(env_cli)
    which = shutil.which("k2f")
    if which and which not in found:
        found.append(which)
    return found


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(
        description="Edit a .K2F by node id, relock, and verify UNSIGNED."
    )
    parser.add_argument("--file", required=True, type=Path, help="Input .K2F package")
    parser.add_argument("--id", required=True, help="Stable node id")
    parser.add_argument("--text", help="New text (Text or Math node)")
    parser.add_argument("--role", help="set_role target (package theme must define it)")
    parser.add_argument("--variant", default=None, help="Optional role variant")
    parser.add_argument(
        "--generated-by",
        default=None,
        help="Changelog agent id (set_generated_by)",
    )
    parser.add_argument(
        "--out",
        required=True,
        type=Path,
        help="Output .K2F path",
    )
    parser.add_argument(
        "--expected-content-hash",
        default=None,
        help="Optimistic lock: must match hash at open time",
    )
    args = parser.parse_args(argv)

    if args.text is None and args.role is None:
        parser.error("provide at least one of --text or --role")

    import k2f

    blob = args.file.read_bytes()
    ed = k2f.Editor.open_bytes(blob)

    before = json.loads(ed.get_node(args.id))
    preview = json.dumps(before, ensure_ascii=False)
    if len(preview) > 160:
        preview = preview[:160] + "…"
    print("before:", preview)

    if args.text is not None:
        ed.replace_text(args.id, args.text)
    if args.role is not None:
        ed.set_role(args.id, args.role, args.variant)
    if args.generated_by is not None:
        ed.set_generated_by(args.generated_by)

    args.out.parent.mkdir(parents=True, exist_ok=True)
    if args.expected_content_hash is not None:
        out_bytes = bytes(ed.save_with(args.expected_content_hash))
    else:
        out_bytes = bytes(ed.save_bytes())
    args.out.write_bytes(out_bytes)

    ed2 = k2f.Editor.open_bytes(out_bytes)
    after = json.loads(ed2.get_node(args.id))
    if args.text is not None:
        value = after.get("content", {}).get("value")
        if value != args.text:
            print(f"reopen text mismatch: {value!r}", file=sys.stderr)
            return 1
    if args.role is not None and after.get("role") != args.role:
        print(f"reopen role mismatch: {after.get('role')!r}", file=sys.stderr)
        return 1

    candidates = k2f_candidates()
    if not candidates:
        print(
            "k2f CLI not found; put `k2f` on PATH or set K2F_CLI. "
            "Semantic reopen check passed.",
            file=sys.stderr,
        )
        print(f"wrote {args.out}")
        return 0

    banner = None
    used = None
    stderr = ""
    for cli in candidates:
        code, out, err = verify_unsigned(cli, args.out)
        if out == "UNSIGNED" and code == 0:
            banner, used, stderr = out, cli, err
            break
        banner, used, stderr = out, cli, err

    print(f"verify ({used}): {banner}")
    if banner != "UNSIGNED":
        if stderr:
            print(stderr, file=sys.stderr)
        print(f"expected UNSIGNED, got {banner!r}", file=sys.stderr)
        return 1
    print(f"wrote {args.out}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
