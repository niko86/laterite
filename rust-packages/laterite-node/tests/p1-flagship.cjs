// P1 — the native engine core: parse→Arrow IPC (typing parity), validate,
// byte-faithful emit, and data→AGS4 via Arrow IPC.
const assert = require("node:assert");
const { tableFromIPC, tableFromArrays, tableToIPC } = require("apache-arrow");
const native = require("../index.js");

const ags =
  '"GROUP","LOCA"\r\n' +
  '"HEADING","LOCA_ID","LOCA_GL","LOCA_CKED","LOCA_STAR"\r\n' +
  '"UNIT","","m","",""\r\n' +
  '"TYPE","ID","2DP","YN","DT"\r\n' +
  '"DATA","BH01","12.30","Y","2023-02-22"\r\n' +
  '"DATA","BH02","13.00","N","2023-03-01"\r\n';

// --- parse → typed Arrow IPC (the boundary, typed like Python/wasm) ---
const reading = native.parseArrow(null, ags, null);
assert.deepStrictEqual(reading.groupCodes(), ["LOCA"]);
const meta = reading.meta("LOCA");
assert.deepStrictEqual(meta.types, ["ID", "2DP", "YN", "DT"]);

const table = tableFromIPC(reading.tableIpc("LOCA"));
const t = (name) => table.getChild(name).type.toString();
console.log("types:", meta.headings.map((h) => `${h}:${t(h)}`).join("  "));
assert.match(t("LOCA_ID"), /Utf8/);
assert.match(t("LOCA_GL"), /Float64/);
assert.match(t("LOCA_CKED"), /Bool/);
assert.match(t("LOCA_STAR"), /Timestamp|Date/);
assert.strictEqual(table.getChild("LOCA_GL").get(0), 12.3);
assert.strictEqual(reading.tableIpc("NOPE"), null);

// --- validate (runCheck: path, text, dictVersion, warnings, fyi, files, enc) ---
const rep = native.runCheck(null, ags, null, false, false, false, null);
console.log("runCheck:", { ok: rep.ok, dictVersion: rep.dictVersion, resolution: rep.resolution, count: rep.count });
assert.strictEqual(rep.ok, true); // parsed fine — `ok` means validatable, not valid
assert.strictEqual(typeof rep.dictVersion, "string");
assert.ok(Array.isArray(rep.findings));
assert.strictEqual(rep.count, rep.findings.length);
assert.ok(rep.count > 0, "a LOCA-only file (no PROJ/TRAN) has findings");
assert.match(rep.json, /"findings"/); // byte-faithful ags4-check --json

// --- byte-faithful re-emit ---
const reEmitted = reading.emit();
assert.match(reEmitted, /"GROUP","LOCA"/);
assert.match(reEmitted, /"DATA","BH01","12\.30","Y"/);
assert.match(reEmitted, /\r\n/);
assert.deepStrictEqual(native.parseArrow(null, reEmitted, null).groupCodes(), ["LOCA"]);

// --- data → AGS4 via Arrow IPC (build arrow-js tables → emit) ---
const toIpc = (tbl) => Buffer.from(tableToIPC(tbl, "stream"));
const proj = tableFromArrays({ PROJ_ID: ["P1"], PROJ_NAME: ["Demo project"] });
const loca = tableFromArrays({ LOCA_ID: ["BH01", "BH02"], LOCA_GL: Float64Array.from([12.3, 13.0]) });
const out = native.emitAgs4FromIpc(
  [{ code: "PROJ", ipc: toIpc(proj) }, { code: "LOCA", ipc: toIpc(loca) }],
  "4.1.1",
  "autofix",
);
const text = out.bytes.toString("utf8");
console.log("emit fixesApplied:", out.fixesApplied, "| findings keys:", Object.keys(JSON.parse(out.findingsJson)).length);
assert.match(text, /"GROUP","PROJ"/);
assert.match(text, /"TYPE","ID","2DP"/); // dict-filled
assert.match(text, /"DATA","BH01","12\.30"/); // Float64 12.3 → canonical 2DP
assert.ok(Buffer.isBuffer(out.bytes));

console.log("P1: PASS");
