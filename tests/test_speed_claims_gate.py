"""The speed-claims gate's parsing seams (#826).

The gate grew a memory half when the close-out promoted the peak-RSS numbers
into the README: the same band-membership predicate, applied to `~N× less
memory` claims against the peak-RSS tables. These tests pin the two seams that
keep the halves apart — the MEM_MARKER split in `read_bands` (the time and
memory tables name the same APIs) and the memory claim pattern.
"""

from __future__ import annotations

from _tools import load_tool

gate = load_tool("check_speed_claims")

_TABLES = """\
**Validation**

| File | `python-ags4 check_file` | `laterite.validate` | speedup |
|---:|---:|---:|:---:|
| 4.9 MB | 1.5 s | 50 ms | **30.0×** |
| 549.7 MB | 70.0 s | 5.4 s | **12.9×** |

**Validation — peak RSS**

| File | `python-ags4 check_file` peak RSS | `laterite.validate` peak RSS | ratio |
|---:|---:|---:|:---:|
| 4.9 MB | 170 MB | 94 MB | **1.81×** |
| 275.5 MB | 2669 MB | 1116 MB | **2.39×** |
"""


def test_time_bands_skip_the_memory_table() -> None:
    bands = gate.read_bands(_TABLES)
    # Only the time rungs — 1.81/2.39 from the peak-RSS table must not leak
    # into the time band (they'd silently widen it downward).
    assert bands["validation"] == (12.9, 30.0)


def test_memory_bands_read_only_the_peak_rss_table() -> None:
    bands = gate.read_bands(_TABLES, memory=True)
    assert bands["validation"] == (1.81, 2.39)


def test_memory_claim_pattern_matches_both_spellings() -> None:
    assert gate.CLAIM_MEM_RE.search("~2× less memory than python-ags4")
    assert gate.CLAIM_MEM_RE.search("1.8x less peak memory than python-ags4")
    # A speed claim is not a memory claim and vice versa.
    assert not gate.CLAIM_MEM_RE.search("~3× faster than python-ags4")
    assert not gate.CLAIM_RE.search("~2× less memory than python-ags4")
