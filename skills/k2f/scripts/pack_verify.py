#!/usr/bin/env python3
"""Pack → compile → verify an unpacked K2F author directory.

Uses the `k2f` CLI on PATH or K2F_CLI. CLI stderr/stdout are forwarded so agents
can map failures via references/writing/errors.md.
"""

from __future__ import annotations

import argparse
import os
import re
import shutil
import subprocess
import sys
from pathlib import Path

PAGES_RE = re.compile(r"pages=(\d+)")
SLACK_MARK = "LAYOUT_SLACK"


def k2f_binary() -> list[str]:
    env_cli = os.environ.get("K2F_CLI")
    if env_cli:
        path = Path(env_cli).expanduser().resolve()
        if path.is_file():
            return [str(path)]
        print(f"error: K2F_CLI is not a file: {path}", file=sys.stderr)
        sys.exit(1)
    which = shutil.which("k2f")
    if which:
        return [which]
    print(
        "error: `k2f` CLI not found on PATH.\n"
        "Install: pip install k2f\n"
        "Or set K2F_CLI to the binary path.",
        file=sys.stderr,
    )
    sys.exit(1)


def run(cmd: list[str], cwd: Path | None = None) -> subprocess.CompletedProcess[str]:
    print("+", " ".join(cmd), flush=True)
    proc = subprocess.run(cmd, cwd=cwd, capture_output=True, text=True)
    if proc.stdout:
        print(proc.stdout, end="" if proc.stdout.endswith("\n") else "\n", flush=True)
    if proc.stderr:
        print(
            proc.stderr,
            end="" if proc.stderr.endswith("\n") else "\n",
            file=sys.stderr,
            flush=True,
        )
    if proc.returncode != 0:
        sys.exit(proc.returncode)
    return proc


def parse_pages(stderr: str) -> int | None:
    matches = PAGES_RE.findall(stderr)
    if not matches:
        return None
    return int(matches[-1])


def main() -> int:
    parser = argparse.ArgumentParser(description="Pack, compile, and verify a K2F source dir")
    parser.add_argument("source_dir", type=Path, help="Unpacked package directory")
    parser.add_argument(
        "-o",
        "--output",
        type=Path,
        required=True,
        help="Output .K2F path",
    )
    parser.add_argument(
        "--render",
        type=Path,
        metavar="PNG",
        help="Optional: render one page to this PNG after verify",
    )
    parser.add_argument(
        "--page",
        type=int,
        default=0,
        help="Page index for --render (default 0)",
    )
    parser.add_argument(
        "--expect-pages",
        type=int,
        default=None,
        metavar="N",
        help="Fail if compile reports a page count other than N (posters: 1)",
    )
    parser.add_argument(
        "--expect-fill",
        action="store_true",
        help="Fail if compile prints LAYOUT_SLACK (poster/slide page shells)",
    )
    args = parser.parse_args()

    source = args.source_dir.expanduser().resolve()
    if not source.is_dir():
        print(f"error: not a directory: {source}", file=sys.stderr)
        return 1
    for required in ("manifest.json", "content/root.json", "styles/theme.json"):
        if not (source / required).is_file():
            print(f"error: missing {required} under {source}", file=sys.stderr)
            return 1
    fonts = source / "assets" / "fonts"
    if not fonts.is_dir() or not any(fonts.iterdir()):
        print(f"error: no fonts under {fonts}", file=sys.stderr)
        return 1

    output = args.output.expanduser().resolve()
    output.parent.mkdir(parents=True, exist_ok=True)

    prefix = k2f_binary()
    print(f"using: {' '.join(prefix)}", flush=True)

    run([*prefix, "pack", str(source), "-o", str(output)])
    compile_proc = run([*prefix, "compile", str(output)])
    pages = parse_pages(compile_proc.stderr)
    if args.expect_pages is not None:
        if pages is None:
            print(
                "error: compile did not print pages=N — upgrade `k2f` CLI (pip install -U k2f)",
                file=sys.stderr,
            )
            return 1
        if pages != args.expect_pages:
            print(
                f"error: expected {args.expect_pages} page(s), got {pages} "
                f"(preview.png is only page 0 — shrink layout or raise --expect-pages)",
                file=sys.stderr,
            )
            return 1
    if args.expect_fill:
        if SLACK_MARK in compile_proc.stderr:
            print(
                "error: LAYOUT_SLACK — page-height shell is not filled. "
                "Copy catalog/content/ex_poster_shell.json ({fr:1} body row); "
                "do not add spacer nodes. See references/writing/errors.md",
                file=sys.stderr,
            )
            return 1
    run([*prefix, "verify", str(output)])

    if args.render is not None:
        render_out = args.render.expanduser().resolve()
        render_out.parent.mkdir(parents=True, exist_ok=True)
        run(
            [
                *prefix,
                "render",
                str(output),
                "--page",
                str(args.page),
                "-o",
                str(render_out),
            ]
        )

    if pages is not None:
        print(f"ok: {output} pages={pages}")
    else:
        print(f"ok: {output}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
