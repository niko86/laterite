# what this shows: drop to raw SQL to join across groups, count samples per location.
import laterite

rel = laterite.read("examples/sample_site.ags").sql(
    "SELECT l.LOCA_ID, count(*) n FROM SAMP s JOIN LOCA l USING (LOCA_ID) "
    "GROUP BY 1 ORDER BY 1"
)

# rel is a DuckDBPyRelation (terminal); materialise to polars with .pl().
df = rel.pl()
print(df)

assert hasattr(rel, "pl")
assert df.height >= 1 and "n" in df.columns
