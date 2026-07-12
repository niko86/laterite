-- Recipe: export a typed group straight to Parquet for a warehouse. The AGS
-- types survive the round-trip (LOCA_GL stays DOUBLE, not text).
-- expect-rows: 14
COPY (SELECT * FROM read_ags('examples/sample_site.ags', 'LOCA')) TO 'loca.parquet' (FORMAT parquet);
SELECT * FROM 'loca.parquet';
