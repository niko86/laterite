#!/usr/bin/env python
"""Emit python-ags4 AGS4.check_file results as rule-keyed JSON.

Contract (consumed by `laterite-ags4-parity`'s `PyOracle`, and through
it by `laterite-ags4-forge` and `laterite-ags4-corpus-qa`):
  stdout : {"AGS Format Rule N": [{"line","group","desc"}, ...], ...}
           (only "AGS Format Rule ..." keys; Metadata/Summary dropped)
  exit 0 : no rule findings
  exit 1 : >= 1 rule finding
  exit 2 : python / parse / encoding error  (stdout = {"error": "..."})
  exit 3 : --selfcheck and python_ags4 not importable

Never prints a bare traceback — the Rust side parses stdout as JSON.

Usage:
  uv run python tools/py_ags4_check_json.py --selfcheck
  uv run python tools/py_ags4_check_json.py [--encoding-fallback] <file.ags>
"""

import argparse
import contextlib
import io
import json
import logging
import os
import pathlib
import sys

# AGS4 edition → python-ags4's bundled standard dictionary filename.
# python-ags4 ships these alongside its package; passing one to
# AGS4.check_file(standard_AGS4_dictionary=...) forces that edition
# instead of the TRAN_AGS auto-pick — the python side of the
# edition-matrix differential (ags4-corpus-qa). Keys match
# lat --dict-version + ags-wiki editions/.
_DICT_BY_EDITION = {
    "4.0.3": "Standard_dictionary_v4_0_3.ags",
    "4.0.4": "Standard_dictionary_v4_0_4.ags",
    "4.1": "Standard_dictionary_v4_1.ags",
    "4.1.1": "Standard_dictionary_v4_1_1.ags",
    "4.2": "Standard_dictionary_v4_2.ags",
}


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("file", nargs="?")
    ap.add_argument("--selfcheck", action="store_true")
    ap.add_argument(
        "--encoding-fallback",
        action="store_true",
        # Accepted but INERT — kept so the Rust parity caller
        # (`laterite-ags4-parity`'s oracle) needs no change. AGS4.check_file
        # opens with errors="replace" (AGS4.py:771) so a UnicodeDecodeError
        # is never raised and there is nothing to "fall back" from. Native
        # python-ags4 deliberately reports cp1252 high bytes as Rule 1 (via
        # the U+FFFD replacement); a cp1252 re-decode here would make this
        # wrapper diverge from python's own default `ags4 check`, i.e. mask
        # real behaviour. The Rust validator mirrors errors="replace"
        # itself (String::from_utf8_lossy). See O-32.
        help="(inert; retained for caller compat — see O-32)",
    )
    ap.add_argument(
        "--dict-version",
        choices=sorted(_DICT_BY_EDITION),
        default=None,
        # Default None = python-ags4's own TRAN_AGS auto-pick (unchanged
        # behaviour — exactly `ags4 check`). An explicit edition forces
        # that bundled standard dictionary, mirroring `lat
        # --dict-version`, so a file can be cross-checked under each
        # edition (the edition-matrix differential — O-30 territory).
        help="force a python-ags4 bundled edition (default: auto)",
    )
    args = ap.parse_args()

    try:
        import python_ags4
        from python_ags4 import AGS4
    except Exception as e:
        print(json.dumps({"error": f"python_ags4 import failed: {e}"}))
        return 3 if args.selfcheck else 2

    if args.selfcheck:
        # Emit the resolved oracle version. The Rust parity harness
        # asserts this equals its pinned EXPECTED_PYAGS4 and warns
        # loudly on drift — the divergence catalogue is encoded against
        # a specific python-ags4 source (see ags-wiki insights
        # oracle-drift-pin). Silent drift was the biggest blind spot.
        print(
            json.dumps(
                {"ok": True, "python_ags4": getattr(python_ags4, "__version__", "?")}
            )
        )
        return 0

    std_dict = None
    if args.dict_version:
        std_dict = str(
            pathlib.Path(python_ags4.__file__).parent
            / _DICT_BY_EDITION[args.dict_version]
        )

    if not args.file:
        print(json.dumps({"error": "no input file"}))
        return 2

    # python-ags4 logs warnings ("DICT group not found", "TRAN_AGS not
    # found …") to stdout/stderr + the logging module. Our stdout
    # contract is JSON-ONLY, so silence + capture everything around the
    # check, then print only the JSON to the real stdout afterwards.
    logging.disable(logging.CRITICAL)
    sink = io.StringIO()
    try:
        # Plain default invocation — exactly what a real `ags4 check
        # file.ags` does. check_file opens utf-8 + errors="replace"
        # (AGS4.py:771): an undecodable cp1252 byte becomes U+FFFD and
        # is_ags_ascii (ord > 255) reports it as "AGS Format Rule 1".
        # We deliberately do NOT probe/force cp1252 — that would mask
        # python's own behaviour. The Rust side now mirrors
        # errors="replace" via from_utf8_lossy, so the two AGREE on a
        # Rule 1 error for these files. (--encoding-fallback is inert;
        # see its help text + O-32.) The dead `except
        # UnicodeDecodeError` retry was removed: errors="replace"
        # never raises it.
        with contextlib.redirect_stdout(sink), contextlib.redirect_stderr(sink):
            errors = AGS4.check_file(
                args.file,
                standard_AGS4_dictionary=std_dict,
                encoding="utf-8",
            )
    except Exception as e:
        print(json.dumps({"error": f"{type(e).__name__}: {e}"}))
        return 2

    # WHICH KEYS COME BACK, and why there are two answers.
    #
    # By default: only `AGS Format Rule N`. That is the parity gate's contract
    # — error-tier rule identity, nothing else (O-45 states it outright: "the
    # parity gate compares only `AGS Format Rule N` keys"). python-ags4 also
    # emits `FYI`, `FYI (Related to Rule N)`, `Warning (...)`, `Summary of
    # data` and `Metadata`, and folding those into the parity verdict would
    # make a tier categorisation look like an error-parity break.
    #
    # `LAT_PY_AGS4_ALL_KEYS=1` keeps everything. A caller comparing FULL TIERS
    # needs it, and filtering one side while leaving the other whole does not
    # under-report — it INVENTS findings. The demo state sweep (#659) did
    # exactly that: python-ags4 raises `FYI (Related to Rule 16)` for a drifted
    # abbreviation description with the same message laterite does, this
    # dropped python's copy, and 145 states were recorded as a laterite-only
    # divergence when both engines agreed. It also hid a real python-only `FYI`
    # in the other direction. A filter nobody can see is a blind spot with a
    # green tick on it (CLAUDE.md, Conventions) — hence the report below, on
    # every run, whichever mode.
    # Report furniture, dropped in BOTH modes. These three sit in the same dict
    # as the findings without being claims about the file's validity, and
    # python-ags4 itself prints them ahead of the error list rather than in it
    # (`AGS4.py::write_error_report`). Carrying them into a comparison would
    # report a python-only difference on every file ever checked.
    # `Metadata` is the run's own header (file name, hash, timestamp) and
    # `Summary of data` a group inventory, so both read as furniture from the
    # name alone. `General` does not, and is the one worth justifying: every
    # site that raises it (`check.py`'s AGS3 caveat, `AGS4.py`'s extended-ASCII
    # note and its "could not complete validation") emits prose ABOUT the run
    # BESIDE the real finding, never in place of one — the abort case pairs it
    # with a `Validator Process Error` key, which IS a finding and is kept.
    NOT_FINDINGS = {"General", "Metadata", "Summary of data"}
    all_keys = os.environ.get("LAT_PY_AGS4_ALL_KEYS") == "1"
    if all_keys:
        rules = {
            k: v
            for k, v in errors.items()
            if isinstance(k, str) and k not in NOT_FINDINGS
        }
        dropped: list[str] = [k for k in errors if k in NOT_FINDINGS]
    else:
        rules = {
            k: v
            for k, v in errors.items()
            if isinstance(k, str) and k.startswith("AGS Format Rule ")
        }
        dropped = [
            k
            for k in errors
            if isinstance(k, str) and not k.startswith("AGS Format Rule ")
        ]
    # Two reasons a key is dropped, reported separately because they are not
    # equally recoverable: furniture goes in BOTH modes and no flag brings it
    # back, and saying otherwise would be a gate misreporting its own scope.
    furniture = sorted(k for k in dropped if k in NOT_FINDINGS)
    filtered = sorted(k for k in dropped if k not in NOT_FINDINGS)
    report = f"py_ags4_check_json: {len(rules)} key(s) returned"
    if furniture:
        report += (
            f"; {len(furniture)} dropped as report furniture "
            f"({', '.join(furniture)}), in either mode"
        )
    if filtered:
        report += (
            f"; {len(filtered)} finding key(s) dropped by the error-tier "
            f"filter ({', '.join(filtered)}) — set LAT_PY_AGS4_ALL_KEYS=1 "
            f"to keep them"
        )
    print(report, file=sys.stderr)
    # `line` may be an int or the string "-"; left as-is — the Rust
    # side compares rule-key *presence* only, never line values.
    print(json.dumps(rules, ensure_ascii=False))
    return 1 if any(rules.values()) else 0


if __name__ == "__main__":
    sys.exit(main())
