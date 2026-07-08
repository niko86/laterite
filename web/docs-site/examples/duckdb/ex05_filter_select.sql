-- Narrow one group to rows and columns — filter + select are plain SQL over read_ags.
-- The dtype IS the AGS type, so `loca_gl > 28` compares numbers, not strings.
-- expect-rows: 7
SELECT loca_id, loca_type, loca_gl
FROM read_ags('examples/sample_site.ags', 'LOCA')
WHERE loca_gl > 28
ORDER BY loca_id;
