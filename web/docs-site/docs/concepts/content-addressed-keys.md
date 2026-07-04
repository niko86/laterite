# Content-addressed join keys

When laterite lands a group in the relational (DuckDB) model, every row carries
two synthetic columns: `_id` and `_parent_id`. Both are **UUIDv8 values derived
from the row's spec key-chain** — its own AGS key fields, plus its parent's, walked
up the group tree. The id is a hash of *what the row is*, not of when or where it
was read.

That single property does the work:

- **Joins are by construction.** A child's `_parent_id` is computed from the same
  key-chain its parent uses for `_id`, so `child._parent_id == parent._id`
  always — there is no foreign key to populate, reconcile, or get wrong.
- **No shared state across reads.** Read `LOCA` from one file and `SAMP` from
  another; if their key fields agree, the ids line up. Nothing has to be read
  together, or in order, for the graph to connect.
- **Identical content → identical id.** The same row re-read (or re-emitted)
  hashes to the same `_id`, so de-duplication is a `DISTINCT` away — no
  surrogate-key bookkeeping.

The join you'd otherwise write on AGS keys collapses to the id columns:

```sql
-- AGS-key join (what you write today across groups)
SELECT * FROM SAMP s JOIN LOCA l USING (LOCA_ID);

-- the same edge, content-addressed
SELECT * FROM SAMP s JOIN LOCA l ON s._parent_id = l._id;
```

Both return the same rows. The difference is that the second join holds for
*every* parent/child edge in the dictionary with one column pair — you don't need
to know that `SAMP` hangs off `LOCA` by `LOCA_ID`, or that `GEOL` hangs off `SAMP`
by `(LOCA_ID, SAMP_TOP)`. The key-chain is baked into `_parent_id`.

## Where the keys live

The keys are **core on every surface** — the Python wheel, the Node package, the
browser (wasm), and the DuckDB extension — and they are **byte-identical** across
them, because all four compute them through the one shared keychain. The same file
read anywhere yields the same `_id`/`_parent_id`.

Two rules govern where you see them:

- **The relational layer always carries them.** `ags.sql(...)` / `ags.at(...)`
  expose `_id`/`_parent_id` on every group, so the content-addressed join above
  needs no opt-in.
- **Frames strip them by default.** `ags["LOCA"]` / `ags.table("LOCA")` return the
  AGS columns only — the synthetic keys don't clutter a data frame. Ask for them
  with `keys=True`:

```python
import laterite as L
ags = L.read("delivery.ags")

ags["LOCA"]                    # AGS columns only
ags.table("LOCA", keys=True)   # + _id / _parent_id
L.read("delivery.ags", keys=True)["LOCA"]   # handle-wide default

# joins work regardless of the flag — the engine always has the keys
ags.sql("SELECT * FROM SAMP s JOIN LOCA l ON s._parent_id = l._id")
```

In Node it is the same shape — `ags.table("LOCA", { keys: true })` and
`await ags.sql("… ON s._parent_id = l._id")`.

Because the ids live in the Arrow columns themselves (with `keys=True`), a join
needs **no engine** — hand the keyed frame to any Arrow tool (polars, arrow-js,
DuckDB, …) and match `_parent_id` to `_id`.

**Emit never writes them.** `save()` / `build_ags4(...)` stay byte-faithful to the
AGS data, so a `keys=True` frame round-tripped back to AGS4 drops the synthetic
columns — they live only in the read model.

!!! note "Why it matters"
    Content-addressing turns the AGS parent/child tree into a stable graph that
    survives independent reads, partial files, and re-emission. Same content, same
    key — so merging two deliveries, or spotting a row that appears twice, is a set
    operation rather than a join you have to hand-author per group pair.

See also: [SQL across groups](../cookbook/sql-across-groups.md) ·
[Born-typed reads](./born-typed.md)
