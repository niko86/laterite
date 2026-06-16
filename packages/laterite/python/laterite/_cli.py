"""``lat-check`` — the Python face of the Rust ``lat-check`` binary.

Flags, ``--json`` / ``--ndjson`` byte-shape and exit codes are
faithful to ``rust-packages/laterite-ags4-check/src/main.rs``
(the JSON/NDJSON strings are built by the *same* serde_json calls in
the native module). Exit codes: 0 clean · 1 findings · 3 not-found/io
· 4 not-utf8/not-ags4/unsupported-edition · 5 bad-dict/bad-args.
"""

from __future__ import annotations

import argparse
import sys
from importlib import resources

from . import _laterite_native as _native

_DICT_CHOICES = ("auto", "4.0.3", "4.0.4", "4.1", "4.1.1", "4.2")


def _print_readme() -> int:
    print(resources.files(__package__).joinpath("README-cli.md").read_text("utf-8"))
    return 0


def _plain(file: str, findings: list[dict], n: int) -> str:
    if n == 0:
        return f"{file}: clean (0 findings)\n"
    w = [4, 4, 5, 11]
    rows = []
    for f in findings:
        short = f["rule"].removeprefix("AGS Format Rule ")
        line = "-" if f["line"] is None else str(f["line"])
        cells = [short, line, f["group"], f["desc"]]
        for i, c in enumerate(cells):
            w[i] = max(w[i], len(c))
        rows.append(cells)
    head = ["Rule", "Line", "Group", "Description"]
    out = [f"{file}: {n} finding(s)"]
    out.append(" | ".join(h.ljust(w[i]) for i, h in enumerate(head)))
    out.append("-+-".join("-" * w[i] for i in range(4)))
    for r in rows:
        out.append(" | ".join(r[i].ljust(w[i]) for i in range(4)))
    return "\n".join(out) + "\n"


def main(argv: list[str] | None = None) -> int:
    argv = list(sys.argv[1:] if argv is None else argv)

    if "--readme" in argv:
        return _print_readme()
    if "-h" in argv or "--help" in argv:
        return _print_readme()

    p = argparse.ArgumentParser(prog="lat-check", add_help=False)
    p.add_argument("file", nargs="?")
    p.add_argument("--dict-version", choices=_DICT_CHOICES, default="auto")
    p.add_argument("--dict")
    p.add_argument("--json", action="store_true")
    p.add_argument("--ndjson", action="store_true")
    p.add_argument("--out")
    p.add_argument("--json-out")
    p.add_argument("--show-warnings", action="store_true")
    p.add_argument("--show-fyi", action="store_true")
    p.add_argument("--check-files", action="store_true")
    p.add_argument("--quiet", action="store_true")

    args, extra = p.parse_known_args(argv)

    # Unknown option (e.g. --tui, which the Python build does not carry):
    # the Rust default build treats it as "unknown option → exit 5".
    if extra:
        print(f"error: unexpected argument {extra[0]!r}", file=sys.stderr)
        return 5
    if args.file is None:
        print("error: an input file is required", file=sys.stderr)
        return 5
    # External --dict is deliberately unimplemented (O-28) — the Rust
    # binary returns BadDict (exit 5) for it too.
    if args.dict:
        print(
            "error: external --dict override is not implemented; use "
            "--dict-version (4.0.3/4.0.4/4.1/4.1.1/4.2) or omit it",
            file=sys.stderr,
        )
        return 5

    r = _native.run_check(
        path=args.file,
        dict_version=args.dict_version,
        include_warnings=args.show_warnings,
        include_fyi=args.show_fyi,
        check_files=args.check_files,
    )
    if not r.get("ok"):
        print(f"error: {r.get('error')}", file=sys.stderr)
        return int(r.get("exit_code", 5))

    n = r["count"]
    code = r["exit_code"]

    if args.json:
        active = r["json"] + "\n"
    elif args.ndjson:
        active = r["ndjson"]
    else:
        active = _plain(r["file"], r["findings"], n)

    if args.json_out:
        with open(args.json_out, "w", encoding="utf-8", newline="") as fh:
            fh.write(r["json"] + "\n")
        # tee: stdout still gets the normal (plain) report
        sys.stdout.write(_plain(r["file"], r["findings"], n))
        return code

    if args.out:
        with open(args.out, "w", encoding="utf-8", newline="") as fh:
            fh.write(active)
        print(f"{r['file']}: {n} finding(s)" if n else f"{r['file']}: clean (0 findings)")
        return code

    sys.stdout.write(active)
    return code


if __name__ == "__main__":
    raise SystemExit(main())
