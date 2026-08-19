# /// script
# requires-python = ">=3.12"
# dependencies = ["laterite==0.11.0"]
# ///
"""Docs example — run it with `uv run ex15_fix.py`, from anywhere.

Everything above the `[start:code]` marker is machinery the page does not
show: the PEP 723 header that makes the file self-installing.
"""

# --8<-- [start:code]
# what this shows: .fix() mechanically repairs a dirty AGS4 file, non-destructively, into a NEW handle.
from laterite import read

# A dirty file: the data row is SHORT — fewer fields than the HEADING row (Rule 4).
# (AGS4 lines are CRLF-terminated; keep them so the only defect is the short row.)
dirty_text = (
    '"GROUP","LOCA"\r\n'
    '"HEADING","LOCA_ID","LOCA_TYPE","LOCA_NATE"\r\n'
    '"UNIT","","",""\r\n'
    '"TYPE","ID","PA","2DP"\r\n'
    '"DATA","BH01","BH"\r\n'  # <- only 3 fields, HEADING declares 4
)

dirty = read(text=dirty_text)
fixed = dirty.fix()  # returns a NEW Ags4File; the original is untouched

kinds = [a["kind"] for a in fixed.fix_report.applied]
print(fixed.fix_report.applied[0]["kind"])

assert fixed is not dirty  # non-destructive: a fresh handle
assert fixed.fix_report.applied[0]["kind"] == "pad_short_row"
assert "pad_short_row" in kinds  # the short row was padded to width
# --8<-- [end:code]
