import laterite

# Read an AGS4 file. `read` takes a path, or text=… / data=… (the three doors).
ags = laterite.read("examples/sample_site.ags")

# A group comes back as a born-typed polars frame — the dtype *is* the TYPE row.
loca = ags["LOCA"]
print(loca.select("LOCA_ID", "LOCA_NATE", "LOCA_GL").head(2))
print({h: str(loca[h].dtype) for h in ("LOCA_ID", "LOCA_NATE", "LOCA_GL")})

assert str(loca["LOCA_GL"].dtype) == "Float64"  # 2DP  → Float64 (no manual cast)
assert str(loca["LOCA_NATE"].dtype) == "Float64"  # 2DP → Float64
assert str(loca["LOCA_ID"].dtype) == "String"  # ID → String
