#!/usr/bin/env python3
"""The duckdb surface runner — read/parse-agreement (laterite-dev#458).

**Repointed from findings-agreement (was laterite-dev#169 5a).** The `laterite_ags4` DuckDB
extension became a *read-only reader* in laterite-dev#446 — `validate_ags()`/`certify_ags()`
were removed (the surface is now 8 read functions). A read-only reader emits no
validation findings, so it can no longer take part in the cross-surface
*findings*-agreement harness the other five surfaces run. This runner instead
asserts the thing the extension actually does: does its `read_ags()` parse the
**same rows** as the canonical in-workspace engine every surface wraps?

The agreement metric is the **content-addressed key set** (`_id`/`_parent_id`,
the deterministic keychain of laterite-dev#303/laterite-dev#144): `_id = UUIDv8(SHA-256(spec key-chain))`,
`_parent_id` the same over the parent's chain. These are already golden-tested
byte-identical across rust/python/node/wasm/duckdb, and — unlike the typed data
columns (`2DP`->DOUBLE, ...) — they carry NO float/temporal formatting that would
drift between a SQL result and a Rust reference. So "duckdb read the same rows"
reduces to "duckdb produced the same `(_id, _parent_id)` set per group", which
the rust `duckdb-parse-check` bin verifies against a core reference.

This runner emits `duckdb-parse.json` (schema 2, `kind: "parse-agreement"`) —
deliberately NOT `duckdb.json`, so the findings comparator (which globs
`*.json` as finding-sets) never mis-reads it; it skips `*-parse.json`.

The extension is built for an EXACT DuckDB version (unstable C API), so this must
run under an ABI-matched duckdb — the ext's pinned venv is guaranteed compatible:

    output/laterite-duckdb-ext/configure/venv/bin/python3 \
        tools/compliance/emit_duckdb.py --fixtures <dir> --out <dir>

Self-skips (writes no file -> parse-check shows nothing to check) if the ext
isn't built or duckdb can't load it.
"""

import argparse
import json
import sys
from pathlib import Path
from typing import Any

REPO_ROOT = Path(__file__).resolve().parents[2]

#: Where a locally-built extension lands, for `--ext` without an argument path.
#: `output/` is gitignored working space; the extension is built from its OWN
#: repo (niko86/laterite-duckdb) and copied here, so this path is a convention
#: rather than something this repo produces.
DEV_EXT = (
    REPO_ROOT
    / "output"
    / "laterite-duckdb-ext"
    / "build"
    / "release"
    / "laterite_ags4.duckdb_extension"
)


def _first_line(exc: Exception) -> str:
    """First line of an exception message (empty-safe) — the diagnostic kept for
    a read error, without the multi-line SQL context DuckDB appends."""
    lines = str(exc).splitlines()
    return lines[0] if lines else exc.__class__.__name__


# `con` is `Any`, not `duckdb.DuckDBPyConnection`: duckdb is imported INSIDE
# `run()` so this script can self-skip when it is absent, and a module-level
# annotation would reintroduce the import it exists to avoid.
def group_key_rows(con: Any, path: str, group: str) -> list:
    """The sorted `[_id, _parent_id]` rows of one group via `read_ags`.

    `_parent_id` is NULL for a root group -> serialized as JSON null, matching
    the rust reference's `Option<String>::None`. Sorted so the list is a
    canonical set representation both sides compare by value."""
    esc_p = path.replace("'", "''")
    esc_g = group.replace("'", "''")
    rows = con.execute(
        f"SELECT _id, _parent_id FROM read_ags('{esc_p}', '{esc_g}')"
    ).fetchall()
    return sorted([r[0], r[1]] for r in rows)


def run(fixtures: list[str], ext_path: Path | None) -> dict:
    import duckdb

    if ext_path is None:
        # The PUBLISHED artefact — what `INSTALL laterite_ags4 FROM community`
        # gives a user. Checking that is the point; a locally-built .duckdb
        # _extension can agree with the engine while the published one does not.
        con = duckdb.connect()
        con.execute("INSTALL laterite_ags4 FROM community")
        con.execute("LOAD laterite_ags4")
    else:
        # A path is the DEV door: an extension built beside the checkout, which
        # is unsigned, hence the flag. Used when testing a change before it is
        # published.
        con = duckdb.connect(config={"allow_unsigned_extensions": "true"})
        con.execute(f"LOAD '{ext_path.as_posix()}'")
    try:
        row = con.execute(
            "SELECT extension_version FROM duckdb_extensions() "
            "WHERE extension_name = 'laterite_ags4'"
        ).fetchone()
        ver = (row[0] if row else None) or f"duckdb-{duckdb.__version__}"
        ver = ver.lstrip(
            "v"
        )  # the ext reports "v0.7.0"; the comparator adds its own "v"
    except Exception:
        ver = f"duckdb-{duckdb.__version__}"

    parses = []
    for p in fixtures:
        name = Path(p).name
        esc_p = p.replace("'", "''")
        try:
            # `"group"` is quoted — it is a SQL reserved word.
            groups = con.execute(
                f"SELECT \"group\", n_rows FROM ags_groups('{esc_p}')"
            ).fetchall()
        except Exception as e:
            # A non-AGS4 / unreadable fixture: read_ags has nothing to agree on.
            # Record the read error so the reference can confirm it also can't
            # parse the file (a hard-error fixture agrees by both failing).
            parses.append({"fixture": name, "read_error": _first_line(e), "groups": []})
            continue
        gset = []
        for gname, n_rows in groups:
            try:
                ids = group_key_rows(con, p, gname)
            except Exception as e:
                # One group failed to read (the fixture as a whole parsed). Omit
                # it — the reference HAS this group, so the comparator flags it as
                # "not read by duckdb" (a real disagreement), not a silent skip.
                print(
                    f"  ! {name}/{gname}: read_ags failed ({_first_line(e)}) — omitting",
                    file=sys.stderr,
                )
                continue
            gset.append({"group": gname, "n_rows": n_rows, "ids": ids})
        parses.append({"fixture": name, "read_error": None, "groups": gset})
    return {
        "schema": 2,
        "surface": "duckdb",
        "kind": "parse-agreement",
        "version": ver,
        "parses": parses,
    }


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--fixtures", required=True)
    ap.add_argument("--out", default="output/compliance-results")
    ap.add_argument(
        "--ext",
        type=Path,
        default=None,
        help="load a locally-built extension instead of the published one "
        "(default: INSTALL ... FROM community)",
    )
    a = ap.parse_args()

    if a.ext is not None and not a.ext.exists():
        print(f"duckdb: extension not built at {a.ext} — skipping (nothing to check)")
        return 0
    try:
        import duckdb  # noqa: F401
    except ImportError:
        print("duckdb: not importable — skipping (run under the ext's venv python)")
        return 0

    # str, not Path — group_key_rows embeds these directly in raw SQL text via
    # str.replace(); a Path would make that a Path.replace() (rename) instead.
    fixtures = sorted(str(f) for f in Path(a.fixtures).glob("*.ags"))
    try:
        surf = run(fixtures, a.ext)
    except Exception as e:  # e.g. an ABI mismatch under the wrong duckdb
        # Includes "no community build for this DuckDB yet", which is a real
        # state after a DuckDB release and not a defect in either engine.
        print(f"duckdb: extension load/run failed ({e}) — skipping (nothing to check)")
        return 0

    Path(a.out).mkdir(parents=True, exist_ok=True)
    path = Path(a.out) / "duckdb-parse.json"
    with path.open("w") as f:
        json.dump(surf, f)
    ngroups = sum(len(p["groups"]) for p in surf["parses"])
    print(
        f"  duckdb v{surf['version']}: {len(surf['parses'])} fixtures / "
        f"{ngroups} groups -> {path}"
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
