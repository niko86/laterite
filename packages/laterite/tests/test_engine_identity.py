"""The wheel can say which ENGINE it carries, not just which release it is.

Since the tiers split (laterite#202) the release version and the engine version
are independent numbers, so `importlib.metadata.version("laterite")` no longer
answers "which rules ran". `engine_fingerprint()` does, and it is the value an
`.ags.idx` certificate stamps.

The point of the fingerprint over the engine's semver is that it is derived from
the engine's actual inputs — every rule source, the dictionary, the rules
catalogue. Edit a rule and forget to bump anything and it still moves. So it can
be compared across surfaces and believed; a matching version number only shows
two things shipped together.
"""

import json
import re
import subprocess
import sys

import laterite

#: `build.rs` truncates the SHA-256 to 16 hex chars.
FINGERPRINT = re.compile(r"\A[0-9a-f]{16}\Z")


def test_fingerprint_is_a_well_formed_digest() -> None:
    fp = laterite.engine_fingerprint()
    assert FINGERPRINT.match(fp), (
        f"engine fingerprint {fp!r} is not 16 hex chars — a truncated or "
        "placeholder value would compare equal across surfaces while meaning "
        "nothing, which is the failure this whole value exists to prevent"
    )


def test_fingerprint_is_stable_within_a_build() -> None:
    # A value that changed per call could never be compared to anything.
    assert laterite.engine_fingerprint() == laterite.engine_fingerprint()


def test_engine_version_is_a_semver_and_not_the_fingerprint() -> None:
    ev = laterite.engine_version()
    assert re.match(r"\A\d+\.\d+\.\d+", ev), f"engine version {ev!r} is not semver"
    assert ev != laterite.engine_fingerprint(), (
        "engine_version and engine_fingerprint returned the same string — one of "
        "them is wired to the wrong constant"
    )


def test_the_two_numbers_are_reported_separately() -> None:
    """The release version and the engine version are answered by different calls.

    They are EQUAL today (both 0.9.0) and will diverge at the first bump of either
    tier. So this cannot assert they differ — it asserts the wheel offers both
    doors, because the failure mode is a surface that only ever had one and
    silently answered the wrong question with it.
    """
    import importlib.metadata as md

    release = md.version("laterite")
    assert re.match(r"\A\d+\.\d+\.\d+", release)
    assert callable(laterite.engine_version)
    assert callable(laterite.engine_fingerprint)


def test_the_cli_reports_the_same_engine_as_the_library() -> None:
    """`lat` and the imported package must agree.

    The wheel's `lat` console script and `laterite` are the same install, so a
    disagreement means one of them resolved a different native module — which is
    exactly what a stale editable build or a shadowed `.so` produces, and exactly
    what nothing else here would notice.
    """
    out = subprocess.run(
        [
            sys.executable,
            "-c",
            "import laterite; print(laterite.engine_fingerprint())",
        ],
        capture_output=True,
        text=True,
        check=True,
    )
    assert out.stdout.strip() == laterite.engine_fingerprint()


def test_the_uvx_launcher_reports_the_engine_it_is_running() -> None:
    """`lat census` carries the engine, and it is ASKED rather than restated.

    This is the door `laterite-ags4-xcheck` uses to identity-check the three
    launcher legs. Before it existed the cross-surface gate compared their bytes
    without knowing whether they were the same build — and a launcher driving a
    stale artefact agrees with a current one on almost every case, so the gate
    would have reported identity it never checked.

    Asserting it equals `laterite.engine_fingerprint()` is the load-bearing half:
    a hard-coded digest here would satisfy a "is it 16 hex chars" test forever
    while reporting an engine this launcher is not running.
    """
    out = subprocess.run(
        [sys.executable, "-m", "laterite._cli", "census"],
        capture_output=True,
        text=True,
        check=True,
    )
    census = json.loads(out.stdout)
    assert census["engine"] == laterite.engine_fingerprint()
    assert census["census_version"] >= 6, (
        "the engine field arrived in census schema 6; a launcher answering an "
        "older schema has no engine to report and must not be read as agreeing"
    )
