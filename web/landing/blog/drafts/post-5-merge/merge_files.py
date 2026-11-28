"""Merge a folder of AGS4 files into one, and check nothing went missing.

uv run --with laterite merge_files.py folder/ merged.ags
"""

import sys
from pathlib import Path

import laterite
from laterite import TranStamp

files = sorted(p for p in Path(sys.argv[1]).iterdir() if p.suffix.lower() == ".ags")

# The merged file is a new transmission, so it gets its own TRAN row.
stamp = TranStamp(
    issue="1",
    date="2026-11-28",
    producer="Example Ltd",
    recipient="Example Client",
    status="Final",
    description="All boreholes, merged",
)
result = laterite.merge(*files, tran=stamp)
for warning in result.warnings:
    print("warning:", warning["message"])

merged = laterite.read(result.bytes)

# Row counts in, row counts out.
for code in ("LOCA", "GEOL", "SAMP"):
    before = sum(f[code].height for f in map(laterite.read, files) if code in f.groups)
    print(f"{code}  {before:>4} in  {merged[code].height:>4} out")

report = merged.validate().report
print(f"{report.errors} errors, {report.warnings} warnings")

Path(sys.argv[2]).write_bytes(result.bytes)
