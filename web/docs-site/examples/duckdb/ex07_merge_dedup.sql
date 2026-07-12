-- Recipe: merge two deliveries, dedup by content identity (_id).
-- The content _id is stable per borehole IDENTITY (parent + AGS key), so the
-- same location from either file collapses to one row — no key columns to name.
-- A revised row keeps its _id (see ex08 to pick a version). Phases are inline
-- via read_ags_text so the example is self-contained.
-- expect-rows: 3
SELECT DISTINCT ON (_id) loca_id, loca_gl
FROM (
  SELECT * FROM read_ags_text('"GROUP","LOCA"
"HEADING","LOCA_ID","LOCA_TYPE","LOCA_GL"
"UNIT","","","m"
"TYPE","ID","PA","2DP"
"DATA","BH01","CP","10.00"
"DATA","BH02","CP","20.00"
', 'LOCA')
  UNION ALL
  SELECT * FROM read_ags_text('"GROUP","LOCA"
"HEADING","LOCA_ID","LOCA_TYPE","LOCA_GL"
"UNIT","","","m"
"TYPE","ID","PA","2DP"
"DATA","BH02","CP","20.00"
"DATA","BH03","CP","30.00"
"DATA","BH01","CP","11.50"
', 'LOCA')
)
ORDER BY _id;
