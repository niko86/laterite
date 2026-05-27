# ags5db query recipes — pattern catalogue

Reusable templates indexed by **the shape of the question**, not the AGS group. When an agent gets a natural-language analytical question, scan the index, pick the matching recipe, copy the template, swap in the actual field names from `ags5db headings <GROUP>`.

These recipes are also available via the CLI: `ags5db recipe` lists them, `ags5db recipe <name>` prints one, and `ags5db recipe <name> --group LLPL` substitutes `<test_group>`/`<GROUP>` placeholders with the chosen group. Preferred when you're driving the CLI rather than reading docs.

Each recipe is short on purpose. They're **starting points** — adjust filters and field names to fit. The gotcha line is the one bit that's easy to get wrong.

## Pattern-by-question-shape index

| If the user asks… | Recipe |
|---|---|
| "what's the average X by soil type / geology layer?" | [depth-band-join](#depth-band-join) |
| "a sample spans a geology boundary — how do I handle it?" | [range-intersect](#range-intersect) |
| "every reading under triaxial test X / specimen Y" | [parent-chain-drill](#parent-chain-drill) |
| "average X binned by depth / per metre" | [depth-bin-aggregate](#depth-bin-aggregate) |
| "compare boreholes / site-level stats" | [cross-borehole-compare](#cross-borehole-compare) |
| "the table has 40k rows, just sample it for a plot" | [high-volume-downsample](#high-volume-downsample) |
| "what does code XYZ mean?" | [abbr-lookup](#abbr-lookup) |
| "how many readings matching X per borehole, top N" | [filter-then-aggregate](#filter-then-aggregate) |

## depth-band-join

**Shape:** Match a point-depth test result (LLPL, single-depth sample) to the geology unit it sits in.

```sql
SELECT g.geol_geol AS soil, AVG(t.<field>) AS avg_value, COUNT(*) AS n
FROM v_<test_group> t
JOIN v_geol g
  ON t.loca_id = g.loca_id
 AND t.<depth_col> >= g.geol_top
 AND t.<depth_col> <  g.geol_base   -- half-open: sample at boundary belongs to ONE layer
WHERE g.geol_geol IS NOT NULL
GROUP BY 1
ORDER BY n DESC
```

`<depth_col>` is `samp_top` for sample-based tests (LLPL, TREG…), or the reading depth (`scpt_dpth`, `mond_dpth`) for in-situ tests.

**Gotcha:** Use `[geol_top, geol_base)` half-open (`>= top AND < base`), not `BETWEEN`. A sample exactly at a layer boundary belongs to one unit only; `BETWEEN` would double-count it.

**Worked example — average atterberg limit by 5 most common soil types:**

```bash
ags5db --output table sql DB.ags5db "
  WITH atterberg_typed AS (
    SELECT l.llpl_ll, l.llpl_pl, l.llpl_pi, g.geol_geol
    FROM v_llpl l
    JOIN v_geol g
      ON l.loca_id = g.loca_id
     AND l.samp_top >= g.geol_top
     AND l.samp_top <  g.geol_base
    WHERE l.llpl_ll IS NOT NULL AND g.geol_geol IS NOT NULL
  )
  SELECT geol_geol AS soil,
         COUNT(*) AS n,
         ROUND(AVG(llpl_ll), 1) AS avg_ll,
         ROUND(AVG(llpl_pl), 1) AS avg_pl,
         ROUND(AVG(llpl_pi), 1) AS avg_pi
  FROM atterberg_typed
  GROUP BY 1 ORDER BY n DESC LIMIT 5
"
```

## range-intersect

**Shape:** A sample (or test interval) physically spans more than one geology layer — point-in-band isn't right.

```sql
SELECT t.<id>, g.geol_geol,
       -- overlap length, for weighted aggregation:
       LEAST(t.samp_base, g.geol_base) - GREATEST(t.samp_top, g.geol_top) AS overlap_m
FROM v_<test_group> t
JOIN v_geol g
  ON t.loca_id = g.loca_id
 AND t.samp_top  < g.geol_base       -- ranges overlap iff
 AND t.samp_base > g.geol_top        -- A.start < B.end AND A.end > B.start
WHERE g.geol_geol IS NOT NULL
```

**Gotcha:** A sample's "length" must be available — `samp_base` exists on some delivery styles but not all. Check `ags5db headings <DB> SAMP` first; if `samp_base` is missing, fall back to [depth-band-join](#depth-band-join) and accept the simplification. To allocate a numeric measurement across layers, weight by `overlap_m / (samp_base - samp_top)`.

## parent-chain-drill

**Shape:** Pull child rows filtered by an ancestor's KEY without writing the JOIN yourself.

The view `v_<group>` already JOINs every ancestor and exposes their KEYs as columns. Filter on the inherited column directly:

```bash
# Every TREL reading under TREG of test type CU
ags5db peek DB.ags5db TREL \
  --fields loca_id,samp_id,treg_type,tret_tesn,trel_strn,trel_dvst \
  --where "treg_type=CU" --limit 200
```

**Gotcha:** `peek`'s `--where` validates fields against the registry, so a typo like `treg_typ` returns exit 5 with the candidate list. Use that as a free schema check. For multi-condition filters, repeat `--where` (ANDed).

## depth-bin-aggregate

**Shape:** Profile-style summary — value averaged into discrete depth bins, e.g. SCPT resistance averaged per metre.

```sql
SELECT
  WIDTH_BUCKET(scpt_dpth, 0, 30, 30) AS bin,   -- 30 buckets spanning 0-30 m
  ROUND(MIN(scpt_dpth), 1) AS bin_top,
  ROUND(AVG(scpt_res), 2)  AS avg_res,
  COUNT(*) AS n
FROM v_scpt
WHERE loca_id = 'AR-CPT01'
GROUP BY 1
ORDER BY 1
```

**Gotcha:** Pick the bin span to match the data range — `WIDTH_BUCKET(x, 0, 30, 30)` gives 1-m bins from 0 to 30 m, with overflow into bucket 0 (below) or 31 (above). Run `ags5db sql DB.ags5db "SELECT MIN(scpt_dpth), MAX(scpt_dpth) FROM v_scpt"` first to size it right.

## cross-borehole-compare

**Shape:** Site-level stats by `loca_id` — ground level distribution, max depth reached, count by location type.

```bash
ags5db --output table sql DB.ags5db "
  SELECT loca_type,
         COUNT(*) AS n,
         ROUND(AVG(loca_gl), 2)  AS avg_gl,
         ROUND(MAX(loca_fdep), 1) AS deepest
  FROM v_loca
  GROUP BY 1 ORDER BY n DESC
"
```

**Gotcha:** Add a spatial filter when the question is regional: `WHERE loca_nate BETWEEN 480000 AND 481000 AND loca_natn BETWEEN 410000 AND 411000`. Eastings/northings are typed DOUBLE so range filters work directly.

## high-volume-downsample

**Shape:** A group like SCPT/MOND has 10k–50k rows. You want a representative subset for a plot, not the full sweep.

```sql
-- One row per N: take every 10th reading.
SELECT loca_id, scpt_dpth, scpt_res
FROM v_scpt
WHERE loca_id = 'AR-CPT01'
  AND CAST(scpt_dpth * 10 AS INTEGER) % 10 = 0  -- every 0.1 m
ORDER BY scpt_dpth

-- Or use DuckDB's USING SAMPLE — random subset:
SELECT loca_id, scpt_dpth, scpt_res
FROM v_scpt USING SAMPLE 1%
WHERE loca_id = 'AR-CPT01'
```

**Gotcha:** Downsampling for plots is fine. Don't downsample before computing aggregates (MIN/MAX/AVG) — you'd be running stats on a sample of a sample. Either downsample for plotting *only*, or use bin-aggregate above for representative summaries.

## abbr-lookup

**Shape:** Decode an AGS code like `ALV` / `CP+RC` / `CU` to its human description.

```bash
ags5db peek DB.ags5db ABBR \
  --fields abbr_hdng,abbr_code,abbr_desc \
  --where "abbr_code=ALV" --output json
```

For a heading-scoped lookup in SQL (the same code can mean different things in different headings):

```sql
SELECT g.geol_geol, a.abbr_desc
FROM v_geol g
LEFT JOIN v_abbr a
  ON a.abbr_code = g.geol_geol
 AND a.abbr_hdng = 'GEOL_GEOL'
LIMIT 10
```

**Gotcha:** ABBR rows have a per-file `abbr_hdng` qualifier — always include it in the JOIN to avoid pulling a `loca_type=IP` definition when you wanted `geol_geol=IP`.

## filter-then-aggregate

**Shape:** "How many readings matching predicate X per borehole, top N" — single-group filter + group-by.

```bash
ags5db --output table sql DB.ags5db "
  SELECT loca_id, COUNT(*) AS n
  FROM v_scpt
  WHERE scpt_dpth >= 10
  GROUP BY 1 ORDER BY n DESC LIMIT 3
"
```

**Gotcha:** For a single threshold with no GROUP BY, `count` is cheaper than `sql`:
`ags5db count DB.ags5db SCPT --where "scpt_dpth>=10"`. Reach for `sql` only when grouping, joining, or running aggregates beyond `COUNT`/`SUM`.
