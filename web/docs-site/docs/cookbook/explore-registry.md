# Explore the registry & KEY chain

**Available in:** Python (`laterite.registry` is a Python-only module — see the
[capability matrix](../surfaces/index.md#what-each-door-can-do))

Ask the AGS dictionary itself which groups hang off a parent, and which KEY
headings a child inherits — without opening a file. The registry is the
in-memory AGS group graph; reach for it to drive UI, build joins, or validate a
schema before you have data.

```python
--8<-- "python/ex13_registry_xn.py"
```

```text
LOCA children: 50
first few: ['BKFL', 'CDIA', 'CHIS']
SAMP inherits: {'LOCA_ID'}
LLPL_PL dtype: Float64
```

`child_groups("LOCA")` returns the `GroupDescriptor` objects that name `LOCA` as
their parent — 50 of them, from `BKFL` through `CDIA` to `CHIS`. Each descriptor
carries the static facts for one group: `.code`, `.parent`, `.headings`, and
`.key_headings` / `.non_key_headings`. None of this needs a delivery loaded —
it's projected from the single-source dictionary, so it answers "what *could* be
here" rather than "what *is* here".

`inherited_key_names("SAMP")` walks the parent chain and returns the KEY
headings a child inherits from its **direct** parent: a `SAMP` sample is located
by its borehole, so it inherits `{LOCA_ID}`. This is the join key — when you fan
out with `.at(...)` or stitch groups in SQL, the inherited KEY is the column the
child shares with its parent. Read it straight off the registry to build a join
condition before touching the data.

The last two lines show the `xn="numeric"` read knob in passing — `LLPL_PL`
(plastic limit, an `XN`/numeric-as-text heading) comes back as `Float64`. That's
a read-side concern covered on its own page below.

One variation — to list every group code, not just `LOCA`'s children, import the
whole graph:

```python
from laterite.registry import GROUPS, child_groups

print(len(GROUPS))                       # 174 groups in the union dictionary
print([g.code for g in child_groups("PROJ")])   # top of the tree
```

Gotcha: `child_groups` is **one level** of the tree, not the full subtree —
`SAMP` is a `LOCA` child but `LLPL` (which hangs off `SAMP`) is not in
`child_groups("LOCA")`. Recurse on `.code` if you want the whole branch.
Likewise `inherited_key_names` returns the *direct* parent's KEY only, not the
full ancestor chain.

See also: [Read XN as numbers](./read-xn-numeric.md) ·
[Born-typed](../concepts/born-typed.md)
