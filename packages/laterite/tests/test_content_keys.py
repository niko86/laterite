"""Content-addressed `_id` / `_parent_id` keys on the Python read surface (#303).

The pitch — stateless cross-group joins / merge / dedup via two synthetic UUIDv8
columns — was sold as core but shipped only in the `.ags5db` DuckDB extension
(`laterite.read(...).sql("... s._parent_id = l._id")` used to raise *Binder
Error: no column "_parent_id"*). Phase 3 makes it real in the wheel, via the one
shared Rust keychain. The model these tests pin:

* the **relational `.sql()` layer always carries** `_id`/`_parent_id` (so joins
  work with no opt-in — the advertised feature),
* the **frame accessor strips them by default**; `read(keys=True)` or a per-call
  `ags.table(code, keys=True)` re-includes them,
* **emit never writes them** (read→build can't leak a synthetic key into AGS4).

This is the "prove the feature, don't claim it" discipline: golden UUID values,
determinism, exact strip/keep behaviour, and the `_id`-never-in-bytes guard.
"""

from __future__ import annotations

import inspect
import uuid

import laterite as L
import polars as pl

# A fixed two-group file (PROJ root + its LOCA child carrying PROJ_ID) so the
# parent-link + golden ids are stable. Keep in sync with the golden values below.
_SRC = (
    '"GROUP","PROJ"\r\n"HEADING","PROJ_ID"\r\n"UNIT",""\r\n"TYPE","ID"\r\n"DATA","P1"\r\n'
    '"GROUP","LOCA"\r\n"HEADING","LOCA_ID","PROJ_ID"\r\n'
    '"UNIT","",""\r\n"TYPE","ID","ID"\r\n"DATA","BH1","P1"\r\n'
)
# Golden UUIDv8s — computed from the one shared `keychain::row_ids`. A change here
# means the id maths moved: Phase 6 asserts these equal Node / wasm / extension.
_PROJ_ID = "ac30a95d-e0ca-85f9-83c8-37a64af2762b"
_LOCA_ID = "a7025a6f-d9b8-83b6-8fad-81c0c744edbc"


# --- the frame accessor: strip by default, keep on request ----------------


def test_default_read_frame_has_no_key_columns():
    f = L.read(text=_SRC)
    assert "_id" not in f["PROJ"].columns
    assert "_parent_id" not in f["PROJ"].columns


def test_keys_true_adds_exactly_id_and_parent_id():
    plain = set(L.read(text=_SRC)["PROJ"].columns)
    keyed = set(L.read(text=_SRC, keys=True)["PROJ"].columns)
    assert keyed - plain == {"_id", "_parent_id"}, "keys=True adds exactly the two key columns"


def test_per_call_table_keys_override():
    f = L.read(text=_SRC)  # handle default is keys=False
    assert "_id" in f.table("PROJ", keys=True).columns  # per-call opt-in
    assert "_id" not in f.table("PROJ").columns  # still stripped without it


def test_key_columns_are_strings():
    loca = L.read(text=_SRC, keys=True)["LOCA"]
    assert loca.schema["_id"] == pl.String
    assert loca.schema["_parent_id"] == pl.String


# --- the relational layer ALWAYS exposes the keys -------------------------


def test_sql_sees_keys_regardless_of_read_flag():
    # default read (keys=False) — the engine still has them.
    rel = L.read(text=_SRC).sql("SELECT _id, _parent_id FROM PROJ").df()
    assert list(rel.columns) == ["_id", "_parent_id"]


def test_cross_group_join_links_child_to_parent():
    ags = L.read(text=_SRC)
    j = ags.sql(
        "SELECT l.LOCA_ID, p.PROJ_ID AS parent FROM LOCA l JOIN PROJ p "
        "ON l._parent_id = p._id"
    ).df()
    assert len(j) == 1 and j["LOCA_ID"][0] == "BH1" and j["parent"][0] == "P1"


# --- golden values + determinism + the root NULL --------------------------


def test_golden_uuid_values():
    fk = L.read(text=_SRC, keys=True)
    assert fk["PROJ"]["_id"][0] == _PROJ_ID
    assert fk["LOCA"]["_id"][0] == _LOCA_ID
    assert uuid.UUID(_PROJ_ID).version == 8


def test_child_parent_id_equals_parent_id():
    fk = L.read(text=_SRC, keys=True)
    assert fk["LOCA"]["_parent_id"][0] == fk["PROJ"]["_id"][0] == _PROJ_ID


def test_root_parent_id_is_null():
    assert L.read(text=_SRC, keys=True)["PROJ"]["_parent_id"][0] is None


def test_ids_are_deterministic_across_reads():
    a = L.read(text=_SRC, keys=True)["LOCA"]["_id"][0]
    b = L.read(text=_SRC, keys=True)["LOCA"]["_id"][0]
    assert a == b == _LOCA_ID


# --- emit never leaks a synthetic key -------------------------------------


def test_handle_bytes_never_contain_id():
    # The handle's own emit uses the retained parse, not the table — always clean.
    assert b"_id" not in L.read(text=_SRC, keys=True).bytes


def test_build_ags4_strips_keys_from_a_keyed_frame():
    fk = L.read(text=_SRC, keys=True)
    out = L.build_ags4({"PROJ": fk["PROJ"], "LOCA": fk["LOCA"]})
    assert "_id" not in out.text and "_parent_id" not in out.text


# --- the `keys=` argument exists where advertised -------------------------


def test_read_and_table_accept_keys_kwarg():
    assert "keys" in inspect.signature(L.read).parameters
    assert "keys" in inspect.signature(L.Ags4File.table).parameters
