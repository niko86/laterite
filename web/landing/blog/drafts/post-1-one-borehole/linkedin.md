# Post 1: LinkedIn

![Borehole log of F1, drawn from the record set](log.png)

---

Someone sends you an AGS file with a dozen boreholes in it. You want one of them: its strata, its samples, its SPTs.

In laterite that is one line.

```python
import laterite

ags = laterite.read("your_file.ags")
frames = ags.at("LOCA", ["F1"]).frames()
```

`frames` is a dict of polars frames, one per group, each cut down to that hole:

```
LOCA     1 rows
GEOL    20 rows
ISPT    20 rows
SAMP   100 rows
```

No loop over groups, no filter written eight times, no key columns to remember. laterite reads which groups hang off the hole from the file itself.

The log below is borehole F1 from Crossrail's Farringdon investigation, drawn from those frames: made ground and alluvium over London Clay, the Lambeth Group, and Thanet Sand from 33.7 m.

The full script, and the two things to watch for, are in the article: [link]

`pip install laterite`

Contains British Geological Survey materials ©NERC 2026.

#AGS4 #geotechnics #python
