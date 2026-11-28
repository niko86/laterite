# Blog series drafts: "one borehole's whole story"

Working notes for the first series of posts on laterite.dev. Nothing here is
published or built. Where the blog lives, and how a post is rendered, is open
in #1240; this folder is a holding place until that is decided.

Each post has a folder holding a first draft: `linkedin.md` (the short post),
`article.md` (the longer one) and the script the article quotes. Every script
was run on the released wheel against the public data named below, and the
output in the drafts is what it printed.

## Before a post goes out

```
uv run check_posts.py
```

One command, on a local machine. It fetches the two datasets, runs every
script and the notebook, and redraws the images to compare with the committed
ones. It ends by listing what it cannot check, which is to be done by hand.

`figures.py` draws each post's figure and `code_cards.py` draws its code as an
image; both write into the post's folder.

## The shape of a post

- A short LinkedIn post: a few lines of text, two images and a link. LinkedIn
  shows code as plain text, so the fenced blocks in `linkedin.md` are not
  posted as text: `code_cards.py` draws them as `code.png`, which goes out as
  the first image, with the figure second.
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
| 0   | `post-0-run-it-with-uv`    | Check a file with nothing installed            | `uvx` and the `lat` command       | The code card only       |
| 1   | `post-1-one-borehole`      | One line gets you a whole borehole             | `.at("LOCA", [...]).frames()`     | A borehole log           |
| 2   | `post-2-typed-columns`     | Your depths are already numbers                | Frames typed as the file declares | Plasticity chart         |
| 3   | `post-3-sql-across-groups` | Ask the file a question                        | `.query()`, then `.sql()`         | Chart coloured by unit   |
| 4   | `post-4-clean-a-real-file` | What is wrong with the AGS file you were sent? | `.validate()`, `.fix()`, by hand  | Errors left after a step |
| 5   | `post-5-merge`             | Seventeen files in, one file out               | `merge`                           | The whole site, one file |

Checking and cleaning were planned as two posts and are drafted as one.

Post 0 is the opener and goes out first: it has the lowest barrier of
anything here. Posts 1 to 5 keep their numbers and their "Post N of 5" headers.

The plan is one post a day for a week: post 0 on the first day, posts 1 to 5
on the next five, and the notebook on the seventh. The LinkedIn texts are
written for that: post 0 says "this week", and post 5 closes it.

`extra-marimo-notebook` is the seventh day's piece: the same toolkit as a marimo
notebook that queries the file through laterite's DuckDB connection, converts
grid coordinates with pyproj, and puts the boreholes on a MapLibre street map
where clicking one draws its log. The street map itself has not been seen
drawn yet; run the notebook on a local machine before the post goes out.

## The older tour notebook

`examples/laterite_tour.py` is an earlier marimo notebook that tours the whole
toolkit on the sample file. Checked while writing these drafts: it still runs
from top to bottom in its own sandbox, and a release gate keeps its laterite
pin current. But no job runs it, and nothing in the README, the docs site or
the landing page links to it. Its map uses folium with CartoDB tiles, which
folium now warns need an API key. It is not used by this series. Whether to
retire it, link it, or fold the new notebook's map into it is undecided; if it
goes, `ags-wiki/design/reliquary.md` is where that is recorded first.

## Fix in laterite first

These posts carry caveats for gaps in laterite. The plan is to close the gaps
before posting, then take the caveat out and re-run `check_posts.py`, which
will fail on any number that changed.

| Issue | Post | What comes out when it is fixed                              |
| ----- | ---- | ------------------------------------------------------------ |
| #1246 | 1    | The warning about an unknown hole ID returning empty frames  |
| #1245 | 2    | The note that type `U` depths in the BGS export read as text |
| #1241 | 4    | Step 3 by hand: ISO dates under a `dd/mm/yyyy` unit          |
| #1242 | 4    | Step 2 by hand, for rows identical in every field            |
| #1243 | 4    | Step 5 by hand: the undeclared concatenator                  |
| #1244 | 4    | Passing the file's units and types back into `build_ags4`    |
| #1249 | 5    | The types check after a merge that turned a number into text |

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
- BGS holds and shares this data; it did not produce it. The owner is whoever
  commissioned the investigation (here, Crossrail). So faults in the data are
  not BGS's, except what its export adds: the `BGS_` columns.
- BGS: Open Government Licence v3.0. Credit "Contains British Geological
  Survey materials ©NERC" with the year.
- No data file is committed here.
