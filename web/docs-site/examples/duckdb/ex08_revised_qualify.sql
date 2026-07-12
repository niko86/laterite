-- Recipe: a location revised between phases (same LOCA_ID, changed data) —
-- dedup on the AGS key and keep the later row. Carry a version column; QUALIFY
-- picks the winner per key (BH01 keeps the phase-2 value 11.50).
-- expect-rows: 3
SELECT loca_id, loca_gl FROM (
  SELECT *, 1 AS ver FROM read_ags_text('"GROUP","LOCA"
"HEADING","LOCA_ID","LOCA_TYPE","LOCA_GL"
"UNIT","","","m"
"TYPE","ID","PA","2DP"
"DATA","BH01","CP","10.00"
"DATA","BH02","CP","20.00"
', 'LOCA')
  UNION BY NAME
  SELECT *, 2 AS ver FROM read_ags_text('"GROUP","LOCA"
"HEADING","LOCA_ID","LOCA_TYPE","LOCA_GL"
"UNIT","","","m"
"TYPE","ID","PA","2DP"
"DATA","BH02","CP","20.00"
"DATA","BH03","CP","30.00"
"DATA","BH01","CP","11.50"
', 'LOCA')
)
QUALIFY row_number() OVER (PARTITION BY loca_id ORDER BY ver DESC) = 1
ORDER BY loca_id;
