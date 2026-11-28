# Post 4: LinkedIn

![Errors left after each cleaning step](steps.png)

![Errors in the file as sent, by rule](by_rule.png)

---

I validated a real AGS file from a public archive. 1,488 errors.

That number looks worse than it is, and working out why is most of the job.

```python
import laterite

ags = laterite.read("your_file.ags")
report = ags.validate().report
report.findings.group_by("rule").len()
```

The findings come back as a table, so you can count them instead of scrolling through them. 1,394 of the 1,488 are one thing: the line endings. Every line in the file trips the same rule.

```python
fixed = ags.fix()
```

94 left. `.fix()` only makes repairs that cannot change what the data says, so it stops there.

The rest took seven small edits by hand, each one a decision: dates written one way and declared another, rows repeated, codes used but never defined, columns with names the format does not allow. About a hundred lines of polars, then the file is rebuilt and validates clean.

Not one borehole, stratum or sample value changed. Nearly everything wrong was in the file's bookkeeping.

The file is a BGS export of Crossrail boreholes at Farringdon. The article walks through all eight steps and is honest about which ones laterite should be doing for you: [link]

Contains British Geological Survey materials ©NERC 2026.

#AGS4 #geotechnics #python #dataquality
