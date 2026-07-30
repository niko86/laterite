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

CLI and DuckDB are documentation-only here (the CLI parser isn't statically typed;
the DuckDB ext lives in a separate gitignored repo absent from this checkout, so a
`output/**` regex would be a permanent no-op — see the register's surface notes).
"""

from __future__ import annotations

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
WASM_LIB = REPO / "rust-packages" / "laterite-ags4-wasm" / "src" / "lib.rs"
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
    "diff": "diff",
    "merge": "merge",
    "censor": "censor",
    "ags4_to_xlsx": "to_excel",
    "xlsx_to_ags4": "from_excel",
}
# Verbs with no file-I/O modality (catalogue/dictionary lookups, build metadata) —
# the by-design allowlist, hygiene-checked below so it can't mask a removed verb.
# `version` reports CARGO_PKG_VERSION, mirroring Node's `version()`; it takes no
# input and touches no file, so it has no I/O modality to register. Added in #556,
# where `ags4-compliance`'s wasm runner had HARD-CODED `version: "0.5.1"` because
# wasm exported nothing to ask — the report then claimed the wasm leg tested a
# two-minor-old build while the gate called it 4-laterite identity.
_WASM_META_ALLOW = {"list_rules", "dictionary", "version"}

_WEB_VERB_CAP = {"lock": "transport-lock", "unlock": "transport-lock"}
_WEB_ALLOW = {"ready"}  # worker readiness handshake, not a capability verb


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
    src = WASM_LIB.read_text(encoding="utf-8")
    return set(
        re.findall(r"^#\[wasm_bindgen\]\n(?:^#\[[^\n]*\]\n)*^pub fn (\w+)", src, re.M)
    )


#: Exports still carrying a positional tail, each with the reason and the work
#: that retires it. An entry here is a RECORDED COMMITMENT, not a permanent
#: exemption — the point of listing them is that "not yet migrated" and "nobody
#: noticed" stop looking the same from outside.
#:
#: `validate` and `certify` are deliberately absent: they were migrated first, to
#: prove the machinery (the decode trait, the unknown-key guard, the hand-written
#: TS interfaces) on the two exports with the smallest blast radius.
_ARITY_EXEMPT: dict[str, str] = {
    "build_ags4": "5 args; options-object migration, next phase",
    "build_ags4_ipc": "5 args; migrates with build_ags4 — they share BuildOptions",
    "merge": "5 args; options-object migration, the phase after build",
    "diff": "4 args; not yet scoped — the plan records it as its own follow-up",
    "censor": "6 args; not yet scoped — same follow-up as diff",
}

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
    src = WASM_LIB.read_text(encoding="utf-8")
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

    **Nothing else enforces this.** `ci.yml:266` runs
    `cargo clippy --workspace ... --exclude laterite-ags4-wasm`, so
    `clippy::too_many_arguments` has never fired on this crate and would not fire
    on the next export either — the `#[allow(too_many_arguments)]` attributes
    that used to sit on these functions were decorative.

    That mattered: `build_ags4` reached NINE parameters, five of them consecutive
    same-typed `Option<String>`, and a browser caller had to pass five
    `undefined`s to reach the sixth. The options-object migration fixed the
    instances; this fixes the class.
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
