// what this shows: build_ags4() — per-group data in, byte-faithful AGS4 out —
// and which catalogs it will derive for you versus which it refuses to invent.
import { readFileSync } from "node:fs";
import { fileURLToPath } from "node:url";
import assert from "node:assert/strict";
import init, { build_ags4, validate } from "@laterite/ags4-wasm";

await init({
  module_or_path: readFileSync(
    fileURLToPath(import.meta.resolve("@laterite/ags4-wasm/ags4_wasm_bg.wasm")),
  ),
});

// An ARRAY of `{ code, headings, rows }` — rows are positional cell arrays, not
// objects. `units`/`types` are optional: omit them and they fill from the chosen
// edition's dictionary. Only the headings you supply are written, so a sparse
// group builds clean rather than padding out the whole dictionary.
const groups = [
  {
    code: "PROJ",
    headings: ["PROJ_ID", "PROJ_NAME"],
    rows: [["P1", "Demo"]],
  },
  {
    code: "LOCA",
    headings: ["LOCA_ID", "LOCA_TYPE", "LOCA_GL"],
    rows: [
      ["BH01", "CP", "23.68"],
      ["BH02", "RC", "32.49"],
    ],
  },
];

// `synthesiseMetadata` derives UNIT and TYPE from your columns (and ABBR when PA
// codes are used). It does NOT invent PROJ, DICT or TRAN: a project identity, a
// schema extension and a record of transmission are authorial facts. A fabricated
// TRAN would SATISFY Rule 14 while asserting a transmission that never happened,
// so the gap is reported instead — pass `tran` to state it.
const report = build_ags4(JSON.stringify(groups), {
  synthesiseMetadata: true,
  tran: {
    issue: "1",
    date: "2026-08-03",
    producer: "Demo Producer",
    recipient: "Demo Recipient",
    status: "Final",
  },
});

const built = validate(new TextEncoder().encode(report.text));
console.log("fixes applied:", report.fixes_applied);
console.log("valid:", built.ok, "findings:", built.finding_count);

assert.ok(report.text.includes('"GROUP","UNIT"'), "UNIT derived");
assert.ok(report.text.includes('"GROUP","TYPE"'), "TYPE derived");
assert.ok(report.text.includes('"GROUP","TRAN"'), "TRAN stated, not invented");
assert.equal(built.ok, true);
