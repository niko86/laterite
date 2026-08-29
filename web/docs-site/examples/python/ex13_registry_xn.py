# /// script
# requires-python = ">=3.12"
# dependencies = ["laterite==0.12.0"]
# ///
"""Docs example — run it with `uv run ex13_registry_xn.py`, from anywhere.

Everything above the `[start:code]` marker is machinery the page does not
show: the PEP 723 header that makes the file self-installing,
and the fixture arm that makes its repo-relative path resolve outside a
checkout.
"""

import urllib.request
from pathlib import Path

_FIXTURE = Path("examples/sample_site.ags")
_RAW = "https://raw.githubusercontent.com/niko86/laterite/main/examples/sample_site.ags"
if not _FIXTURE.exists():
    # Cold only for a reader running this outside the repo: in a checkout (and in
    # CI, cwd = repo root) the file is already there and this arm never executes,
    # so the gates stay offline. Fetching it — rather than rewriting the example
    # to an absolute path — is what keeps the text on the page the text you would
    # actually type.
    _FIXTURE.parent.mkdir(parents=True, exist_ok=True)
    _FIXTURE.write_bytes(urllib.request.urlopen(_RAW, timeout=30).read())

# --8<-- [start:code]
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
# --8<-- [end:code]
