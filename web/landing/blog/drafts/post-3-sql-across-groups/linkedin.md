# Post 3: LinkedIn

![The code in this post, drawn as an image](code.png)

![The same results coloured by soil unit](by_unit.png)

---

Which of these results are high plasticity? That is a question about one table, and laterite has its own query for it:

```python
import laterite

ags = laterite.read("your_file.ags")
high = (
    ags.query("SELECT * FROM LLPL")
    .filter("LLPL_LL >= 50")
    .select("LOCA_ID", "SAMP_TOP", "LLPL_LL", "LLPL_PI")
    .frame()
)
```

Which soil unit did each one come from? AGS does not say. A lab result knows its borehole and depth; a stratum knows its top and base. Matching them takes a join, so laterite makes every group a SQL table:

```sql
SELECT g.GEOL_STAT AS unit, count(*) AS tests,
       round(avg(l.LLPL_LL), 1) AS mean_ll,
       round(avg(l.LLPL_PI), 1) AS mean_pi
FROM LLPL l
JOIN GEOL g
  ON g.LOCA_ID   = l.LOCA_ID
 AND l.SAMP_TOP >= g.GEOL_TOP
 AND l.SAMP_TOP <  g.GEOL_BASE
GROUP BY ALL ORDER BY unit
```

```
unit  tests  mean_ll  mean_pi
B1        2     56.5     32.5
B2       12     48.3     24.3
C2       33     37.8     18.3
D         6     44.3     23.0
```

Not fluent in SQL? Nor am I. An LLM writes this well if you give it the column names and ask in plain words. What to paste in, and how to check the answer: [link]

Data: Netherlands Enterprise Agency (RVO) / Fugro.

#AGS4 #geotechnics #python #sql
