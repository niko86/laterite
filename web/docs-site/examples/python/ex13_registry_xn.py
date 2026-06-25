# what this shows: querying the AGS dictionary registry, and reading XN-typed columns as real numbers.
import laterite
from laterite.registry import child_groups, inherited_key_names

# The registry is the in-memory AGS group graph. child_groups returns GroupDescriptor
# objects; list their .code to see every group that hangs off LOCA.
loca_children = child_groups("LOCA")
print("LOCA children:", len(loca_children))
print("first few:", [g.code for g in loca_children[:3]])

# inherited_key_names walks the parent chain and returns the KEY headings a group
# inherits — SAMP samples are located by their parent borehole's LOCA_ID.
print("SAMP inherits:", inherited_key_names("SAMP"))

# AGS "XN" headings are numeric-but-text on disk. xn="numeric" casts them on read,
# so LLPL_PL (plastic limit) comes back as Float64 instead of String.
ags = laterite.read("examples/sample_site.ags", xn="numeric")
print("LLPL_PL dtype:", ags["LLPL"]["LLPL_PL"].dtype)

assert len(loca_children) == 50
assert inherited_key_names("SAMP") == {"LOCA_ID"}
assert str(ags["LLPL"]["LLPL_PL"].dtype) == "Float64"
