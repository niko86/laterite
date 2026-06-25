# what this shows: ags.at(code, ids) fans out to one borehole's related group set, exposed as q.groups.
import laterite

ags = laterite.read("examples/sample_site.ags")
q = ags.at("LOCA", ["BH01", "BH02"])

print(q.groups)
assert "LOCA" in q.groups and "SAMP" in q.groups
