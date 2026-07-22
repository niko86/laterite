"""Guard: every path in ci.yml's `changes` path-filter must resolve to a file
that exists in THIS tree — unless it's explicitly allow-listed as absent.

Why this exists (the CI/CD audit's filter<->gate finding): the public mirror
strips the private `tests/` faithfulness suite, but the strip does NOT prune the
path filters that referenced it. So a filter path can outlive the file or gate
it triggered, silently firing the heavy required `python` job for a gate that no
longer runs here — a README badge bump rebuilding the whole wheel was the visible
symptom. Asserting that every filter path still resolves catches that drift the
day a filter edit or a mirror strip introduces it, not months later by hand.

The one documented exception is `tests/**` (KNOWN_ABSENT): the AGS4
dictionary-faithfulness suite is private-only, so the public mirror ships without
it and the glob matches nothing BY DESIGN. Any OTHER unresolved path is drift and
fails here. This test carries no import of `laterite`, so it runs identically on
the public mirror and the private tree.
"""

from __future__ import annotations

import re
from pathlib import Path

import pytest

# tests/ -> laterite/ -> packages/ -> repo root
REPO_ROOT = Path(__file__).resolve().parents[3]
CI_YML = REPO_ROOT / ".github" / "workflows" / "ci.yml"

# Paths a tree may legitimately lack (documented divergence, not drift): the
# private root `tests/` suite is stripped from the public mirror.
KNOWN_ABSENT = {"tests/**"}


def _filter_paths() -> list[tuple[str, str]]:
    """(filter_name, path) for every entry in the changes-job filters.

    Parsed textually so the guard needs no YAML dependency: the block is
    `<name>:` headers followed by `- '<path>'  # comment` lines, sitting between
    `filters: |` and the next top-level job.
    """
    text = CI_YML.read_text(encoding="utf-8")
    m = re.search(r"\n {10}filters: \|\n(.*?)\n {2}\w", text, re.S)
    assert m, "could not locate the `filters:` block in ci.yml"
    out: list[tuple[str, str]] = []
    current: str | None = None
    for line in m.group(1).splitlines():
        s = line.strip()
        if not s or s.startswith("#"):
            continue
        if s.endswith(":") and not s.startswith("- "):
            current = s[:-1]
            continue
        m2 = re.match(r"- ['\"]?([^'\"#]+?)['\"]?(\s+#.*)?$", s)
        if m2 and current:
            out.append((current, m2.group(1).strip()))
    assert out, "parsed zero filter paths — the parser or the filter shape changed"
    return out


@pytest.mark.parametrize(("filter_name", "path"), _filter_paths())
def test_ci_filter_path_resolves(filter_name: str, path: str) -> None:
    if path.startswith("!") or path in KNOWN_ABSENT:
        return  # an exclusion, or documented-absent — nothing to resolve
    if any(ch in path for ch in "*?["):
        assert any(REPO_ROOT.glob(path)), (
            f"`{filter_name}` filter glob {path!r} matches no file in this tree — "
            "stale trigger (fix the filter, or add to KNOWN_ABSENT if private)"
        )
    else:
        assert (REPO_ROOT / path).exists(), (
            f"`{filter_name}` filter path {path!r} does not exist in this tree — "
            "stale trigger (fix the filter, or add to KNOWN_ABSENT if private)"
        )
