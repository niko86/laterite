-- Run the numbered rules over a file — one row per finding; clean = zero rows.
-- expect-rows: 0
SELECT rule, line, "group", "desc" FROM validate_ags('examples/sample_site.ags');
