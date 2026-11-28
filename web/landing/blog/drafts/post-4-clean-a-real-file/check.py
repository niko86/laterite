"""Check an AGS4 file, read the findings, and apply the safe fixes.

uv run --with laterite check.py your_file.ags fixed.ags
"""

import sys

import laterite
import polars as pl

ags = laterite.read(sys.argv[1])
report = ags.validate().report
print(f"{report.errors} errors, {report.warnings} warnings")

# The findings are a polars frame: count them, group them, filter them.
findings = report.findings
by_rule = (
    findings.group_by("rule")
    .agg(pl.len().alias("n"), pl.col("desc").first().alias("for_example"))
    .sort("n", descending=True)
)
with pl.Config(fmt_str_lengths=70, tbl_rows=20):
    print(by_rule)

# The safe fixes: mechanical repairs that cannot change what the data says.
fixed = ags.fix()
for change in fixed.fix_report.applied:
    print("fixed:", change["label"])
print(f"{fixed.validate().report.errors} errors left")

fixed.save(sys.argv[2])
