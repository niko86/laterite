# read_typed builds a typed PROJ tree you can walk by group code.
from laterite.ags4 import read_typed

proj = read_typed("examples/sample_site.ags")

print(proj.proj_id)
print(len(proj.walk("LOCA")))

# walk REQUIRES the group code arg; it returns every node of that group.
assert proj.proj_id == "LAT-DEMO" and len(proj.walk("LOCA")) == 14
