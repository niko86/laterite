-- Inspect the AGS4 dictionary bundled inside the extension (no download):
-- ags_dictionary() is every group/heading with ags_type/sql_type; and
-- ags_relationships() is the parent/child (KEY) graph.
-- expect-rows: 47
SELECT child, parent, shared_keys FROM ags_relationships() WHERE parent = 'LOCA';
SELECT heading, ags_type, sql_type FROM ags_dictionary() WHERE "group" = 'LOCA' ORDER BY ordinal;
