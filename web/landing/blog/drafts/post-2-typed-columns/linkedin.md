# Post 2: LinkedIn

![Plasticity chart of 53 Atterberg results](plasticity.png)

---

Every AGS file tells you what type each column is. Most tools read it all as text anyway, and you spend the first twenty lines of every script turning "12.50" back into 12.5.

An AGS file has a TYPE row under every heading: `2DP`, `1DP`, `DT`, `X`. laterite reads it and gives you columns already typed that way.

```python
import laterite
import polars as pl

ags = laterite.read("your_file.ags")
limits = ags["LLPL"]
```

What the file declares, and what you get:

```
SAMP_TOP   2DP  Float64
LLPL_LL    1DP  Float64
LLPL_PL    1DP  Float64
LLPL_PI    1DP  Float64
LLPL_LAB   X    String
```

Depths are floats. Dates are datetimes. Counts are integers. So this works with no casting:

```python
high = limits.filter(pl.col("LLPL_LL") >= 50)
```

The chart is 53 Atterberg results from Fugro's boreholes for the Hollandse Kust (zuid) wind farm, plotted straight from the frame.

One catch worth knowing: laterite types a column as the file declares it. If the file says a water content is text, you get text. The article shows how to spot that in one line: [link]

Data: Netherlands Enterprise Agency (RVO) / Fugro.

#AGS4 #geotechnics #python
