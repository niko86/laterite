# what this shows: laterite.diff(a, b) — a KEY-aware, type-aware revision diff between two AGS4 texts.
from pathlib import Path

import laterite

# Two revisions of the same submission, differing in one PROJ cell (PROJ_NAME).
baseline = Path("examples/sample_site.ags").read_text()
revision = baseline.replace(
    "laterite demo site (synthetic starter - replace me)",
    "laterite demo site (Rev B)",
)
assert revision != baseline

# diff() returns a RevisionDelta dict: per-group row/heading deltas + counts.
# Rows are matched by the group's dictionary KEY headings, and cells are compared
# through the *typed* value, so only a genuine quantity change registers.
delta = laterite.diff(baseline, revision)

# Pull out the PROJ group and its single changed row.
proj = next(g for g in delta["groups"] if g["code"] == "PROJ")
changed = [row for row in proj["rows"] if row["kind"] == "changed"]

print("totals:", delta["total_added"], delta["total_removed"], delta["total_changed"])
print("PROJ key headings:", proj["key_headings"])
print("changed row key:", changed[0]["key"])
print("changed cell:", changed[0]["cells"][0])

# A 'changed' PROJ row, keyed on PROJ_ID, carrying a heading/type/a/b cell.
assert delta["total_changed"] == 1
assert proj["key_headings"] == ["PROJ_ID"]
assert proj["keyed"] is True
row = changed[0]
assert row["kind"] == "changed"
assert row["key"] == ["LAT-DEMO"]  # the PROJ_ID value
cell = row["cells"][0]
assert cell["heading"] == "PROJ_NAME"
assert cell["type"] == "X"
assert cell["a"] == "laterite demo site (synthetic starter - replace me)"
assert cell["b"] == "laterite demo site (Rev B)"
