-- Joins across groups are plain SQL — count samples per location.
-- expect-rows: 14
SELECT l.loca_id, count(*) AS n
FROM read_ags('examples/sample_site.ags', 'SAMP') s
JOIN read_ags('examples/sample_site.ags', 'LOCA') l USING (loca_id)
GROUP BY 1 ORDER BY 1;
