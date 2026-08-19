# /// script
# requires-python = ">=3.12"
# dependencies = ["laterite==0.11.0"]
# ///
"""Docs example — run it with `uv run ex19_read_text_door.py`, from anywhere.

Everything above the `[start:code]` marker is machinery the page does not
show: the PEP 723 header that makes the file self-installing.

No fixture arm: this one carries its own AGS4 as a string, which is the whole
point of the `text=` door it demonstrates.
"""

# --8<-- [start:code]
import laterite

# A minimal AGS4 string — GROUP / HEADING / UNIT / TYPE rows, then DATA.
ags4_text = """"GROUP","LOCA"
"HEADING","LOCA_ID","LOCA_GL"
"UNIT","","m"
"TYPE","ID","2DP"
"DATA","BH01","23.68"
"DATA","BH02","32.49"
"""

ags = laterite.read(text=ags4_text)  # the text= door
loca = ags["LOCA"]
print(loca)
print({h: str(loca[h].dtype) for h in ("LOCA_ID", "LOCA_GL")})

assert str(loca["LOCA_GL"].dtype) == "Float64"  # 2DP → Float64, from text
assert loca.height == 2
# --8<-- [end:code]
