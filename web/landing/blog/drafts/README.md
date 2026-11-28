# Blog series drafts: "one borehole's whole story"

Working notes for the first series of posts on laterite.dev. Nothing here is
published or built. Where the blog lives, and how a post is rendered, is open
in #1240; this folder is a holding place until that is decided.

Each post has a folder holding a first draft: `linkedin.md` (the short post),
`article.md` (the longer one) and the script the article quotes. Every script
was run on the released wheel against the public data named below, and the
output in the drafts is what it printed. `figures.py` draws every image into its post's folder.

## The shape of a post

- A short LinkedIn post: an image, a few lines of code, a link.
- A longer article on laterite.dev with the full runnable script.
- Each post is about one thing laterite does, on the reader's own AGS4 file.
  It starts from `laterite.read("your_file.ags")`, or for the merge post, a
  folder of them.
- A closing line may point to `niko86/laterite-quarry` for readers with no
  file to hand. Figures may be drawn with `niko86/laterite-outcrop`. Neither
  is the subject of a post.

## The series

| #   | Folder                     | Hook                                           | What laterite delivers            | Image                    |
| --- | -------------------------- | ---------------------------------------------- | --------------------------------- | ------------------------ |
| 1   | `post-1-one-borehole`      | One line gets you a whole borehole             | `.at("LOCA", [...]).frames()`     | A borehole log           |
| 2   | `post-2-typed-columns`     | Your depths are already numbers                | Frames typed as the file declares | Plasticity chart         |
| 3   | `post-3-sql-across-groups` | Ask the file a question                        | `.query()`, then `.sql()`         | Chart coloured by unit   |
| 4   | `post-4-clean-a-real-file` | What is wrong with the AGS file you were sent? | `.validate()`, `.fix()`, by hand  | Errors left after a step |
| 5   | `post-5-merge`             | Seventeen files in, one file out               | `merge`                           | The whole site, one file |

Checking and cleaning were planned as two posts and are drafted as one.

`extra-run-it-with-uv` is an optional companion, not one of the five: checking
a file with `uvx` and the `lat` command with nothing installed, and running
the series' scripts the same way.

`extra-marimo-notebook` is a second companion: the same toolkit as a marimo
notebook that queries the file through laterite's DuckDB connection, converts
grid coordinates with pyproj, and puts the boreholes on a MapLibre street map
where clicking one draws its log. The street map itself has not been seen
drawn yet; run the notebook on a local machine before the post goes out.

## What each post has to get right

Found by running each post's code on a fresh install of the released wheel
(0.17.1) against two public datasets: Fugro's Hollandse Kust (zuid) site III
boreholes from RVO, and a BGS export of Crossrail boreholes at Farringdon.

**Post 1**

- `PROJ` is not part of the fan-out. A log's header needs `ags["PROJ"]` as well.
- A hole ID that is not in the file returns empty frames with no warning (#1246).

**Post 2**

- A column is typed as the file's TYPE row declares it, so the claim holds for
  a well-typed file and not for every file. Say so, and show how to check with
  `ags.types(group)`.
- The Fugro files declare depths `2DP`: numbers. The BGS export declares most
  columns `X` and some depths `U`, so its sample and final depths are text,
  though its stratum depths are numbers (#1245).
- Use the Fugro data for this post.

**Post 3**

- Leads with laterite's own `.query().filter().select()` on one group, then
  steps up to a SQL join, then to a grouped summary. It says plainly that SQL
  fluency is not needed: an LLM writes the query from the headings and a
  plain-words question.
- `.pl()` and `.df()` on a `.sql()` result need pyarrow and pandas. On a base
  install use `pl.DataFrame(ags.sql(...))`.
- On the BGS export the depth comparison fails to bind, because the depths are
  text. It needs `TRY_CAST(... AS DOUBLE)`.
- On the Fugro file every sample lands in a stratum.

**Post 4**

- `post-4-clean-a-real-file/clean_recipe.py` takes one BGS Crossrail file from
  its findings as sent to none, in eight steps. `.fix()` clears the line
  endings, which is most of them; the rest is seven small edits by hand.
- It is a worked example for that file, not a general cleaner. On the other
  two Crossrail files it leaves one and three findings; on files from other
  projects it fails, because their groups differ.
- One description in it is a placeholder (for a legend code the source never
  defines). A real clean needs the right text.
- The borehole, stratum, sample and SPT rows come back with identical values.
  Three repeated water-strike rows are dropped; everything else that changes
  is the file's own bookkeeping: UNIT, DICT, ABBR and TRAN.
- The safe fix does nothing on the Fugro files, which are nearly clean. Use the
  BGS file.
- The manual steps need the file's own units and types passed back into
  `build_ags4` (#1244). Gaps this exposed: #1241, #1242, #1243.

**Post 5**

- Merging the seventeen Fugro files gives one file whose row counts equal the
  sum of the inputs.
- `merge` warns that no TRAN stamp was supplied. Pass `tran=TranStamp(...)`.

**All posts**

- All 541 samples in the BGS Crossrail file have a blank `SAMP_ID`. The `None`
  warning and `by="_id"` behave as the cookbook says.
- Checked on Python 3.12, 3.13 and 3.14 on Linux. Not checked on Windows or macOS.
- `laterite-quarry` 0.1.0 and `laterite-outcrop` 0.1.1 are on PyPI. Every script,
  figure line and closing pointer was run from those releases, each by the
  usage line in its own docstring.
- Each post's code should be a script that a gate runs against a small
  committed fixture, as the docs examples are (an open question in #1240).

## Data and credit

- RVO: site content is CC0 unless a document states otherwise. Credit
  "Netherlands Enterprise Agency (RVO) / Fugro".
- BGS: Open Government Licence v3.0. Credit "Contains British Geological
  Survey materials ©NERC" with the year.
- No data file is committed here.
