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


def _run_fix(args) -> int:
    """`--fix`: mechanically repair the file and write the result. Faithful to
    the Rust `run_fix` (sibling `<file>.fixed.ags` by default; `--in-place` /
    `--fix-out` to redirect). Exit 0 if the repaired file is clean, 1 if findings
    remain that aren't auto-fixable, 3/4/5 on read/parse/dict errors."""
    import json
    from pathlib import Path

    if args.in_place and args.fix_out:
        print("error: --in-place and --fix-out are mutually exclusive", file=sys.stderr)
        return 5

    r = _native.fix_file(
        path=args.file,
        dict_version=args.dict_version,
        include_risky=args.fix_risky,
        encoding=args.encoding,
    )
    if not r.get("ok"):
        print(f"error: {r.get('error')}", file=sys.stderr)
        return int(r.get("exit_code", 5))

    src = Path(args.file)
    if args.in_place:
        dest = src
    elif args.fix_out:
        dest = Path(args.fix_out)
    elif src.suffix:
        dest = src.with_name(f"{src.stem}.fixed{src.suffix}")  # delivery.ags → delivery.fixed.ags
    else:
        dest = src.with_name(f"{src.name}.fixed")
    try:
        dest.write_bytes(r["fixed"])
    except OSError as e:
        print(f"error: writing {dest}: {e}", file=sys.stderr)
        return 3

    kinds = sorted({a["kind"] for a in r["applied"]})
    n_applied = r["fixes_applied"]
    if n_applied == 0:
        print(f"no fixes applicable → {dest}")
    else:
        print(f"applied {n_applied} fix(es) [{', '.join(kinds)}] → {dest}")

    by_rule = json.loads(r["findings_json"])
    n_residual = sum(len(v) for v in by_rule.values())
    if n_residual == 0:
        print(f"{dest}: clean (0 findings)")
        return 0
    print(f"{dest}: {n_residual} finding(s) remain (not mechanically fixable)")
    return 1


def _list_rules(as_json: bool) -> int:
    """`--list-rules`: print the rule catalogue and exit 0. `--json` emits the
    raw gated `rules_meta.json` (byte-faithful to the Rust binary); otherwise a
    compact Rule | Title | Severity | Fix? table."""
    raw = _native.list_rules()
    if as_json:
        print(raw)
        return 0
    import json

    rules = json.loads(raw)["rules"]
    w = [4, 5, 8, 4]
    rows = []
    for r in rules:
        cells = [
            r["rule"],
            r.get("title", ""),
            r.get("severity", ""),
            "yes" if r.get("fixable") else "",
        ]
        for i, c in enumerate(cells):
            w[i] = max(w[i], len(c))
        rows.append(cells)
    head = ["Rule", "Title", "Severity", "Fix?"]
    # rstrip each line so the last (ljust-padded) column carries no trailing
    # whitespace — matching the Rust table.
    out = [" | ".join(h.ljust(w[i]) for i, h in enumerate(head)).rstrip()]
    out.append("-+-".join("-" * w[i] for i in range(4)))
    for r in rows:
        out.append(" | ".join(r[i].ljust(w[i]) for i in range(4)).rstrip())
    print("\n".join(out))
    return 0


def _run_diff(args) -> int:
    """`--diff <other>`: print the KEY-aware/type-aware revision delta between the
    input file (baseline) and <other> (revision). Faithful to the Rust `run_diff`:
    a per-group `+added -removed ~changed` summary (or the full `RevisionDelta` with
    `--json`). Exit 0; 3/4/5 on read/parse/dict errors."""
    import json
    from pathlib import Path

    try:
        a = Path(args.file).read_bytes()
        b = Path(args.diff).read_bytes()
    except OSError as e:
        print(f"error: {e}", file=sys.stderr)
        return 3
    r = _native.diff_files(a, b, dict_version=args.dict_version, encoding=args.encoding)
    if not r.get("ok"):
        print(f"error: {r.get('error')}", file=sys.stderr)
        return int(r.get("exit_code", 5))
    delta = json.loads(r["delta_json"])
    if args.json:
        print(json.dumps(delta, indent=2))
        return 0
    print(f"{args.file} → {args.diff}")
    for g in delta["groups"]:
        print(f"  {g['code']:<6} +{g['added']} -{g['removed']} ~{g['changed']}")
    if delta["groups_added"]:
        print(f"  groups added:   {', '.join(delta['groups_added'])}")
    if delta["groups_removed"]:
        print(f"  groups removed: {', '.join(delta['groups_removed'])}")
    print(
        f"  total: +{delta['total_added']} added · "
        f"−{delta['total_removed']} removed · ~{delta['total_changed']} changed"
    )
    return 0


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
    p.add_argument("--encoding")
    p.add_argument("--quiet", action="store_true")
    p.add_argument("--fix", action="store_true")
    p.add_argument("--fix-risky", action="store_true")
    p.add_argument("--in-place", action="store_true")
    p.add_argument("--fix-out")
    p.add_argument("--diff")
    p.add_argument("--list-rules", action="store_true")

    args, extra = p.parse_known_args(argv)

    # Unknown option (e.g. --tui, which the Python build does not carry):
    # the Rust default build treats it as "unknown option → exit 5".
    if extra:
        print(f"error: unexpected argument {extra[0]!r}", file=sys.stderr)
        return 5
    # `--list-rules`: informational, input-independent — print + exit (before
    # the required-input-file check), faithful to the Rust binary.
    if args.list_rules:
        return _list_rules(args.json)
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

    # `--json` and `--ndjson` are mutually exclusive (faithful to main.rs:249).
    if args.json and args.ndjson:
        print("error: --json and --ndjson are mutually exclusive", file=sys.stderr)
        return 5

    # `--diff <other>`: compare the input file against <other> and exit (faithful
    # to main.rs `run_diff`); never falls through to the validate-report.
    if args.diff:
        return _run_diff(args)

    # `--fix`: repair-and-write path (faithful to main.rs `run_fix`); never
    # falls through to the validate-report below.
    if args.fix or args.fix_risky:
        return _run_fix(args)
    if args.in_place or args.fix_out:
        print("error: --in-place / --fix-out only apply with --fix", file=sys.stderr)
        return 5

    r = _native.run_check(
        path=args.file,
        dict_version=args.dict_version,
        include_warnings=args.show_warnings,
        include_fyi=args.show_fyi,
        check_files=args.check_files,
        encoding=args.encoding,
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
