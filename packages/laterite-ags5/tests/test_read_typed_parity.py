"""W2 parity guard: ``laterite.ags4.read_typed`` (DuckDB-free base path) must
produce a typed PROJ tree byte-identical to the reference
``convert → read_db`` path.

``read_typed`` is a *base* AGS4 API and no longer routes through a temp
``.ags5db`` (that dragged DuckDB into a pure-AGS4 read). Its linkage is a
hand port of the converter's parent resolution
(``laterite-ags5-db/src/convert.rs`` — ``insert_group_rows`` /
``resolve_parent_uuid``). This test pins the two implementations together so
a change to *either* — the Rust converter or the Python port — that diverges
the tree fails loud. It lives in the ``[ags5]`` suite because the reference
path needs DuckDB; the base oracle (``packages/laterite/tests/
test_ags4_typed.py``) covers the no-DuckDB contract.

The fixtures both *start from an AGS4 file* and reach the typed tree two
ways, so it's a true apples-to-apples comparison of the same source:
  * a rich multi-level tree (PROJ → LOCA×2 → {SAMP×2 → TREG → TRET → TREL,
    GEOL×2}) — generated via build → write_db → export so the denormalised
    multi-heading shared keys are correct by construction, and
  * a passthrough group (custom QQTS) — the dynamic-class path.
"""

from __future__ import annotations

from pathlib import Path
from typing import Any

from laterite import ags5db
from laterite.ags4 import read_typed
from laterite.ags_types import CanonicalType, canonical_type
from laterite.registry import GROUPS, child_groups

# --- generic typed-tree normaliser (works on compiled + dynamic nodes) ---


def _node_code(node: Any) -> str:
    return getattr(node, "_ags_code", None) or type(node).__name__


def _scalars(node: Any, code: str) -> dict[str, Any]:
    """Scalar field values by name. Compiled ``#[pyclass]`` scalars live in
    Rust fields (read via getattr, named by the dictionary); dynamic-class
    scalars are listed on ``_ags_headings``."""
    g = GROUPS.get(code)
    names = (
        [h.name.lower() for h in g.headings]
        if g is not None
        else list(getattr(node, "_ags_headings", ()))
    )
    return {n: getattr(node, n, None) for n in names}


def _children(node: Any, code: str) -> dict[str, list[Any]]:
    """Map child group code → its (non-empty) instance list. Standard
    children hang on the compiled ``<code>s`` field; passthrough children are
    setattr'd onto ``__dict__`` as plural-named lists."""
    found: dict[str, list[Any]] = {}
    standard_fields = set()
    for cg in child_groups(code):
        field = f"{cg.code.lower()}s"
        standard_fields.add(field)
        lst = getattr(node, field, None)
        if lst:
            found[cg.code] = list(lst)
    for field, val in vars(node).items():
        if field in standard_fields or not isinstance(val, list) or not val:
            continue
        found[_node_code(val[0])] = list(val)
    return found


def _normalize(node: Any) -> tuple:
    """Order-independent canonical form: (code, sorted scalars, sorted
    children). Children are sorted by repr so sibling/row order never matters."""
    code = _node_code(node)
    scalars = tuple(sorted(_scalars(node, code).items()))
    children = tuple(
        sorted(
            (
                cc,
                tuple(sorted((_normalize(c) for c in lst), key=repr)),
            )
            for cc, lst in _children(node, code).items()
        )
    )
    return (code, scalars, children)


def _assert_parity(ags4_path: Path, tmp_path: Path) -> tuple:
    """Reach the typed tree two ways from the same AGS4 source and assert
    they're identical. Returns the shared normal form (so callers can make
    extra structural assertions on it)."""
    db = tmp_path / f"{ags4_path.stem}.ags5db"
    ags5db.convert(ags4_path, db)
    reference = _normalize(ags5db.read_db(db))
    candidate = _normalize(read_typed(ags4_path))
    assert candidate == reference
    return candidate


# --- fixtures --------------------------------------------------------


def _rich_ags4(tmp_path: Path) -> Path:
    """A deep, multi-branch tree → write_db → export to AGS4. Going through
    export guarantees the denormalised key columns (SAMP's full key tuple on
    each TREG row, LOCA_ID on every descendant) are spec-correct, so the
    shared-key linkage being tested is exercised on real multi-heading
    tuples."""
    from laterite import GEOL, LOCA, PROJ, SAMP, TREG, TREL, TRET

    locas = []
    for bh, gl in (("BH01", 10.5), ("BH02", 8.0)):
        samps = []
        for top, ref in ((1.0, "S1"), (2.5, "S2")):
            samp_keys = dict(
                loca_id=bh, samp_top=top, samp_ref=ref, samp_type="U", samp_id="X"
            )
            spec_keys = {**samp_keys, "spec_ref": "R", "spec_dpth": top}
            # one SAMP per LOCA carries a deep TREG → TRET → TREL chain.
            tregs = []
            if ref == "S1":
                trels = [
                    TREL(**spec_keys, tret_tesn="1", trel_mnum=i, trel_cell=350.0 + i)
                    for i in range(2)
                ]
                tregs = [
                    TREG(
                        **spec_keys,
                        treg_type="CU",
                        trets=[TRET(**spec_keys, tret_tesn="1", trels=trels)],
                    )
                ]
            samps.append(SAMP(**samp_keys, tregs=tregs))
        geols = [
            GEOL(loca_id=bh, geol_top=t, geol_base=t + 1.0, geol_leg="CLAY")
            for t in (0.0, 1.0)
        ]
        locas.append(
            LOCA(loca_id=bh, loca_type="CP", loca_gl=gl, samps=samps, geols=geols)
        )
    proj = PROJ(proj_id="P1", proj_name="parity", locas=locas)

    db = tmp_path / "rich.ags5db"
    ags5db.write_db(proj, db)
    ags4 = tmp_path / "rich.ags"
    ags5db.export(db, ags4)
    return ags4


def _codes_in(normal: tuple) -> set[str]:
    """Every group code present anywhere in a normalised tree."""
    code, _, children = normal
    out = {code}
    for _, lst in children:
        for child in lst:
            out |= _codes_in(child)
    return out


def _synth_cell(htype: str, n: int) -> str:
    """A parse-safe AGS4 *string* cell for a heading of the given AGS type.
    The actual value is irrelevant to parity (both paths parse the same text);
    it just has to be well-typed so DT/DATE/TIME/numeric headings exercise the
    real per-type parsing rather than degenerating to passthrough strings."""
    try:
        ct = canonical_type(htype)
    except ValueError:
        ct = CanonicalType.STRING
    if ct is CanonicalType.INTEGER:
        return str(n % 900 + 1)
    if ct is CanonicalType.DECIMAL:
        return f"{n % 900 + 1}.5"
    if ct is CanonicalType.DATETIME:
        return "2020-01-15 09:30:00"
    if ct is CanonicalType.DATE:
        return "2020-01-15"
    if ct is CanonicalType.TIME:
        return "09:30:00"
    if ct is CanonicalType.BOOL:
        return "Y" if n % 2 else "N"
    return f"V{n}"  # string / enum / picklist


def _all_reachable_groups_ags4(tmp_path: Path) -> tuple[Path, int]:
    """Generate ONE AGS4 file covering every group reachable from PROJ via the
    registry's parent edges — each with all its headings, values synthesised
    per AGS type, KEY columns denormalised down the chain so the shared-key
    linkage actually fires. Returns (path, reachable_group_count).

    This is the breadth guard: it forces read_typed and convert→read_db to
    agree on *every* group's parsing and *every* edge's linkage, not just the
    handful a hand-written fixture would touch."""
    counter = [0]
    rows: dict[str, tuple[Any, dict[str, str]]] = {}
    order: list[str] = []

    def walk(code: str, ctx: dict[str, str]) -> None:
        g = GROUPS[code]
        row: dict[str, str] = {}
        local = dict(ctx)
        for h in g.headings:
            if h.status == "KEY":
                if h.name not in local:
                    counter[0] += 1
                    local[h.name] = _synth_cell(h.type, counter[0])
                row[h.name] = local[h.name]
            else:
                counter[0] += 1
                row[h.name] = _synth_cell(h.type, counter[0])
        rows[code] = (g, row)
        order.append(code)
        for cg in child_groups(code):
            walk(cg.code, local)

    walk("PROJ", {})

    lines: list[str] = []
    for code in order:
        g, row = rows[code]
        lines.append(f'"GROUP","{code}"')
        lines.append('"HEADING",' + ",".join(f'"{h.name}"' for h in g.headings))
        lines.append('"UNIT",' + ",".join(f'"{h.unit or ""}"' for h in g.headings))
        lines.append('"TYPE",' + ",".join(f'"{h.type}"' for h in g.headings))
        lines.append('"DATA",' + ",".join(f'"{row[h.name]}"' for h in g.headings))
        lines.append("")

    ags4 = tmp_path / "all_groups.ags"
    ags4.write_text("\n".join(lines), encoding="utf-8")
    return ags4, len(order)


_PASSTHROUGH_AGS = (
    '"GROUP","PROJ"\n'
    '"HEADING","PROJ_ID","PROJ_NAME"\n'
    '"UNIT","",""\n'
    '"TYPE","ID","X"\n'
    '"DATA","P1","passthrough parity"\n'
    "\n"
    '"GROUP","LOCA"\n'
    '"HEADING","LOCA_ID","LOCA_TYPE","LOCA_GL"\n'
    '"UNIT","","","m"\n'
    '"TYPE","ID","PA","2DP"\n'
    '"DATA","BH01","CP","10.50"\n'
    '"DATA","BH02","CP","8.00"\n'
    "\n"
    '"GROUP","QQTS"\n'
    '"HEADING","LOCA_ID","QQTS_REF","QQTS_VAL"\n'
    '"UNIT","","","kPa"\n'
    '"TYPE","ID","X","1DP"\n'
    '"DATA","BH01","R1","100.0"\n'
    '"DATA","BH01","R2","200.0"\n'
)


# --- tests -----------------------------------------------------------


def test_read_typed_matches_read_db_rich_tree(tmp_path: Path) -> None:
    """Deep multi-level tree: read_typed == convert→read_db, and the tree
    actually has the structure we think (guards a degenerate both-empty pass)."""
    ags4 = _rich_ags4(tmp_path)
    normal = _assert_parity(ags4, tmp_path)

    # Sanity that parity isn't trivially over an empty tree.
    code, _, children = normal
    assert code == "PROJ"
    child_codes = {cc for cc, _ in children}
    assert "LOCA" in child_codes
    locas = next(lst for cc, lst in children if cc == "LOCA")
    assert len(locas) == 2  # both boreholes linked under the single PROJ


def test_read_typed_matches_read_db_passthrough(tmp_path: Path) -> None:
    """Custom group via the dynamic-class path: read_typed == convert→read_db."""
    from laterite import dynamic

    dynamic.clear_cache()
    ags4 = tmp_path / "passthrough.ags"
    ags4.write_text(_PASSTHROUGH_AGS, encoding="utf-8")
    _assert_parity(ags4, tmp_path)


def test_read_typed_matches_read_db_every_reachable_group(tmp_path: Path) -> None:
    """Breadth guard: read_typed == convert→read_db for EVERY group reachable
    from PROJ (every edge, every heading type). Catches any group shape — odd
    KEY intersection, pseudo-key drift, an edge AGS type that parses
    differently — that a hand-picked fixture would miss."""
    ags4, reachable = _all_reachable_groups_ags4(tmp_path)
    normal = _assert_parity(ags4, tmp_path)

    covered = _codes_in(normal)
    # The generator reaches every parented group; guard against the registry
    # (or the tree walk) silently shrinking what's actually exercised.
    assert reachable >= 80, f"only {reachable} groups generated"
    assert len(covered) >= 80, f"only {len(covered)} groups in the compared tree"
