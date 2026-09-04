"""The modality standing gate — the axis the behavioural-knob parity gates miss.

`test_free_chained_parity` / `test_cross_surface_parity` compare behavioural KNOBS
on pairs that exist on both sides and STRIP the modality-bearing params
(`source/path/text/data/index/in_place/out`) before comparing. So a capability
offered in fewer I/O *forms* on one surface — path-only vs bytes-only transport,
the exact gap that motivated this audit — has no pair to fire against and slips
through. This gate closes that hole.

It reads `modality.json` (the repo-root register + by-design allowlist) and
checks it against REFLECTED reality on the three in-repo surfaces:

* **Python** — EXECUTED probes for the runtime-sniffed source doors (`read` of a
  path / str / bytes / BytesIO), `inspect` for keyword doors, and module/attr
  presence for the transport + emit + certify forms. Probes are the strength
  mechanism: Python funnels every input form through one positional `source: Any`
  with runtime sniffing (`_resolve_source`), invisible to `inspect.signature`, so
  a new door added there still moves the reflected set. **This is the check that
  would have moved the register when `lock_bytes` landed.**
* **Node** — a focused TS parser: the exported verbs' first-param type unions
  (`string | Uint8Array | Ags4File`, alias-followed), the option-interface
  `text`/`index` fields, and the transport export-name set.
* **Browser (wasm + web/src/lib)** — an orphan-guard: the top-level
  `#[wasm_bindgen] pub fn` verb set and the web transport exports must map 1:1 to
  the register's browser capabilities (the completeness half of
  `test_fixable_contract`).

Each reflected cell asserts, for the forms it can OBSERVE:
* every observed-PRESENT form is credited present/divergent in the register — a
  door added without a register edit fails CI;
* every observed-ABSENT form is NOT credited present — a register that claims a
  door which no longer exists (a closed/removed gap) fails CI.

The CLI is documentation-only here — its parser isn't statically typed.

DuckDB no longer is, and the register's `reflection` says so (`manifest-cross-check`,
was `documentation`). #763 shipped `tools/check_duckdb_manifest.py`, gated in
`repo-gates`, which cross-checks the register's duckdb verbs against the extension's
own `functions.json` in BOTH directions. Note the scope: that instrument sees the
PRESENCE half only. Whether a cell's *content* is right is still hand-authored and
unreflected, so a duckdb cell can be wrong in every way except existing.
"""

from __future__ import annotations

import importlib.util
import inspect
import io
import json
import re
from pathlib import Path

import laterite as L
from laterite import transport

REPO = Path(__file__).parents[3]
REGISTER = REPO / "modality.json"
TS = REPO / "rust-packages" / "laterite-node" / "ts"
WASM_SRC = REPO / "rust-packages" / "laterite-ags4-wasm" / "src"


def _gen_modality():
    """The generator, loaded by path — the house pattern for a tools/ script.

    Imported for `uncovered()` alone. That predicate is what `gen_modality.py`
    already PRINTS on every run, and this test is what makes it fail; defining
    it twice would let the report and the gate drift into disagreeing about
    which pairs are missing.
    """
    spec = importlib.util.spec_from_file_location(
        "gen_modality", REPO / "tools" / "gen_modality.py"
    )
    assert spec and spec.loader
    mod = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(mod)
    return mod


def _wasm_source() -> str:
    """Every module of the wasm crate, as one text to match against.

    The whole `src/`, not `lib.rs`: that crate is one module per verb (#381), so
    the exports these gates enumerate are spread across it. A single-file read
    finds nothing and — for the orphan guard — would report every verb as
    missing rather than saying it had looked in the wrong place.
    """
    return "\n".join(
        p.read_text(encoding="utf-8") for p in sorted(WASM_SRC.glob("*.rs"))
    )


WEB_TRANSPORT = REPO / "web" / "src" / "lib" / "transportClient.ts"

_AGS = "\r\n".join(
    [
        '"GROUP","PROJ"',
        '"HEADING","PROJ_ID","PROJ_NAME"',
        '"UNIT","",""',
        '"TYPE","ID","X"',
        '"DATA","P1","modality probe"',
        "",
    ]
)
_AGS_BYTES = _AGS.encode("utf-8")


# --------------------------------------------------------------------------- #
# register access
# --------------------------------------------------------------------------- #


def _doc() -> dict:
    return json.loads(REGISTER.read_text(encoding="utf-8"))


def _cell(
    doc: dict, capability: str, surface: str, spelling: str | None = None
) -> dict:
    for cap in doc["capabilities"]:
        if cap["capability"] != capability:
            continue
        for cell in cap["cells"]:
            if cell["surface"] == surface and (
                spelling is None or cell.get("spelling") == spelling
            ):
                return cell
    raise AssertionError(f"no register cell for {capability}/{surface}/{spelling}")


def _offered(cell: dict, direction: str) -> set[str]:
    """The forms the register credits as reachable (present OR divergent)."""
    dd = "input" if direction == "in" else "output"
    return {f for f, st in cell.get(dd, {}).items() if st in ("present", "divergent")}


def _assert_reflection(
    cell: dict, direction: str, present: set[str], absent: set[str]
) -> None:
    """The bidirectional bite: observed-present must be credited; observed-absent
    must not be credited. Names the cell + the fix in the message."""
    offered = _offered(cell, direction)
    where = f"{cell['surface']} {cell.get('spelling', '')} {direction}".strip()
    uncredited = present - offered
    assert not uncredited, (
        f"{where}: form(s) {sorted(uncredited)} exist in reality but the register "
        f"does not credit them — a door was added/closed without updating modality.json"
    )
    phantom = absent & offered
    assert not phantom, (
        f"{where}: register credits form(s) {sorted(phantom)} that reflection shows "
        f"ABSENT — a removed door left a stale claim in modality.json"
    )


# --------------------------------------------------------------------------- #
# well-formedness (the register is its own contract — no separate faithfulness gate)
# --------------------------------------------------------------------------- #


def test_register_well_formed():
    doc = _doc()
    forms = doc["forms"]
    declared = {"in": set(forms["input"]), "out": set(forms["output"])}
    surfaces = {s["id"] for s in doc["surfaces"]}
    seen: set[tuple] = set()
    for cap in doc["capabilities"]:
        c = cap["capability"]
        for cell in cap["cells"]:
            key = (c, cell["surface"], cell.get("spelling"))
            assert key not in seen, f"duplicate cell {key}"
            seen.add(key)
            assert cell["surface"] in surfaces, f"{key}: unknown surface"
            for d, dd in (("in", "input"), ("out", "output")):
                for f, st in cell.get(dd, {}).items():
                    assert f in declared[d], f"{key}: {dd} form {f!r} not declared"
                    assert st in ("present", "absent", "divergent"), (
                        f"{key}: {dd}[{f}]={st!r}"
                    )
                for v in cell.get("verbs", []):
                    for f in v.get("in_forms", []):
                        assert f in declared["in"], (
                            f"{key}: verb {v['name']} bad in_form {f!r}"
                        )
                    for f in v.get("out_forms", []):
                        assert f in declared["out"], (
                            f"{key}: verb {v['name']} bad out_form {f!r}"
                        )
            for g in cell.get("gaps", []):
                dd = "input" if g["direction"] == "in" else "output"
                st = cell.get(dd, {}).get(g["form"])
                assert st in ("absent", "divergent"), (
                    f"{key}: gap on {g['form']!r} but reflected {st!r} — a closed gap must "
                    f"flip its form to present, not linger as a 'gap' label"
                )
                assert g["verdict"] in ("gap", "by-design"), f"{key}: bad verdict"
                assert g["priority"] in ("P1", "P2", "P3", "by-design"), (
                    f"{key}: bad priority"
                )
                assert g.get("reason", "").strip(), (
                    f"{key}: gap needs a non-empty reason"
                )
                # by-design ⇔ no work planned; a real gap always carries a work priority.
                if g["verdict"] == "by-design":
                    assert g["priority"] == "by-design", (
                        f"{key}: by-design needs by-design priority"
                    )
                else:
                    assert g["priority"] != "by-design", (
                        f"{key}: a gap needs a P1/P2/P3 priority"
                    )


# --------------------------------------------------------------------------- #
# Python — executed probes + inspect + attr presence
# --------------------------------------------------------------------------- #


def test_every_capability_has_a_cell_for_every_surface():
    """The grid is WHOLE — every capability x surface pair holds a verdict.

    `test_register_well_formed` validates every cell that EXISTS: it iterates
    `for cell in cap["cells"]`, so a pair with no cell is never visited and
    never judged. That is the blind spot this closes, and it is not academic —
    twelve pairs were missing when this landed, ten of them `cli` (#772, #779).

    The other two guards cannot see it either. `gen_modality.py --check`
    compares the rendered page to the SSOT, and a missing cell renders
    faithfully as nothing, so both agree while both omit the row.

    The pair COUNT is asserted rather than printed. Per the house rule a gate
    says what it covered on every run, and pytest captures stdout — so a number
    nobody sees is not a report. In the assertion it is a gate: add a surface
    or a capability and this fails until every pair on the new row or column
    carries a verdict, which is the whole point.
    """
    doc = _doc()
    surfaces = [s["id"] for s in doc["surfaces"]]
    caps = doc["capabilities"]

    missing = _gen_modality().uncovered(doc)
    assert not missing, (
        f"{len(missing)} capability x surface pair(s) hold no verdict — a cell "
        f"absent from the register is invisible to every other guard here:\n  "
        + "\n  ".join(missing)
    )

    covered = sum(len({c["surface"] for c in cap["cells"]}) for cap in caps)
    assert covered == len(caps) * len(surfaces), (
        f"grid covers {covered} pairs, expected "
        f"{len(caps)} capabilities x {len(surfaces)} surfaces = "
        f"{len(caps) * len(surfaces)}"
    )


def test_python_read_input_doors():
    """The source-sniff doors, EXECUTED (invisible to inspect.signature)."""
    doc = _doc()
    cell = _cell(doc, "read", "python", "free")
    present, absent = set(), set()

    # path
    import tempfile

    with tempfile.NamedTemporaryFile("wb", suffix=".ags", delete=False) as fh:
        fh.write(_AGS_BYTES)
        p = fh.name
    try:
        L.read(p)
        present.add("path")
    except Exception:
        absent.add("path")
    finally:
        Path(p).unlink(missing_ok=True)

    # text (a bare AGS4 string sniffs as content)
    try:
        L.read(_AGS)
        present.add("text")
    except Exception:
        absent.add("text")

    # bytes
    try:
        L.read(_AGS_BYTES)
        present.add("bytes")
    except Exception:
        absent.add("bytes")

    # file-like (.read())
    try:
        L.read(io.BytesIO(_AGS_BYTES))
        present.add("file-like")
    except Exception:
        absent.add("file-like")

    # cert (keyword door — inspect is honest here, the param is explicit)
    if "index" in inspect.signature(L.read).parameters:
        present.add("cert")
    else:
        absent.add("cert")

    _assert_reflection(cell, "in", present, absent)


def test_python_transport_forms():
    """The pack_bytes/lock_bytes MOTIVATING case: path via pack/lock, bytes via the
    *_bytes twins. Drop lock_bytes and this fails — the bite that would have caught
    the original path-only-vs-browser-bytes-only gap."""
    doc = _doc()
    exports = set(transport.__all__)
    # transport-pack: pack/unpack (path) + pack_bytes/unpack_bytes (bytes)
    pack = _cell(doc, "transport-pack", "python", "free")
    present = set()
    absent = set()
    (present if {"pack", "unpack"} <= exports else absent).add("path")
    (present if {"pack_bytes", "unpack_bytes"} <= exports else absent).add("bytes")
    _assert_reflection(pack, "in", present, absent)
    _assert_reflection(
        pack,
        "out",
        {"file"} if "path" in present else set(),
        {"file"} if "path" in absent else set(),
    )

    # transport-lock: lock/unlock (path) + lock_bytes/unlock_bytes (bytes)
    lock = _cell(doc, "transport-lock", "python", "free")
    present, absent = set(), set()
    (present if {"lock", "unlock"} <= exports else absent).add("path")
    (present if {"lock_bytes", "unlock_bytes"} <= exports else absent).add("bytes")
    _assert_reflection(lock, "in", present, absent)


def test_python_emit_and_certify_output_forms():
    """The read handle's emit forms (.text/.bytes/.save) and certify's output forms:
    certify() writes a file, certify_bytes() (#390) returns the cert bytes in
    memory. Probing the callables couples the register to the real handle surface."""
    doc = _doc()
    handle = L.read(_AGS_BYTES)

    emit = _cell(doc, "emit", "python", "chained")
    present, absent = set(), set()
    (present if isinstance(handle.text, str) else absent).add("text")
    (present if isinstance(handle.bytes, (bytes, bytearray)) else absent).add("bytes")
    (present if callable(getattr(handle, "save", None)) else absent).add("file")
    _assert_reflection(emit, "out", present, absent)

    certify = _cell(doc, "certify", "python", "chained")
    present, absent = set(), set()
    (present if callable(getattr(handle, "certify", None)) else absent).add("file")
    # the in-memory cert-bytes form: certify_bytes() returns the cert bytes (#390).
    (present if callable(getattr(handle, "certify_bytes", None)) else absent).add(
        "bytes"
    )
    _assert_reflection(certify, "out", present, absent)


def test_python_excel_bytes_forms():
    """EXECUTED excel probes (#391): to_excel(source) with no output returns the
    .xlsx bytes, and from_excel(xlsx_bytes) consumes raw workbook bytes. Running
    the conversions couples the register to the real bytes doors."""
    doc = _doc()
    xlsx = L.to_excel(_AGS_BYTES)  # no output → in-memory .xlsx bytes

    te = _cell(doc, "to_excel", "python", "free")
    present, absent = {"file"}, set()  # the path form is always there
    (present if isinstance(xlsx, (bytes, bytearray)) else absent).add("bytes")
    _assert_reflection(te, "out", present, absent)

    fe = _cell(doc, "from_excel", "python", "free")
    present, absent = set(), set()
    # bytes-in is reachable iff from_excel accepts the raw workbook bytes.
    try:
        ok = isinstance(L.from_excel(xlsx), L.Ags4File)
    except Exception:
        ok = False
    (present if ok else absent).add("bytes")
    _assert_reflection(fe, "in", present, absent)


# --------------------------------------------------------------------------- #
# Node — focused TS parser
# --------------------------------------------------------------------------- #


def _read(name: str) -> str:
    return (TS / name).read_text(encoding="utf-8")


def _ts_interface_fields(
    src: str, name: str, _seen: set[str] | None = None
) -> set[str]:
    """Field names of a TS interface, following one `extends` chain (lifted from
    test_cross_surface_parity — used only for the text/index option doors)."""
    _seen = _seen or set()
    if name in _seen:
        return set()
    _seen.add(name)
    m = re.search(
        rf"export interface {name}(?: extends (\w+))?\s*\{{(.*?)\n\}}", src, re.DOTALL
    )
    assert m is not None, f"interface {name} not found"
    parent, body = m.group(1), m.group(2)
    fields = set(re.findall(r"^\s*(\w+)\??\s*:", body, re.MULTILINE))
    if parent:
        fields |= _ts_interface_fields(src, parent, _seen)
    return fields


def _split_top_union(type_str: str) -> list[str]:
    """Split a TS type union on `|` at bracket depth 0 (so `Map<string, X>` and
    `Array<[string, Y]>` don't split internally)."""
    parts, depth, cur = [], 0, ""
    for ch in type_str:
        if ch in "<([{":
            depth += 1
        elif ch in ">)]}":
            depth -= 1
        if ch == "|" and depth == 0:
            parts.append(cur.strip())
            cur = ""
        else:
            cur += ch
    if cur.strip():
        parts.append(cur.strip())
    return parts


# TS source-type identifier → input modality form.
_TS_FORM = {
    "string": "path",
    "Uint8Array": "bytes",
    "Buffer": "bytes",
    "Ags4File": "handle",
}


def _node_source_forms(
    src: str, fn: str, alias: dict[str, str] | None = None
) -> set[str]:
    """The input forms carried by an exported fn's FIRST parameter type union.
    Resolves a single type alias (e.g. `DiffSource`)."""
    m = re.search(rf"export function {fn}\(\s*\w+\??\s*:\s*([^,)]+)", src)
    assert m is not None, f"function {fn} first-param type not found"
    raw = m.group(1).strip()
    members = _split_top_union(raw)
    forms: set[str] = set()
    for mem in members:
        mem = mem.strip()
        if alias and mem in alias:
            mem = alias[mem]
        for sub in _split_top_union(mem):
            key = sub.strip()
            if key in _TS_FORM:
                forms.add(_TS_FORM[key])
    return forms


def _resolve_alias(src: str, name: str) -> str:
    m = re.search(rf"export type {name}\s*=\s*([^;]+);", src)
    return m.group(1).strip() if m else name


def test_node_read_input_doors():
    doc = _doc()
    cell = _cell(doc, "read", "node", "free")
    idx = _read("index.ts")
    present = _node_source_forms(idx, "read")  # string|Uint8Array → path,bytes
    opts = _ts_interface_fields(idx, "ReadOptions")
    if "text" in opts:
        present.add("text")
    if "index" in opts:
        present.add("cert")
    # file-like is by-design absent on Node — reflection can't manufacture it.
    absent = {"file-like"}
    _assert_reflection(cell, "in", present, absent)


def test_node_fix_input_doors():
    doc = _doc()
    cell = _cell(doc, "fix", "node", "free")
    idx = _read("index.ts")
    present = _node_source_forms(idx, "fix")
    if "text" in _ts_interface_fields(idx, "FixOptions"):
        present.add("text")
    _assert_reflection(cell, "in", present, set())


def test_node_diff_input_doors():
    doc = _doc()
    cell = _cell(doc, "diff", "node", "free")
    idx = _read("index.ts")
    alias = {"DiffSource": _resolve_alias(idx, "DiffSource")}
    present = _node_source_forms(idx, "diff", alias)  # string|Uint8Array|Ags4File
    _assert_reflection(cell, "in", present, {"file-like"})


def test_node_transport_forms():
    """The remaining leg of the motivating gap: Node transport is path-only. If a
    `*Bytes`/`*_bytes` export lands (closing the P1), reflection sees `bytes` and
    the register's absent-claim fails until the row flips to present."""
    doc = _doc()
    tsrc = _read("transport.ts")
    exports = set(re.findall(r"export function (\w+)", tsrc))
    has_path = {"pack", "unpack", "lock", "unlock"} <= exports
    has_bytes = any(re.search(r"[Bb]ytes", e) for e in exports)
    for capability in ("transport-pack", "transport-lock"):
        cell = _cell(doc, capability, "node", "free")
        present, absent = set(), set()
        (present if has_path else absent).add("path")
        (present if has_bytes else absent).add("bytes")
        _assert_reflection(cell, "in", present, absent)


def test_node_certify_output_forms():
    """Node certify (Ags4File.certify) writes a path → file form; certifyBytes()
    (#390) returns a Buffer → bytes form. Reflected from the two method signatures
    in ags4-file.ts, so removing either flips the register."""
    doc = _doc()
    cell = _cell(doc, "certify", "node", "chained")
    ags4 = _read("ags4-file.ts")
    present, absent = set(), set()
    # The signatures carry knobs now (`dictVersion`), so match the FORMS — the path-in
    # and the return type — not the exact parameter list.
    m = re.search(r"certify\(path\?:\s*string[^)]*\)\s*:\s*(\w+)", ags4)
    assert m is not None, "Ags4File.certify signature not found"
    (present if m.group(1) == "string" else absent).add("file")
    # the in-memory bytes twin: certifyBytes() -> Buffer.
    has_bytes = re.search(r"certifyBytes\([^)]*\)\s*:\s*Buffer", ags4) is not None
    (present if has_bytes else absent).add("bytes")
    _assert_reflection(cell, "out", present, absent)


def test_node_excel_bytes_forms():
    """Node toExcel / fromExcel accept bytes-in (#391): their first-param unions
    widened from `string` to `string | Uint8Array`. Plus the Ags4File.toExcel()
    handle method. Reflected from the TS signatures, so a regression flips it."""
    doc = _doc()
    idx = _read("index.ts")
    for capability, fn in (("to_excel", "toExcel"), ("from_excel", "fromExcel")):
        cell = _cell(doc, capability, "node", "free")
        forms = _node_source_forms(idx, fn)  # string|Uint8Array → {path, bytes}
        _assert_reflection(cell, "in", forms, {"path", "bytes"} - forms)
    # the handle method must exist for the register's Ags4File.toExcel verb.
    assert re.search(r"toExcel\(\s*xlsxPath\?:\s*string", _read("ags4-file.ts")), (
        "Ags4File.toExcel() handle method missing — register credits it (#391)"
    )


# --------------------------------------------------------------------------- #
# Browser (wasm crate + web/src/lib) — orphan-guard
# --------------------------------------------------------------------------- #

# The wasm-verb → capability map, coupled to discovered reality (the
# test_fixable_contract idiom). A new top-level `#[wasm_bindgen] pub fn` with no
# entry here fails until it is mapped AND given a register browser cell.
_WASM_VERB_CAP = {
    "read": "read",
    "validate": "validate",
    "certify": "certify",
    "compute_fixes": "fix",
    "apply_fixes": "fix",
    "build_ags4": "build",
    "build_ags4_ipc": "build",
    "build_ags4_unchecked": "build-unchecked",
    "diff": "diff",
    "merge": "merge",
    "censor": "censor",
    "ags4_to_xlsx": "to_excel",
    "xlsx_to_ags4": "from_excel",
}
# Verbs with no file-I/O modality (catalogue/dictionary lookups, build metadata) —
# the by-design allowlist, hygiene-checked below so it can't mask a removed verb.
# `version` reports CARGO_PKG_VERSION, mirroring Node's `version()`; it takes no
# input and touches no file, so it has no I/O modality to register. Added in laterite-dev#556,
# where `ags4-compliance`'s wasm runner had HARD-CODED `version: "0.5.1"` because
# wasm exported nothing to ask — the report then claimed the wasm leg tested a
# two-minor-old build while the gate called it 4-laterite identity.
#
# `engine_version` / `engine_fingerprint` are the same shape one level down, and
# they exist because `version` stopped being able to answer the question. Since
# the tiers split (#202) this package carries the PRODUCT number while the rules
# carry their own, so a matching `version` across surfaces now shows only that
# they shipped together. The fingerprint is a digest of the engine's actual
# inputs, so surfaces reporting the same one are running the same rules — which
# is what laterite-dev#556's report thought it was asserting.
_WASM_META_ALLOW = {
    "list_rules",
    "dictionary",
    "version",
    "engine_version",
    "engine_fingerprint",
}

_WEB_VERB_CAP = {"lock": "transport-lock", "unlock": "transport-lock"}
# No allow-list entries: the readiness handshake export (`ready`) was dropped
# in #379 when transportClient moved onto the shared channel — nothing called
# it, and a request that lands early queues in the worker behind its own init.
_WEB_ALLOW: set[str] = set()


def _wasm_verbs() -> set[str]:
    """Top-level `#[wasm_bindgen] pub fn` verbs — the bare attr at col 0 above a
    col-0 `pub fn` (so impl-block getters and `pub struct`/`impl` are excluded).

    Any number of FURTHER col-0 attributes may sit between the two. That is not
    cosmetic tolerance: this used to require the two lines be adjacent, so adding
    an `#[allow(...)]` to an export made it invisible here. For an existing verb
    that fails loudly as a `stale` entry — but for a NEW one it fails *silently*,
    since an undiscovered verb simply never appears in `unregistered` and the
    guard passes while the export is unmapped. Exactly what this test exists to
    prevent.
    """
    src = _wasm_source()
    return set(
        re.findall(r"^#\[wasm_bindgen\]\n(?:^#\[[^\n]*\]\n)*^pub fn (\w+)", src, re.M)
    )


#: Exports still carrying a positional tail, each with the reason and the work
#: that retires it. An entry here is a RECORDED COMMITMENT, not a permanent
#: exemption — the point of listing them is that "not yet migrated" and "nobody
#: noticed" stop looking the same from outside.
#:
#: **Empty, and that is the finished state.** `diff` and `censor` were the last
#: two; both now take an options object, so every wasm export satisfies the gate
#: unaided. `test_arity_exemptions_are_live` is what emptied it — leaving the
#: migrated entries in would have failed as stale, which is the point of writing
#: the commitment down rather than a comment.
_ARITY_EXEMPT: dict[str, str] = {}

#: A wasm export takes its inputs, then ONE options object. Two positional
#: arguments is the shape (`data` + `opts`); three allows a genuine second input
#: (`merge(a, b, opts)`).
_MAX_WASM_ARITY = 3


def _wasm_verb_arity() -> dict[str, int]:
    """Each top-level `#[wasm_bindgen] pub fn`'s parameter count.

    Counts top-level commas only, so a generic or slice parameter
    (`Option<Vec<u8>>`, `&[u8]`) is one parameter rather than however many commas
    it encloses. `//` comments are stripped first — the parameter lists carry
    why-comments, and those contain prose commas that would otherwise be counted
    as parameters (which is exactly how this parser was wrong on first write).
    """
    src = _wasm_source()
    out: dict[str, int] = {}
    for m in re.finditer(
        r"^#\[wasm_bindgen\]\n(?:^#\[[^\n]*\]\n)*^pub fn (\w+)\s*\(", src, re.M
    ):
        depth, end = 1, m.end()
        for i, ch in enumerate(src[m.end() :], start=m.end()):
            if ch in "(<[":
                depth += 1
            elif ch in ")>]":
                depth -= 1
                if depth == 0:
                    end = i
                    break
        body = re.sub(r"//[^\n]*", "", src[m.end() : end])
        depth, params, buf = 0, [], ""
        for ch in body:
            if ch in "(<[":
                depth += 1
            elif ch in ")>]":
                depth -= 1
            if depth == 0 and ch == ",":
                params.append(buf)
                buf = ""
            else:
                buf += ch
        params.append(buf)
        out[m.group(1)] = len([p for p in params if p.strip()])
    return out


def test_wasm_exports_take_an_options_object_not_a_positional_tail():
    """No wasm export may grow a positional tail.

    Clippy does now see this crate (the `--exclude laterite-ags4-wasm` came off
    the workspace lint in #187), but `clippy::too_many_arguments` only fires at
    SEVEN — and seven positional parameters is already far past the shape that
    caused the trouble. This is the gate that holds the line at three.

    That line is not arbitrary: `build_ags4` reached NINE parameters, five of them
    consecutive same-typed `Option<String>`, and a browser caller had to pass
    five `undefined`s to reach the sixth. Clippy would have shrugged at eight of
    those nine. The options-object migration fixed the instances; this fixes the
    class.
    """
    over = {
        name: n
        for name, n in _wasm_verb_arity().items()
        if n > _MAX_WASM_ARITY and name not in _ARITY_EXEMPT
    }
    assert not over, (
        f"wasm exports with a positional tail: {over}. "
        f"An export takes its inputs then ONE options object (max {_MAX_WASM_ARITY} "
        "parameters). Add the option as a field on the export's options struct — "
        "and remember to add it to that struct's `KEYS`, which "
        "`option_keys_match_the_structs` enforces. If an export genuinely needs "
        "more, add it to _ARITY_EXEMPT with the reason."
    )


def test_arity_exemptions_are_live():
    """Every exemption must still be needed, and must name a real export.

    A stale entry is worse than none: it reads as "known and tracked" while the
    export has either been migrated (so the exemption hides that the gate now
    passes on its own) or renamed (so the exemption silently protects nothing).
    The same reasoning as `test_allowlist_is_live` in the cross-surface gate.
    """
    arities = _wasm_verb_arity()
    unknown = set(_ARITY_EXEMPT) - set(arities)
    assert not unknown, f"exemptions naming exports that do not exist: {unknown}"
    no_longer_needed = {
        name for name in _ARITY_EXEMPT if arities[name] <= _MAX_WASM_ARITY
    }
    assert not no_longer_needed, (
        f"these exports no longer need an exemption — delete their entries: "
        f"{no_longer_needed}"
    )


def test_arity_gate_can_see_the_exports():
    """Zero is a bad witness: an empty parse would make the gate above vacuous."""
    arities = _wasm_verb_arity()
    assert len(arities) > 5, f"the arity parser found almost nothing: {arities}"
    # A known shape, so a parser that returns 0 for everything is caught too.
    assert arities.get("validate") == 2, (
        f"validate should be (data, opts): {arities.get('validate')}"
    )


def test_wasm_orphan_guard():
    discovered = _wasm_verbs()
    mapped = set(_WASM_VERB_CAP) | _WASM_META_ALLOW
    unregistered = discovered - mapped
    assert not unregistered, (
        f"wasm verb(s) {sorted(unregistered)} exported but unmapped — map each to a "
        f"capability in _WASM_VERB_CAP (+ a browser cell in modality.json) or to "
        f"_WASM_META_ALLOW with a reason"
    )
    stale = mapped - discovered
    assert not stale, (
        f"wasm verb map has {sorted(stale)} not found in lib.rs — the export was "
        f"renamed/removed; update _WASM_VERB_CAP / _WASM_META_ALLOW"
    )


def test_wasm_capabilities_have_browser_cells():
    doc = _doc()
    for cap in set(_WASM_VERB_CAP.values()):
        _cell(doc, cap, "browser")  # raises if the browser cell is missing


def test_web_transport_orphan_guard():
    """The motivating surface itself: browser transport lives in web/src/lib
    (JS zstd/age), NOT the wasm crate. Guard its exports too."""
    src = WEB_TRANSPORT.read_text(encoding="utf-8")
    discovered = set(re.findall(r"export function (\w+)", src))
    mapped = set(_WEB_VERB_CAP) | _WEB_ALLOW
    assert discovered == mapped, (
        f"web transport export drift — unmapped {sorted(discovered - mapped)}, "
        f"stale {sorted(mapped - discovered)}"
    )
    doc = _doc()
    for cap in set(_WEB_VERB_CAP.values()):
        _cell(doc, cap, "browser")


# --------------------------------------------------------------------------- #
# Rust facade — reflected from the public-API snapshot
# --------------------------------------------------------------------------- #
#
# The `rust` surface joined the register in #241 with NO reflector: its cells
# were hand-written claims, and nothing compared them to the crate. Proven, not
# assumed — asserting `read: file-like = present` (false) passed this file 17/17.
#
# It reflects from the `tools/release/public-api/` facade snapshots rather than
# by parsing `src/`, because those snapshots are machine-generated by `cargo
# public-api` and held against the crate by their own CI gate. Reading them here
# chains onto a guarantee that already exists instead of adding a second, weaker
# parser — the register's own reflect-don't-hand-list discipline.


#: BOTH facade snapshots — the default surface and the all-features one
#: (dec-facade-parity decision 4). This reflector reads their UNION: whether a
#: door sits behind a feature (phase 7's `excel`) is not this register's axis,
#: and reading only the default file would report a feature-gated door absent
#: while the crate exports it.
PUBLIC_API_SNAPSHOTS = (
    REPO / "tools" / "release" / "public-api" / "laterite.txt",
    REPO / "tools" / "release" / "public-api" / "laterite.all-features.txt",
)

#: form -> the door that offers it. Absence of the door IS the absent reading,
#: so a form can be observed missing rather than merely not asserted.
_RUST_READ_DOORS = {
    "path": "read",
    "bytes": "read_bytes",
    "text": "read_str",
    # `impl std::io::Read` — Rust HAS a universal file-like, so node's
    # by-design reason for omitting it cannot be borrowed here.
    "file-like": "read_reader",
    # A builder method, not a free `read_cert`: the certificate is not a SOURCE,
    # it is an accelerator for a path/bytes read — it carries the byte index that
    # lets `.only([…])` slice a group instead of parsing the file.
    "cert": "Read::index",
}
#: certify's OUTPUT forms — the certificate itself, written or returned.
_RUST_CERTIFY_DOORS = {
    "file": "Certify::to_path",
    "bytes": "Certify::to_bytes",
}
#: cert-input's INPUT form. On this surface the door is on validate, not on
#: read: the facade's validate is a free function with no `Document::validate()`
#: for a read-time certificate to reach.
_RUST_CERT_INPUT_DOORS = {
    "cert": "Validate::index",
}
_RUST_VALIDATE_DOORS = {
    "path": "validate",
    "bytes": "validate_bytes",
    "text": "validate_str",
    "file-like": "validate_reader",
}
#: form -> `Type::method` on the write side.
_RUST_EMIT_DOORS = {
    "file": "Write::to_path",
    "bytes": "Written::bytes",
    "text": "Written::text",
}
#: fix's INPUT forms — the three source doors, same names as read/validate.
_RUST_FIX_IN_DOORS = {
    "path": "fix",
    "bytes": "fix_bytes",
    "text": "fix_str",
}
#: fix's OUTPUT forms. In place is the source path named as the destination
#: rather than a separate flag, so `to_path` is the whole file form.
_RUST_FIX_OUT_DOORS = {
    "value": "Fix::run",
    "file": "Fix::to_path",
}
#: build's INPUT forms. `value` is the caller's own data (`GroupData` rows of
#: `Cell`); `handle` is a `Document`, which is this surface's answer to python
#: and node's typed-graph root.
_RUST_BUILD_IN_DOORS = {
    "value": "build",
    "handle": "build_document",
}
_RUST_BUILD_OUT_DOORS = {
    "value": "Build::run",
}
#: diff's INPUT forms. The handle door compares each document as it stands —
#: edits included — which is why it is a distinct entry point rather than a
#: convenience over the bytes one.
_RUST_DIFF_IN_DOORS = {
    "path": "diff",
    "bytes": "diff_bytes",
    "handle": "diff_documents",
}
_RUST_DIFF_OUT_DOORS = {
    "value": "Diff::run",
}
_RUST_MERGE_IN_DOORS = {
    "path": "merge",
    "bytes": "merge_bytes",
    "handle": "merge_documents",
}
_RUST_MERGE_OUT_DOORS = {
    "value": "Merge::run",
}
# The `excel` feature's doors (dec-facade-parity phase 7). They render only in
# the ALL-FEATURES snapshot, which is why this reflector reads the union of the
# two facade files — reading `laterite.txt` alone would report every one of
# these absent while the crate exports them.
_RUST_TO_EXCEL_IN_DOORS = {
    "path": "to_excel",
    "bytes": "to_excel_bytes",
}
_RUST_TO_EXCEL_OUT_DOORS = {
    "file": "ToExcel::to_path",
    "bytes": "Workbook::bytes",
}
_RUST_FROM_EXCEL_IN_DOORS = {
    "path": "from_excel",
    "bytes": "from_excel_bytes",
}
_RUST_FROM_EXCEL_OUT_DOORS = {
    "file": "FromExcel::to_path",
    "bytes": "Converted::bytes",
}


def _rust_api() -> set[str]:
    """Every public item in the facade, as `name` or `Type::method`.

    Lifetimes and type parameters are stripped so `Write<'a>::to_path`,
    `Write<'_>::fmt` and `merge<I, P>` collapse onto one spelling — the snapshot
    carries every variant and the distinction is noise for "does this door
    exist".

    The comma in the character class is load-bearing: without it a generic
    function with more than one parameter — `merge<I, P>(…)` — does not match at
    all, and the reflector reports its door ABSENT while the crate exports it.
    Found by adding exactly such a door.
    """
    items: set[str] = set()
    for snapshot in PUBLIC_API_SNAPSHOTS:
        for line in snapshot.read_text(encoding="utf-8").splitlines():
            m = re.match(r"pub fn laterite::ags4::([A-Za-z0-9_:<>',\s]+?)\(", line)
            if m:
                items.add(re.sub(r"<[^>]*>", "", m.group(1)).strip())
    assert items, "public-API snapshot yielded no items — the reflector is broken"
    return items


def _reflect_rust(doors: dict[str, str], api: set[str]) -> tuple[set[str], set[str]]:
    present = {form for form, door in doors.items() if door in api}
    absent = {form for form, door in doors.items() if door not in api}
    return present, absent


def test_rust_facade_reflects_the_register():
    """Every I/O form the register claims for the Rust facade must be a door the
    published API actually has — and every door it has must be credited."""
    doc = _doc()
    api = _rust_api()
    for capability, doors, direction in (
        ("read", _RUST_READ_DOORS, "in"),
        ("validate", _RUST_VALIDATE_DOORS, "in"),
        ("emit", _RUST_EMIT_DOORS, "out"),
        ("certify", _RUST_CERTIFY_DOORS, "out"),
        ("cert-input", _RUST_CERT_INPUT_DOORS, "in"),
        ("fix", _RUST_FIX_IN_DOORS, "in"),
        ("fix", _RUST_FIX_OUT_DOORS, "out"),
        ("build", _RUST_BUILD_IN_DOORS, "in"),
        ("build", _RUST_BUILD_OUT_DOORS, "out"),
        ("diff", _RUST_DIFF_IN_DOORS, "in"),
        ("diff", _RUST_DIFF_OUT_DOORS, "out"),
        ("merge", _RUST_MERGE_IN_DOORS, "in"),
        ("merge", _RUST_MERGE_OUT_DOORS, "out"),
        ("to_excel", _RUST_TO_EXCEL_IN_DOORS, "in"),
        ("to_excel", _RUST_TO_EXCEL_OUT_DOORS, "out"),
        ("from_excel", _RUST_FROM_EXCEL_IN_DOORS, "in"),
        ("from_excel", _RUST_FROM_EXCEL_OUT_DOORS, "out"),
    ):
        present, absent = _reflect_rust(doors, api)
        _assert_reflection(_cell(doc, capability, "rust"), direction, present, absent)


def test_rust_source_doors_are_all_mapped():
    """A new `read_*` / `validate_*` / `fix_*` / `build_*` / `diff_*` / `merge_*`
    entry point must be given a form here.

    The completeness half — without it the reflector only ever sees doors someone
    remembered to add to the table, which is the hand-list it exists to replace.
    Same shape as the wasm and web orphan guards above.
    """
    api = _rust_api()
    discovered = {
        item
        for item in api
        if re.fullmatch(
            r"(read|validate|fix|build|diff|merge|to_excel|from_excel)(_\w+)?", item
        )
        and "::" not in item
    }
    mapped = (
        set(_RUST_READ_DOORS.values())
        | set(_RUST_VALIDATE_DOORS.values())
        | set(_RUST_FIX_IN_DOORS.values())
        | set(_RUST_BUILD_IN_DOORS.values())
        | set(_RUST_DIFF_IN_DOORS.values())
        | set(_RUST_MERGE_IN_DOORS.values())
        | set(_RUST_TO_EXCEL_IN_DOORS.values())
        | set(_RUST_FROM_EXCEL_IN_DOORS.values())
    )
    unmapped = discovered - mapped
    assert not unmapped, (
        f"the facade offers {sorted(unmapped)} but no form is mapped to it — add "
        "it to the _RUST_*_DOORS tables and credit the form in modality.json, or "
        "the register will under-report the surface"
    )
