-- One group as a table — columns come back born-typed (the dtype IS the TYPE row).
-- expect-rows: 2
SELECT loca_id, loca_nate, loca_gl
FROM read_ags('examples/sample_site.ags', 'LOCA')
ORDER BY loca_id LIMIT 2;
