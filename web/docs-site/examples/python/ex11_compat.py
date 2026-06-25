# EX11 — the python-ags4 drop-in: laterite.compat is a faithful AGS4_to_dataframe shim.
from laterite import compat as AGS4

result = AGS4.AGS4_to_dataframe("examples/sample_site.ags")

# python-ags4 returns a (tables, headings) 2-tuple; tables maps group -> pandas DataFrame.
print(type(result), list(result[0])[:5])
print(result[0]["LOCA"].shape)

assert isinstance(result, tuple) and len(result) == 2 and "LOCA" in result[0]
