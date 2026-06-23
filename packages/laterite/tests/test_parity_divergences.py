"""Live compat-vs-python-ags4 divergence lock (#190).

The `parity` CI job already proves the BROAD contract — compat satisfies
python-ags4 1.2.0's own test suite (122 pass). This is the complementary,
TARGETED half: a per-PR check that the *specific, intentional* divergences
(the O-N catalogue) are still exactly as documented.

Each fixture is a curated probe that reproduces ONE divergence; we run it
through both `laterite.compat` (our engine) and the REAL `python_ags4`
library (pinned 1.2.0, a dev dep), and assert the (compat, python) rule-family
verdict pair matches the frozen expectation. It is deliberately small and
high-signal — NOT a corpus of random files (those would just re-confirm the
agreement the 122 oracle already covers and churn on every count tweak).

A failure means either a real regression in our engine (compat drifted on a
documented edge) or that a divergence has changed/closed — both warrant a look
(and, if intentional, an OBSERVATIONS + expectation update). Regenerate the
map with `LATERITE_REGEN_DIVERGENCES=1 pytest -s ...` and copy the printed dict.

python-ags4 is pinned, so its side is frozen; a mismatch there flags an
unexpected upstream/version change.
"""
import json
import os
import re
from pathlib import Path

import pytest

pytest.importorskip("python_ags4", reason="parity oracle needs the python-ags4 dev dep")
import laterite.compat as compat  # noqa: E402
from python_ags4 import AGS4  # noqa: E402

PROBES = Path(__file__).resolve().parents[3] / "ags-wiki" / ".bootstrap" / "probes"
_RULE = re.compile(r"^AGS Format Rule (\S+)$")

# probe -> the documented divergence + the frozen (compat, python) verdicts.
# Each entry is ONE O-N; the verdict is the {rule: count} family map.
EXPECTED = {
    "probe-o8-dup-heading.ags": {
        "note": "O-8: duplicate HEADING. compat reports it cleanly; python-ags4 "
                "renames + flags it as a non-standard heading (Rule 9/18).",
        "compat": {"7": 1},
        "python": {"7": 2, "9": 1, "18": 1},
    },
    "probe-o27-file-ondisk.ags": {
        "note": "O-27: Rule 20 on-disk FILE check — different count (data-level "
                "vs on-disk interpretation).",
        "compat": {"20": 1},
        "python": {"20": 2},
    },
    "probe-rule6-embedded-cr.ags": {
        "note": "O-2: a lone embedded CR. python-ags4's universal-newline split "
                "turns it into Rule 2a+3+5; laterite attributes it to Rule 6 "
                "(which python-ags4's rule_6 leaves a no-op).",
        "compat": {"6": 1},
        "python": {"2a": 1, "3": 1, "5": 2},
    },
    "probe-rule19-digit.ags": {
        "note": "Rule 19 name-format: laterite flags a leading-digit GROUP/HEADING "
                "name (Rule 19/19b) that python-ags4 does not.",
        "compat": {"7": 1, "9": 1, "10b": 4, "10c": 1, "18": 1, "19": 1, "19b": 1},
        "python": {"7": 1, "9": 1, "10b": 4, "10c": 1, "18": 1},
    },
    "probe-o42-edition-4-0-alias.ags": {
        "note": "O-42: TRAN_AGS=4.0 file using 4.0.4 vocabulary. python-ags4's "
                "stale 4.0->4.0.3 alias over-reports (false Rule 9 on SAMP_RECL, "
                "Rule 10c on PMTL, Rule 18); laterite resolves 4.0->4.0.4 and omits them.",
        "compat": {"7": 2, "10a": 1, "10b": 5, "15": 1, "16": 1, "17": 1},
        "python": {"7": 3, "9": 1, "10a": 1, "10b": 5, "10c": 1, "15": 1,
                   "16": 1, "17": 1, "18": 1},
    },
}


def _families(errs):
    return {
        _RULE.match(k).group(1): len(v)
        for k, v in errs.items()
        if _RULE.match(k) and isinstance(v, list)
    }


def _verdict(fn, path):
    try:
        return _families(fn(str(path)))
    except Exception as exc:  # a parser hard-fail is itself a verdict
        return {"__error__": type(exc).__name__}


def test_regen_divergence_map():
    """Opt-in helper: LATERITE_REGEN_DIVERGENCES=1 prints the current map."""
    if not os.environ.get("LATERITE_REGEN_DIVERGENCES"):
        pytest.skip("set LATERITE_REGEN_DIVERGENCES=1 to regenerate")
    out = {}
    for name in EXPECTED:
        p = PROBES / name
        out[name] = {"compat": _verdict(compat.check_file, p),
                     "python": _verdict(AGS4.check_file, p)}
    print(json.dumps(out, indent=2, sort_keys=True))


@pytest.mark.parametrize("name", sorted(EXPECTED), ids=lambda n: n.removesuffix(".ags"))
def test_documented_divergence_holds(name):
    spec = EXPECTED[name]
    probe = PROBES / name
    assert probe.exists(), f"missing probe fixture {probe}"
    got_compat = _verdict(compat.check_file, probe)
    got_python = _verdict(AGS4.check_file, probe)
    assert got_compat == spec["compat"], (
        f"{name}: compat verdict drifted ({spec['note']})\n"
        f"  expected {spec['compat']}\n  got      {got_compat}"
    )
    assert got_python == spec["python"], (
        f"{name}: python-ags4 verdict changed — version drift? ({spec['note']})\n"
        f"  expected {spec['python']}\n  got      {got_python}"
    )
    # The divergence must still BE a divergence (guards against silent closure).
    assert got_compat != got_python, f"{name}: divergence closed — update EXPECTED + OBSERVATIONS"
