"""Render the root CHANGELOG.md into the docs as a versioned Changelog page (#372).

The release notes were invisible on the docs site — CHANGELOG.md lived only at
the repo root, in no nav and no include. This single-sources it into the docs
(the same file `bump-version.sh` rolls) and stamps the current shipped version
from `packages/laterite/pyproject.toml`. Both are **derived at build time**, so
the page can't drift: a merged release deploys on the next master push and the
docs show the new version + its notes at `/laterite/docs/` with nothing to
hand-maintain here.
"""

from __future__ import annotations

import re
import tomllib
from pathlib import Path

import mkdocs_gen_files

_REPO = Path(__file__).resolve().parents[3]
_MIRROR_BLOB = "https://github.com/niko86/laterite/blob/master"

_version = tomllib.loads(
    (_REPO / "packages" / "laterite" / "pyproject.toml").read_text()
)["project"]["version"]

_changelog = (_REPO / "CHANGELOG.md").read_text()

# Drop the source file's own "# Changelog" H1 — the page supplies its own title
# plus the version banner; keep everything else (intro + every version + the
# reference-link definitions the version headers resolve against).
_body = _changelog.split("\n", 1)[1] if _changelog.startswith("# ") else _changelog

# Repo-relative doc links (e.g. `docs/parity-coverage-map.md`) resolve at the
# repo root but not inside the docs tree — point them at the public mirror so
# the strict build's link check passes. http(s) + in-page anchors are left alone.
_body = re.sub(
    r"\]\((?!https?:|#)([^)]+)\)",
    lambda m: f"]({_MIRROR_BLOB}/{m.group(1)})",
    _body,
)

_page = f"""# Changelog

Current release: **laterite {_version}** — `pip install laterite` ·
`npm install laterite`. The same numbered version ships the Python wheel, the
`lat-check` CLI, the npm package, and (as `laterite_ags4`) the DuckDB extension.
{_body}"""

with mkdocs_gen_files.open("reference/changelog.md", "w") as fd:
    fd.write(_page)
