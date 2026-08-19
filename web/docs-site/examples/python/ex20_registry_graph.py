# /// script
# requires-python = ">=3.12"
# dependencies = ["laterite==0.11.0"]
# ///
"""Docs example — run it with `uv run ex20_registry_graph.py`, from anywhere.

Everything above the `[start:code]` marker is machinery the page does not
show: the PEP 723 header that makes the file self-installing.

No fixture arm: the registry is the bundled dictionary, so this one reads no
file at all — which is the point the page is making about it.
"""

# --8<-- [start:code]
# what this shows: the whole group graph is importable — no file, no engine call.
from laterite.registry import GROUPS, child_groups

print(len(GROUPS))  # groups in the union dictionary
print([g.code for g in child_groups("PROJ")])  # top of the tree

# child_groups is ONE level, not the subtree: SAMP is a LOCA child, LLPL hangs
# off SAMP and is therefore not in LOCA's children.
loca_children = {g.code for g in child_groups("LOCA")}
assert "SAMP" in loca_children
assert "LLPL" not in loca_children
# --8<-- [end:code]
