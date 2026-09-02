# Produce AGS4

```python
--8<-- "python/ex09a_build_from_frames.py:code"
```

```text
--8<-- "python/ex09a_build_from_frames.out"
```

`build_ags4` takes a `{code: frame}` mapping (the columns are your AGS
headings) and constructs a file from exactly the groups you supplied.

AGS4 also mandates the metadata catalogs (`TRAN`, `UNIT`, `TYPE`, plus `ABBR`
for any `PA` pick-list codes), which your frames don't carry. Those are
**reported, not invented**: hence the three findings above, Rules 14/15/17.
`mode="autofix"` repairs what your input _contains_; it does not mint groups you
never wrote.

To get them, ask:

```python
res = laterite.build_ags4(
    {"PROJ": proj, "LOCA": loca},
    synthesise_metadata=True,
    tran=laterite.TranStamp(
        issue="1",
        date="2026-07-30",
        producer="Your Firm",
        recipient="The Client",
        status="Final",
    ),
)
```

`UNIT` and `TYPE` are derived from your columns. `TRAN` is not derivable; only
you know who sent what to whom, so you state it. Omit the stamp and no `TRAN`
is written and Rule 14 reports the gap, rather than a placeholder being invented
that would _satisfy_ the rule while asserting a transmission that never happened.
All five are required together: they are REQUIRED headings, so `TranStamp`
demands them rather than letting a half-stamp reach the file. `TRAN_AGS`,
`TRAN_DLIM` and `TRAN_RCON` are absent from it on purpose: they describe the
file the emitter is writing, so it fills them.

Synthesis is opt-in on every surface (`synthesise_metadata=` in Python,
`{ synthesiseMetadata }` in Node and in the browser wasm build) so nothing
appears in your file that you didn't ask for.

## From a typed PROJ graph

```python
--8<-- "python/ex09b_build_from_typed_graph.py:code"
```

```text
--8<-- "python/ex09b_build_from_typed_graph.out"
```

The other door takes a typed graph: a `PROJ` with `LOCA` children attached via
`.locas.append(...)` or the `locas=[...]` constructor kwarg. `build_ags4` walks
it depth-first and, like the frames door, emits **only the headings you
set**, and only the groups you built. That's why a sparse graph builds clean:
nothing is invented, in your data columns or around them. `synthesise_metadata=True`
works here too, and reports the same Rules 14/15/17 without it. The managed child
collection is append-only, so reassigning `p.locas` raises `AttributeError` rather
than silently dropping the rows you built up.

!!! tip
    Both doors return the same `BuildResult`. Inspect `res.text` / `res.bytes`
    in memory, check `res.findings` for any caveats autofix couldn't resolve, or
    `res.save("out.ags")` to persist a byte-faithful AGS4 file to disk.

## Straight to disk

When the file's destination is a path anyway, say so in the call:

```python
saved = laterite.build_ags4({"PROJ": proj, "LOCA": loca}, out="delivery.ags")
saved.path           # where the judged document landed
saved.findings       # the same verdict a BuildResult carries
```

`out=` returns a `BuildSaved`: the path plus the findings/fixes verdict, and
deliberately **no** `bytes`. It exists for long-lived processes that don't
want the whole file resident on the result after the call. Build-and-judge
survives the trip to disk: the document is staged to a temporary file beside
the destination and moved into place only after the verdict allows, so the
path never holds unjudged output, and a `mode="strict"` refusal raises with
nothing written. Node mirrors it as `buildAgs4(groups, { out })`; the browser
build has no filesystem, so there is nothing to mirror there.

## Three write doors, one honest difference

Three ways out of laterite produce an AGS4 file, and what separates them is
what each one *claims* about its output:

- **`compat`'s writer** is the faithful blind write: your string frames go to
  disk exactly as you hold them, matching python-ags4's behaviour byte for
  byte. It claims nothing: fidelity to your data *is* the contract.
- **`build_ags4`** is build-and-judge: canonical formatting, dictionary
  fills, and then the full rule engine over the result. The verdict (the
  `findings` on the object you get back) is the premium you are paying for,
  and it is most of what the call spends its time on.
- **`build_ags4_unchecked`** is the same build with the verdict declined:

```python
raw = laterite.build_ags4_unchecked({"PROJ": proj, "LOCA": loca})   # bytes
laterite.build_ags4_unchecked({"PROJ": proj, "LOCA": loca}, out="delivery.ags")
```

The bytes are **identical to `build_ags4(mode="report")`'s**: same fills,
same canonical cells, same order, with a test pinning the identity. But
nothing has checked them against any AGS4 rule, and nothing will. There is
no `mode=` (there is no verdict for a mode to act on), no
`synthesise_metadata=`/`tran=` (synthesis fills gaps only a report would
surface), and the return is plain `bytes` rather than a `BuildResult`,
because an empty findings list would read as "judged clean" when nothing
judged anything. With `out=` it stages and renames exactly as above, minus
the verdict gate in front of the write.

Reach for it when the verdict is genuinely spent elsewhere: a pipeline's
inner loop whose final output gets validated once, a file bound for an
external checker. You are choosing to ship unchecked bytes; that is the whole
feature, and the choice is yours to make.

## Keeping the build's memory peak down

`build_ags4` judges its own writing as it streams, so the door's side of the
peak is lean. What usually dominates a build-from-a-read workflow is the
**caller's own holds**, live across the call:

- **Drop the read handle once your frames exist.** A handle from
  `laterite.read` retains the parsed file; if you only needed it to make the
  frames, `del handle` before building. The door never needed it.
- **Pass Arrow-capsule tables rather than materialised frames** where you
  have them: anything exposing `__arrow_c_stream__` goes straight through,
  without a whole-file frame materialisation beside it.

Both were measured on the perf campaign's write workflow (the variant table
on #848 records each hold's share); they are the caller's allocations, which
is why they live here as a recipe rather than inside the door.

→ See the whole fluent API assembled in the
[Chaining Showcase](../chaining/index.md).
