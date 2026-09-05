"""One loader for the repo's script tools — because `tools/` is not a package.

Every gate and generator here is a standalone script, importable only by path,
and until #925 each test file carried its own hand-written
`importlib.util.spec_from_file_location` shim — 33 copies, each guessing
independently whether to register the module in `sys.modules`. This is the one
copy. It preserves the dominant shim's exact semantics:

* **Fresh execution per call** — no cache. Each test file gets its own module
  object, exactly as the per-file shims did, so module-level mutation in one
  file cannot leak into another.
* **`sys.modules` registration** — so a tool importing a sibling by bare name
  (`engine_cut` → `release_status`, `librarian` → `refs`) binds to the
  instance the SAME test file just loaded, not a second private copy with its
  own class objects (`isinstance`/`dataclasses` identity would silently break
  across two copies).
* **The tool's directory goes on `sys.path`** — the other half of sibling
  imports, previously done ad hoc by the files that needed it.
"""

from __future__ import annotations

import importlib.util
import sys
from pathlib import Path
from typing import TYPE_CHECKING

if TYPE_CHECKING:
    from types import ModuleType

REPO = Path(__file__).resolve().parents[1]

#: Where a tool may live, searched in order. First match wins; two tools with
#: one name across these directories would be a repo bug this refuses loudly.
#: The last two are narrower homes with the same by-path problem: the xcheck
#: gates import a sibling (`emit_cli`), and the mkdocs hooks are scripts mkdocs
#: loads by path — neither directory is a package either.
_DIRS = (
    REPO / "tools",
    REPO / "tools" / "release",
    REPO / "tools" / "xcheck",
    REPO / "ags-wiki" / ".bootstrap",
    REPO / "web" / "docs-site" / "hooks",
)


def load_tool(name: str) -> ModuleType:
    """Import `<name>.py` from the repo's script directories as a module.

    `name` is the MODULE name, so a hyphenated script file
    (`bench-vs-python-ags4.py`) is found by its underscore spelling — the
    hyphens are why it needs a path-loader at all.
    """
    stems = {name, name.replace("_", "-")}
    hits = [
        d / f"{stem}.py"
        for d in _DIRS
        for stem in sorted(stems)
        if (d / f"{stem}.py").is_file()
    ]
    if not hits:
        msg = f"no {name}.py under any of: " + ", ".join(str(d) for d in _DIRS)
        raise FileNotFoundError(msg)
    if len(hits) > 1:
        msg = f"{name}.py is ambiguous: " + ", ".join(str(p) for p in hits)
        raise RuntimeError(msg)
    path = hits[0]
    if str(path.parent) not in sys.path:
        sys.path.insert(0, str(path.parent))
    spec = importlib.util.spec_from_file_location(name, path)
    assert spec and spec.loader
    mod = importlib.util.module_from_spec(spec)
    sys.modules[name] = mod
    spec.loader.exec_module(mod)
    return mod


def default_crate(rs: ModuleType):
    """The all-quiet engine-crate row the release-status test files vary with
    `dataclasses.replace(DEFAULT, **kw)`.

    ONE default instead of the two hand-mirrored 17-key dicts it replaces —
    #802 added three keys and had to thread them through both fixtures by
    hand; a new `CrateStatus` field now fails construction HERE, once.

    Takes the loaded module rather than importing it: each test file loads
    its own fresh `release_status`, and an instance built from another
    file's copy would carry a different class object.
    """
    return rs.CrateStatus(
        crate="laterite-ags4-core",
        version="0.12.0",
        last_stamp="abc1234 2026-08-29 release: x",
        registry_state="ok",
        registry_latest="0.12.0",
        api_added=0,
        api_removed=0,
        api_removed_names=[],
        verdict="none",
        tier="engine",
        published_live="0.12.0",
        delta_baseline="publish 0.12.0",
        code_changed=False,
        deps_behind=[],
        part_required="none",
        cut_action="none",
        cut_why="",
    )


def report_of(rs: ModuleType, *crates):
    """A `collect()`-shaped report around `crates`, quiet product tier."""
    return rs.Report(
        engine_crates=list(crates),
        product=rs.ProductStatus(
            version="0.12.0", last_stamp="abc 2026-08-29 x", verdict="none"
        ),
        changelog_unreleased={},
    )
