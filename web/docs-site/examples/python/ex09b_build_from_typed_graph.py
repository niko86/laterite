import laterite
from laterite import build_ags4
from laterite.groups import LOCA, PROJ

# A typed PROJ graph — children attach via .append or the constructor kwarg.
p = PROJ(proj_id="LAT-DEMO", proj_name="Built from a typed graph")
p.locas.append(LOCA(loca_id="BH01", loca_gl=12.50))  # attach via .append …
PROJ(proj_id="P2", locas=[LOCA(loca_id="BH02", loca_gl=13.75)])  # … or the ctor kwarg

# #214: the graph is walked depth-first, and the door emits only the headings
# you set (like the frames door) — so a sparse graph stays sparse.
res = build_ags4(p)
print("groups:", laterite.read(data=res.bytes).groups)
print("findings:", len(res.findings))

# The root-metadata groups (TRAN/UNIT/TYPE/ABBR/DICT) have no parent, so they are
# not part of a PROJ-rooted graph and cannot be reached by the walk. They are
# reported, not invented:
assert set(laterite.read(data=res.bytes).groups) == {"PROJ", "LOCA"}
assert res.findings

# `synthesise_metadata=True` derives the ones that CAN be derived — UNIT and TYPE
# from your data, ABBR when PA codes are used. PROJ, DICT and TRAN are never
# invented: a project identity, a schema extension and a record of transmission
# are yours to state. A guessed DICT parent would quietly mislead the relational
# checks, and a placeholder TRAN would satisfy the rule while asserting a
# transmission that never happened.
full = build_ags4(
    p,
    synthesise_metadata=True,
    tran_issue="1",
    tran_date="2026-07-30",
    tran_producer="Demo Producer",
    tran_recipient="Demo Recipient",
    tran_status="Final",
)
assert {"PROJ", "LOCA", "TRAN", "UNIT", "TYPE"}.issubset(
    laterite.read(data=full.bytes).groups
)
assert not full.findings  # a valid file, in one call, because you asked

# The managed child collection is append-only — reassigning it raises:
try:
    p.locas = [LOCA(loca_id="BH99")]
    raise AssertionError("expected AttributeError")
except AttributeError:
    pass
