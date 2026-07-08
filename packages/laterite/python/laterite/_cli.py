"""``lat`` — the Python face of the Rust ``lat`` binary.

A clap-style subcommand tool (``validate`` / ``fix`` / ``diff`` / ``certify`` /
``rules``) whose output, ``--json`` / ``--ndjson`` byte-shape and exit codes are
faithful to ``rust-packages/laterite-ags4-check`` (the JSON/NDJSON strings are
built by the *same* serde_json calls in the native module). A bare ``lat <file>``
is shorthand for ``lat validate <file>``. Exit codes: 0 clean · 1 findings ·
3 not-found/io · 4 not-utf8/not-ags4/unsupported-edition · 5 bad-dict/bad-args.
"""

from __future__ import annotations

import argparse
import sys
from importlib import resources

from . import _laterite_native as _native

_DICT_CHOICES = ("auto", "4.0.3", "4.0.4", "4.1", "4.1.1", "4.2")
_SUBCOMMANDS = (
    "validate",
    "read",
    "fix",
    "diff",
    "certify",
    "rules",
    "pack",
    "unpack",
    "lock",
    "unlock",
    "excel",
)


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


def _run_validate(args) -> int:
    r = _native.run_check(
        path=args.file,
        dict_version=args.dict_version,
        include_warnings=not args.no_warnings,
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
        sys.stdout.write(_plain(r["file"], r["findings"], n))  # tee: plain still to stdout
        return code

    if args.out:
        with open(args.out, "w", encoding="utf-8", newline="") as fh:
            fh.write(active)
        print(f"{r['file']}: {n} finding(s)" if n else f"{r['file']}: clean (0 findings)")
        return code

    sys.stdout.write(active)
    return code


def _run_fix(args) -> int:
    """`lat fix`: mechanically repair the file. Faithful to the Rust `fix`
    (sibling `<file>.fixed.ags` by default; `--in-place` / `--fix-out` redirect).
    Exit 0 clean · 1 residual · 3/4/5 read/parse/dict errors."""
    import json
    from pathlib import Path

    if args.in_place and args.fix_out:
        print("error: --in-place and --fix-out are mutually exclusive", file=sys.stderr)
        return 5

    r = _native.fix_file(
        path=args.file,
        dict_version=args.dict_version,
        include_risky=args.risky,
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
        dest = src.with_name(f"{src.stem}.fixed{src.suffix}")
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
    """`lat rules`: print the rule catalogue and exit 0. `--json` emits the raw
    gated `rules_meta.json` (byte-faithful to the Rust binary); otherwise a
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
    out = [" | ".join(h.ljust(w[i]) for i, h in enumerate(head)).rstrip()]
    out.append("-+-".join("-" * w[i] for i in range(4)))
    for r in rows:
        out.append(" | ".join(r[i].ljust(w[i]) for i in range(4)).rstrip())
    print("\n".join(out))
    return 0


def _run_diff(args) -> int:
    """`lat diff <a> <b>`: the KEY-aware/type-aware revision delta. Faithful to
    the Rust `diff`: a per-group summary (or the full delta with `--json`)."""
    import json
    from pathlib import Path

    try:
        a = Path(args.file).read_bytes()
        b = Path(args.other).read_bytes()
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
    print(f"{args.file} → {args.other}")
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


def _run_certify(args) -> int:
    """`lat certify <file>`: mint the `.ags.idx` for an error-clean file, via the
    high-level library (read → validate → certify). Faithful to the Rust `certify`
    output ("certificate written to <path>"); a file with errors can't be certified."""
    from . import read

    dv = None if args.dict_version == "auto" else args.dict_version
    try:
        handle = read(args.file, encoding=args.encoding).validate(
            dict_version=dv, check_files=args.check_files
        )
    except FileNotFoundError:
        print(f"error: {args.file}: not found", file=sys.stderr)
        return 3
    except Exception as e:  # noqa: BLE001 — parse/dict errors surface here
        print(f"error: {e}", file=sys.stderr)
        return 4

    if not handle.report.is_valid:
        print(
            f"cannot certify: {handle.report.count} finding(s) — a certificate attests a "
            f"clean validation (run `lat validate {args.file}` to see them)",
            file=sys.stderr,
        )
        return 1
    try:
        dest = handle.certify(args.out) if args.out else handle.certify()
    except Exception as e:  # noqa: BLE001
        print(f"error: {e}", file=sys.stderr)
        return 3
    print(f"certificate written to {dest}")
    return 0


def _emit(body: str, out: str | None) -> int:
    if out:
        with open(out, "w", encoding="utf-8", newline="") as fh:
            fh.write(body)
        print(f"note: written to {out}", file=sys.stderr)
    else:
        sys.stdout.write(body)
    return 0


def _csv_row(cells) -> str:
    """One RFC-4180-ish CSV line — byte-identical to the Rust `read` CSV: quote a
    field iff it contains `,` / `"` / CR / LF, doubling internal quotes."""
    out = []
    for c in cells:
        if any(ch in c for ch in ',"\r\n'):
            out.append('"' + c.replace('"', '""') + '"')
        else:
            out.append(c)
    return ",".join(out) + "\n"


def _read_table(headings, rows) -> str:
    """A plain aligned table for `lat read <group>` — the human view (the Rust
    binary renders its own comfy-table box grid; only --json/--csv are byte-coherent)."""
    w = [len(h) for h in headings]
    for row in rows:
        for i, c in enumerate(row):
            w[i] = max(w[i], len(c))
    lines = [" | ".join(h.ljust(w[i]) for i, h in enumerate(headings)).rstrip()]
    lines.append("-+-".join("-" * w[i] for i in range(len(headings))))
    for row in rows:
        lines.append(" | ".join(row[i].ljust(w[i]) for i in range(len(headings))).rstrip())
    return "\n".join(lines) + "\n"


def _run_read(args) -> int:
    """`lat read <file> [group]` — list the file's group codes, or dump a group
    as a table / CSV / JSON. Raw file cells via the native read codec, so the
    Rust binary and this CLI agree byte-for-byte on the group listing and on
    `read --json` / `--csv` (the human --table is each surface's own — #430 PR 2)."""
    import json as _json
    from pathlib import Path

    if not Path(args.file).exists():
        print(f"error: {args.file}: not found", file=sys.stderr)
        return 3
    try:
        raw = _native.read_groups_raw(args.file)
    except Exception as e:  # noqa: BLE001 — parse/dict errors surface here
        print(f"error: {e}", file=sys.stderr)
        return 4

    order = raw["order"]
    if args.group is None:
        if not order:
            print("note: no groups in the file", file=sys.stderr)
            return 0
        body = (
            _json.dumps(order, indent=2, ensure_ascii=False) + "\n"
            if args.json
            else "\n".join(order) + "\n"
        )
        return _emit(body, args.out)

    if args.group not in raw["groups"]:
        present = ", ".join(order) or "none"
        print(
            f"error: group {args.group!r} not found in {args.file} (present: {present})",
            file=sys.stderr,
        )
        return 4

    g = raw["groups"][args.group]
    headings, rows = g["headings"], g["rows"]
    if args.json:
        objs = [dict(zip(headings, row, strict=True)) for row in rows]
        body = _json.dumps(objs, indent=2, ensure_ascii=False) + "\n"
    elif args.csv:
        body = "".join(_csv_row(r) for r in [headings, *rows])
    else:
        body = _read_table(headings, rows)
    return _emit(body, args.out)


def _resolve_password(password_file, prompt: str) -> str:
    """Passphrase precedence — `--password-file` → `$LAT_TRANSPORT_PASSWORD` → an
    interactive prompt (never echoed). NEVER a `--password` flag: argv leaks into
    `ps` and shell history. Mirrors the Rust `lat`'s resolution exactly."""
    import os
    from pathlib import Path

    if password_file:
        return Path(password_file).read_text(encoding="utf-8").rstrip("\r\n")
    env = os.environ.get("LAT_TRANSPORT_PASSWORD")
    if env:
        return env
    import getpass

    return getpass.getpass(prompt)


def _run_pack(args) -> int:
    from pathlib import Path

    from . import transport

    if not Path(args.input).exists():
        print(f"error: {args.input}: not found", file=sys.stderr)
        return 3
    try:
        transport.pack(args.input, dest=args.output, level=args.level)
    except Exception as e:  # noqa: BLE001 — zstd failure surfaces here
        print(f"error: {e}", file=sys.stderr)
        return 6
    print(f"packed {args.input} → {args.output}", file=sys.stderr)
    return 0


def _run_unpack(args) -> int:
    from pathlib import Path

    from . import transport

    if not Path(args.input).exists():
        print(f"error: {args.input}: not found", file=sys.stderr)
        return 3
    try:
        transport.unpack(args.input, dest=args.output)
    except Exception as e:  # noqa: BLE001
        print(f"error: {e}", file=sys.stderr)
        return 6
    print(f"unpacked {args.input} → {args.output}", file=sys.stderr)
    return 0


def _run_lock(args) -> int:
    from pathlib import Path

    from . import transport

    if not Path(args.input).exists():
        print(f"error: {args.input}: not found", file=sys.stderr)
        return 3
    pw = _resolve_password(args.password_file, "Passphrase to lock with: ")
    try:
        transport.lock(args.input, password=pw, level=args.level, log_n=args.log_n, dest=args.output)
    except Exception as e:  # noqa: BLE001 — zstd / age failure surfaces here
        print(f"error: {e}", file=sys.stderr)
        return 6
    print(f"locked {args.input} → {args.output}", file=sys.stderr)
    return 0


def _run_unlock(args) -> int:
    from pathlib import Path

    from . import transport

    if not Path(args.input).exists():
        print(f"error: {args.input}: not found", file=sys.stderr)
        return 3
    pw = _resolve_password(args.password_file, "Passphrase to unlock: ")
    try:
        transport.unlock(args.input, password=pw, dest=args.output)
    except Exception as e:  # noqa: BLE001 — wrong password / corrupt file surfaces here
        print(f"error: {e}", file=sys.stderr)
        return 6
    print(f"unlocked {args.input} → {args.output}", file=sys.stderr)
    return 0


def _run_excel(args) -> int:
    """`lat excel <in> <out>` — AGS4 ↔ Excel. Direction is inferred from the
    output extension (`.xlsx` ⇒ export, `.ags` ⇒ import), or forced with
    `--export` / `--import`. Mirrors the Rust `lat excel`."""
    from pathlib import Path

    import laterite

    if not Path(args.input).exists():
        print(f"error: {args.input}: not found", file=sys.stderr)
        return 3
    if args.export:
        export = True
    elif args.import_:
        export = False
    else:
        ext = Path(args.output).suffix.lower()
        if ext == ".xlsx":
            export = True
        elif ext == ".ags":
            export = False
        else:
            print(
                f"error: can't infer direction from output {args.output} — "
                "pass --export (→ .xlsx) or --import (→ .ags)",
                file=sys.stderr,
            )
            return 5
    try:
        if export:
            laterite.to_excel(args.input, args.output)
        else:
            laterite.from_excel(
                args.input, args.output, format_numeric_columns=not args.no_format_numeric
            )
    except Exception as e:  # noqa: BLE001 — parse / xlsx failure surfaces here
        print(f"error: {e}", file=sys.stderr)
        return 6
    print(f"{'exported' if export else 'imported'} {args.input} → {args.output}", file=sys.stderr)
    return 0


def _global_parent() -> argparse.ArgumentParser:
    p = argparse.ArgumentParser(add_help=False)
    p.add_argument("--json", action="store_true")
    p.add_argument("--ndjson", action="store_true")
    p.add_argument("--quiet", action="store_true")
    return p


def _dict_parent() -> argparse.ArgumentParser:
    p = argparse.ArgumentParser(add_help=False)
    p.add_argument("--dict-version", choices=_DICT_CHOICES, default="auto")
    p.add_argument("--dict")
    p.add_argument("--encoding")
    return p


def _with_default_subcommand(argv: list[str]) -> list[str]:
    """Splice `validate` in when the first non-flag token isn't a known subcommand
    — so `lat <file>` (and `lat <file> --json`) route to validate, mirroring the
    Rust argv pre-scan. Global flags are valueless bools, so a leading run of `-`
    tokens is skipped without consuming a value."""
    for i, a in enumerate(argv):
        if a.startswith("-"):
            continue
        if a not in _SUBCOMMANDS:
            return argv[:i] + ["validate"] + argv[i:]
        return argv
    return argv


def main(argv: list[str] | None = None) -> int:
    argv = list(sys.argv[1:] if argv is None else argv)

    if "--readme" in argv or "-h" in argv or "--help" in argv:
        return _print_readme()

    argv = _with_default_subcommand(argv)

    gp, dp = _global_parent(), _dict_parent()
    p = argparse.ArgumentParser(prog="lat", add_help=False)
    sub = p.add_subparsers(dest="cmd")

    pv = sub.add_parser("validate", add_help=False, parents=[gp, dp])
    pv.add_argument("file")
    pv.add_argument("--no-warnings", action="store_true")
    pv.add_argument("--show-fyi", action="store_true")
    pv.add_argument("--check-files", action="store_true")
    pv.add_argument("--out")
    pv.add_argument("--json-out")

    prd = sub.add_parser("read", add_help=False, parents=[gp])
    prd.add_argument("file")
    prd.add_argument("group", nargs="?")
    prd.add_argument("--csv", action="store_true")
    prd.add_argument("--out")

    pf = sub.add_parser("fix", add_help=False, parents=[gp, dp])
    pf.add_argument("file")
    pf.add_argument("--risky", action="store_true")
    pf.add_argument("--in-place", action="store_true")
    pf.add_argument("--fix-out")

    pdf = sub.add_parser("diff", add_help=False, parents=[gp, dp])
    pdf.add_argument("file")
    pdf.add_argument("other")

    pc = sub.add_parser("certify", add_help=False, parents=[gp, dp])
    pc.add_argument("file")
    pc.add_argument("--check-files", action="store_true")
    pc.add_argument("--out")

    sub.add_parser("rules", add_help=False, parents=[gp])

    pk = sub.add_parser("pack", add_help=False, parents=[gp])
    pk.add_argument("input")
    pk.add_argument("output")
    pk.add_argument("--level", type=int, default=9)

    pup = sub.add_parser("unpack", add_help=False, parents=[gp])
    pup.add_argument("input")
    pup.add_argument("output")

    pl = sub.add_parser("lock", add_help=False, parents=[gp])
    pl.add_argument("input")
    pl.add_argument("output")
    pl.add_argument("--level", type=int, default=9)
    pl.add_argument("--log-n", dest="log_n", type=int, default=None)
    pl.add_argument("--password-file", dest="password_file")

    pul = sub.add_parser("unlock", add_help=False, parents=[gp])
    pul.add_argument("input")
    pul.add_argument("output")
    pul.add_argument("--password-file", dest="password_file")

    pe = sub.add_parser("excel", add_help=False, parents=[gp])
    pe.add_argument("input")
    pe.add_argument("output")
    _exdir = pe.add_mutually_exclusive_group()
    _exdir.add_argument("--export", action="store_true")
    _exdir.add_argument("--import", dest="import_", action="store_true")
    pe.add_argument("--no-format-numeric", dest="no_format_numeric", action="store_true")

    try:
        args, extra = p.parse_known_args(argv)
    except SystemExit:
        return 5  # a clap-style "bad args" — argparse would exit 2 otherwise

    if extra:
        print(f"error: unexpected argument {extra[0]!r}", file=sys.stderr)
        return 5
    if getattr(args, "cmd", None) is None:
        print("error: a subcommand or input file is required", file=sys.stderr)
        return 5

    # External --dict is deliberately unimplemented (O-28) — the Rust binary
    # returns BadDict (exit 5) for it too.
    if getattr(args, "dict", None):
        print(
            "error: external --dict override is not implemented; use "
            "--dict-version (4.0.3/4.0.4/4.1/4.1.1/4.2) or omit it",
            file=sys.stderr,
        )
        return 5
    if getattr(args, "json", False) and getattr(args, "ndjson", False):
        print("error: --json and --ndjson are mutually exclusive", file=sys.stderr)
        return 5

    if args.cmd == "rules":
        return _list_rules(args.json)
    if args.cmd == "read":
        return _run_read(args)
    if args.cmd == "pack":
        return _run_pack(args)
    if args.cmd == "unpack":
        return _run_unpack(args)
    if args.cmd == "lock":
        return _run_lock(args)
    if args.cmd == "unlock":
        return _run_unlock(args)
    if args.cmd == "excel":
        return _run_excel(args)
    if args.cmd == "diff":
        return _run_diff(args)
    if args.cmd == "fix":
        return _run_fix(args)
    if args.cmd == "certify":
        return _run_certify(args)
    return _run_validate(args)


if __name__ == "__main__":
    raise SystemExit(main())
