"""The vendored AGS4 dictionaries must still equal the source they claim (#558).

`rust-packages/laterite-ags4-validator/data/PROVENANCE.md` makes a precise,
falsifiable claim: the five `Standard_dictionary_v4_*.ags` were

> Retrieved 2026-05-16 … from the bundled copies shipped in the **`python-ags4`**
> package (v1.2.0) … These five editions are exactly the set `python-ags4` itself
> ships (its `STANDARD_DICT_FILES` map).

Every dictionary in this product descends from those five files: `gen_dictionary.py`
projects them into `ags_dictionary.json`, and the validator, the wasm build, the
typed-graph codegen and the web all read that union. The five files are the root.

**Nothing checked the claim.** `tests/test_dictionary_faithful.py` re-runs the
generator and asserts the committed union still equals the projection — so it
proves the union is faithful to *whatever the files currently say*, not that the
files are faithful to their stated source. Hand-edit a `.ags` and regenerate and
every gate goes green: the derivation is guarded, the authority it derives from is
not. That is #549's Shape 1 at the root of the dictionary — the gate enforces a
PROXY (the projection) for the promise (this is the AGS standard's dictionary),
and nothing compares proxy back to promise.

The refresh path is `tools/sync-standard-dicts.sh` — `cp -f` from the installed
package, then "update the retrieval date in PROVENANCE.md" by hand. A human
reading a note inside a file they are already not reading. Same shape as
`tools/vendor/laterite-duckdb-functions.json`.

This is cheap to close because the source is **already installed**: `python-ags4`
is a declared dev dependency (`pyproject.toml`, `python-ags4==1.2.0`) — it is the
parity oracle. So the claim can be checked offline, with no clone and no network,
by comparing our copies to the ones sitting in the venv.

What this does NOT prove: that python-ags4's dictionaries match the AGS standard
PDF. Our dictionary comes from our own parity oracle, so the parity suite is
structurally incapable of catching a shared divergence — both sides read the same
bytes. PROVENANCE.md argues the source is authoritative *because* python-ags4 is
the AGS Data Format Working Group's own reference implementation, which is a
reasonable position, and it is the position — not a proof. This test pins us to
the source we chose. It does not audit the source.
"""

from __future__ import annotations

import importlib.metadata
import json
import re
import tomllib
from pathlib import Path

import pytest
from python_ags4.check import LATEST_DICT_VERSION, STANDARD_DICT_FILES

_REPO = Path(__file__).resolve().parents[1]
_VENDORED = _REPO / "rust-packages/laterite-ags4-validator/data"
_UNION = _REPO / "rust-packages/laterite-ags4-reference/data/ags_dictionary.json"
_PROVENANCE = _VENDORED / "PROVENANCE.md"


def _upstream_dir() -> Path:
    import python_ags4

    return Path(python_ags4.__file__).resolve().parent


def _upstream_filenames() -> set[str]:
    """The distinct dictionary files upstream ships.

    `STANDARD_DICT_FILES` maps six *edition strings* onto five files — '4.0' and
    '4.0.3' share `Standard_dictionary_v4_0_3.ags`. It is the file set that is
    'exactly the set python-ags4 ships', which is what PROVENANCE.md claims.
    """
    return set(STANDARD_DICT_FILES.values())


def test_the_upstream_source_is_actually_present() -> None:
    """Guard the guard, first.

    Every assertion below compares against the installed package. If it were
    missing, an over-forgiving test would skip and report green — a vendored tree
    that has drifted and a vendored tree nobody checked look identical from the
    outside, which is the failure this whole file exists to prevent. python-ags4
    is a declared dev dependency; its absence is a broken environment, not a
    reason to have no opinion.
    """
    up = _upstream_dir()
    assert up.is_dir(), f"python_ags4 package dir not found at {up}"
    files = _upstream_filenames()
    assert len(files) == 5, (
        f"expected 5 distinct upstream dictionaries, got {sorted(files)}"
    )
    for name in files:
        assert (up / name).is_file(), (
            f"upstream ships {name} in its map but the file is absent"
        )


def test_the_vendored_set_is_exactly_what_upstream_ships() -> None:
    """The file *set*, before any byte comparison.

    Catches both directions: an edition published upstream that we never picked
    up, and a file in our data dir that upstream does not ship — i.e. one we
    invented. A byte-comparison loop over our own filenames would be blind to
    both, because it would only ever compare the files we already have.
    """
    ours = {p.name for p in _VENDORED.glob("Standard_dictionary_v4_*.ags")}
    theirs = _upstream_filenames()
    assert ours, f"no vendored dictionaries found in {_VENDORED} — the scan is broken"
    extra = sorted(ours - theirs)
    missing = sorted(theirs - ours)
    assert not extra, (
        f"we vendor dictionaries upstream does not ship: {extra}\n"
        "PROVENANCE.md claims these are exactly python-ags4's set."
    )
    assert not missing, (
        f"upstream ships dictionaries we do not vendor: {missing}\n"
        "A new AGS4 edition may have landed. Run tools/sync-standard-dicts.sh, "
        "regenerate the union, and update PROVENANCE.md's retrieval date."
    )


def test_the_vendored_dictionaries_are_byte_identical_to_their_stated_source() -> None:
    """The claim itself, byte for byte."""
    up = _upstream_dir()
    drifted: list[str] = []
    checked = 0
    for name in sorted(_upstream_filenames()):
        ours = (_VENDORED / name).read_bytes()
        theirs = (up / name).read_bytes()
        checked += 1
        if ours != theirs:
            drifted.append(
                f"  {name}  (ours {len(ours)} bytes, upstream {len(theirs)} bytes)"
            )
    assert checked == 5, f"compared {checked} dictionaries, expected 5"
    if drifted:
        pytest.fail(
            "vendored dictionaries no longer match the source PROVENANCE.md names:\n"
            + "\n".join(drifted)
            + "\n\nEvery dictionary in this product is projected from these files. If the "
            "drift is deliberate, PROVENANCE.md's claim is no longer true and must be "
            "rewritten to say what these files actually are. If it is not deliberate, "
            "run tools/sync-standard-dicts.sh and regenerate the union."
        )


def test_the_fallback_edition_mirrors_upstreams_latest() -> None:
    """`FALLBACK_EDITION` is a hand-copy of upstream's `LATEST_DICT_VERSION`.

    This one is not documentation — it is *behaviour*. It decides which edition
    validates a file whose `TRAN_AGS` is absent or unparsable, and it was set to
    python-ags4's value deliberately, so dogfood parity reflects real defects
    rather than fallback artefacts (gen_dictionary.py says so at the constant).

    That rationale holds only while the two agree. If upstream bumps its default
    and ours stays, the two validators quietly disagree about every untagged file
    — and the stated reason for the constant's value silently becomes false, with
    the comment still asserting it.
    """
    fallback = json.loads(_UNION.read_text(encoding="utf-8"))["fallback_edition"]
    assert fallback == LATEST_DICT_VERSION, (
        f"the union's fallback_edition is {fallback!r}, python-ags4's "
        f"LATEST_DICT_VERSION is {LATEST_DICT_VERSION!r}.\n"
        "These are deliberately the same value (tools/gen_dictionary.py, "
        "FALLBACK_EDITION). If upstream moved, decide whether to follow — and if "
        "we deliberately diverge, rewrite that comment, because it currently "
        "claims we match."
    )


def test_every_stated_python_ags4_version_matches_the_installed_one() -> None:
    """Four hand-written `1.2.0`s naming one upstream version.

    They agree today by coincidence — each was typed by a person who knew the
    value at the time, and nothing compares them:

      pyproject.toml               `python-ags4==1.2.0`     what actually installs
      .github/workflows/parity.yml `PYTHON_AGS4_VERSION`    the suite tag cloned
      parity-known-failures.json   `python_ags4_version`    the oracle the set was measured against
      data/PROVENANCE.md           "(v1.2.0)"               where the dictionaries came from

    The installed distribution is the fact; the rest are claims about it. Bumping
    the oracle must move all four together, as a visible act — a suite cloned at
    one tag and a library installed at another would produce a parity verdict
    about neither.
    """
    real = importlib.metadata.version("python-ags4")

    claims: dict[str, str | None] = {}

    pyproj = tomllib.loads((_REPO / "pyproject.toml").read_text(encoding="utf-8"))
    pins = [
        d
        for group in pyproj.get("dependency-groups", {}).values()
        for d in group
        if isinstance(d, str) and d.replace("_", "-").startswith("python-ags4")
    ]
    m = re.search(r"==\s*([\w.]+)", pins[0]) if pins else None
    claims["pyproject.toml (python-ags4==)"] = m.group(1) if m else None

    workflow = (_REPO / ".github/workflows/parity.yml").read_text(encoding="utf-8")
    m = re.search(r'^\s*PYTHON_AGS4_VERSION:\s*"?([\w.]+)"?', workflow, re.M)
    claims[".github/workflows/parity.yml (PYTHON_AGS4_VERSION)"] = (
        m.group(1) if m else None
    )

    fixture = json.loads(
        (_REPO / "parity-known-failures.json").read_text(encoding="utf-8")
    )
    claims["parity-known-failures.json (python_ags4_version)"] = fixture.get(
        "python_ags4_version"
    )

    m = re.search(
        r"`python-ags4`\*\* package \(v([\w.]+)\)",
        _PROVENANCE.read_text(encoding="utf-8"),
    )
    claims["data/PROVENANCE.md (retrieval source)"] = m.group(1) if m else None

    unparsed = sorted(k for k, v in claims.items() if v is None)
    assert not unparsed, (
        "could not find the python-ags4 version in:\n"
        + "\n".join(f"  {k}" for k in unparsed)
        + "\n\nThe scanner is broken, not the tree — and a scan that finds nothing "
        "passes every comparison below. Fix the pattern."
    )
    assert len(claims) == 4, f"expected 4 stated versions, scanned {len(claims)}"

    wrong = sorted(f"  {k}: says {v!r}" for k, v in claims.items() if v != real)
    if wrong:
        pytest.fail(
            f"python-ags4 {real} is installed; these disagree:\n"
            + "\n".join(wrong)
            + "\n\nBumping the oracle moves all four. A suite cloned at one tag against a "
            "library installed at another is a verdict about neither."
        )
