// laterite — a runnable Node.js tour.
//
//   npm install laterite
//   node examples/node_tour.mjs
//
// The same clean-room Rust AGS4 engine as the Python package, surfaced for
// Node with born-typed apache-arrow output. (The marimo notebook
// examples/laterite_tour.py is the Python + DuckDB-extension showcase; molab
// is a Python sandbox, so this Node tour runs locally.)

import { writeFileSync, rmSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";

import { read, validate, buildAgs4 } from "laterite";

// A small embedded AGS4 file (synthetic, MIT) so the script is self-contained.
const SAMPLE = `"GROUP","PROJ"
"HEADING","PROJ_ID","PROJ_NAME"
"UNIT","",""
"TYPE","ID","X"
"DATA","123456","Node tour fixture (hand-authored, MIT)"

"GROUP","TRAN"
"HEADING","TRAN_ISNO","TRAN_AGS"
"UNIT","",""
"TYPE","X","X"
"DATA","1","4.1"

"GROUP","LOCA"
"HEADING","LOCA_ID","LOCA_TYPE","LOCA_NATE","LOCA_NATN","LOCA_FDEP"
"UNIT","","","m","m","m"
"TYPE","ID","PA","2DP","3DP","2DP"
"DATA","BH01","CP","451000.10","162000.250","18.50"
"DATA","BH02","RC","451120.40","162005.100","22.75"
`;

const path = join(tmpdir(), "laterite_node_tour.ags");
writeFileSync(path, SAMPLE);

try {
  // 1 · Read — born-typed apache-arrow tables.
  const ags = read(path); // or read(bytes) / read(undefined, { text: SAMPLE })
  console.log("1 · read");
  console.log("   groups:", ags.groups.join(", "));
  const loca = ags.table("LOCA");
  // LOCA_NATE is a `2DP` heading → a JS number, not a string.
  console.log("   BH01 easting (born-typed):", loca.getChild("LOCA_NATE")?.get(0));

  // 2 · Validate — every numbered AGS4 rule, byte-identical JSON to lat-check.
  const report = validate(path);
  console.log("\n2 · validate");
  console.log("   valid:", report.isValid, "· findings:", report.findings.length);
  for (const f of report.findings.slice(0, 3)) {
    console.log(`   - ${f.rule}${f.group ? " [" + f.group + "]" : ""}: ${f.desc}`);
  }

  // 3 · Produce — build valid AGS4 from plain rows (or a typed PROJ/LOCA graph).
  const built = buildAgs4(
    new Map([
      ["PROJ", [{ PROJ_ID: "P1", PROJ_NAME: "Built in Node" }]],
      ["LOCA", [
        { LOCA_ID: "BH01", LOCA_GL: 12.5 },
        { LOCA_ID: "BH02", LOCA_GL: 13.75 },
      ]],
    ]),
    { mode: "autofix" },
  );
  console.log("\n3 · buildAgs4");
  console.log("   produced", built.text.length, "chars of AGS4");
  console.log("   first line:", built.text.split(/\r?\n/)[0]);

  console.log("\nlaterite Node tour complete · npm install laterite");
} finally {
  rmSync(path, { force: true });
}
