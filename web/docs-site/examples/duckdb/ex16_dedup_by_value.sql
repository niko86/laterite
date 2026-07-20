-- Recipe: union two deliveries, dedup by VALUE with _content_hash.
-- _content_hash fingerprints a row's VALUES (typed, unit-aware) — the value twin
-- of _id's IDENTITY. So a borehole REVISED between phases (same LOCA_ID, changed
-- LOCA_GL) has a DIFFERENT _content_hash and BOTH versions survive, while a row
-- byte-identical in both files collapses to one. Contrast ex07, where
-- DISTINCT ON (_id) keeps a single (arbitrary) row per identity.
-- Same two phases as ex07: BH01 is revised 10.00 -> 11.50, BH02 is unchanged.
-- expect-rows: 4
SELECT DISTINCT ON (_content_hash) loca_id, loca_gl
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
ORDER BY _content_hash;
