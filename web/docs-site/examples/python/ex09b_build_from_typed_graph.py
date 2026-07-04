import laterite
from laterite import build_ags4
from laterite.groups import LOCA, PROJ

# A typed PROJ graph — children attach via .append or the constructor kwarg.
p = PROJ(proj_id="LAT-DEMO", proj_name="Built from a typed graph")
p.locas.append(LOCA(loca_id="BH01", loca_gl=12.50))  # attach via .append …
PROJ(proj_id="P2", locas=[LOCA(loca_id="BH02", loca_gl=13.75)])  # … or the ctor kwarg

# #214: the graph is walked depth-first. The door emits only the headings you
# set (like the frames door), and autofix synthesizes the metadata catalogs —
# so a sparse graph builds a valid file in one call.
res = build_ags4(p)
print("groups:", laterite.read(data=res.bytes).groups)
print("findings:", len(res.findings))

assert {"PROJ", "LOCA", "TRAN", "UNIT", "TYPE"}.issubset(laterite.read(data=res.bytes).groups)
assert not res.findings  # a valid file, no caveats

# The managed child collection is append-only — reassigning it raises:
try:
    p.locas = [LOCA(loca_id="BH99")]
    raise AssertionError("expected AttributeError")
except AttributeError:
    pass
