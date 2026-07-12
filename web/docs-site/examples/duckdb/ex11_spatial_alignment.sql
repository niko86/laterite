-- Recipe: boreholes near an alignment. The born-typed easting/northing become
-- geometry with no parsing and no CAST. Needs DuckDB's spatial extension
-- (self-installed here; the monthly gate runner has network).
-- expect-rows: 7
INSTALL spatial; LOAD spatial;
SELECT loca_id
FROM read_ags('examples/sample_site.ags', 'LOCA')
WHERE ST_DWithin(
        ST_Point(loca_nate, loca_natn),
        ST_GeomFromText('LINESTRING(451000 162000, 451400 162100)'),
        100)
ORDER BY loca_id;
