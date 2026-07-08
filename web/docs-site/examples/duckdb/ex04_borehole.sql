-- One borehole's record set via keyed joins — LOCA to its SAMP rows on the shared key.
-- (DuckDB has no fan-out helper; you name the related groups and join them yourself.)
-- expect-rows: 4
SELECT l.loca_id, s.samp_ref, s.samp_top, s.samp_type
FROM read_ags('examples/sample_site.ags', 'LOCA') l
JOIN read_ags('examples/sample_site.ags', 'SAMP') s USING (loca_id)
WHERE l.loca_id = 'BH01'
ORDER BY s.samp_top;
