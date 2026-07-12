-- Inspect a file's own structure as queryable tables. ags_groups lists the
-- groups (with row + heading counts); ags_headings gives each group's headings
-- with the ags_type -> sql_type mapping. Both take just (path).
-- expect-rows: 6
SELECT "group", n_rows, n_headings FROM ags_groups('examples/sample_site.ags');
SELECT heading, unit, ags_type, sql_type
FROM ags_headings('examples/sample_site.ags') WHERE "group" = 'LOCA' ORDER BY ordinal;
