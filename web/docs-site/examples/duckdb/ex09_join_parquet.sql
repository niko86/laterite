-- Recipe: join AGS4 to external data. read_ags is just a table, so it joins
-- straight to Parquet. Fabricate a planning-zones table, then tag each borehole.
-- expect-rows: 14
COPY (
  SELECT loca_id AS parcel, 'Zone ' || chr((65 + (row_number() OVER ()) % 3)::INTEGER) AS zone
  FROM read_ags('examples/sample_site.ags', 'LOCA')
) TO 'planning_zones.parquet' (FORMAT parquet);
SELECT l.loca_id, l.loca_gl, z.zone
FROM read_ags('examples/sample_site.ags', 'LOCA') l
JOIN 'planning_zones.parquet' z ON z.parcel = l.loca_id
ORDER BY l.loca_id;
