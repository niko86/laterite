import { read } from "laterite";
import assert from "node:assert/strict";

// Read an AGS4 file. `read` takes a path, raw bytes, or { text } (the three doors).
const ags = read("examples/sample_site.ags");

// A group comes back as a born-typed arrow-js Table — the dtype *is* the TYPE row.
const loca = ags.table("LOCA");
const dtype = (name) =>
  String(loca.schema.fields.find((f) => f.name === name).type);
console.log(`LOCA_ID[0]=${loca.getChild("LOCA_ID").get(0)} LOCA_GL[0]=${loca.getChild("LOCA_GL").get(0)}`);
console.log({ LOCA_ID: dtype("LOCA_ID"), LOCA_NATE: dtype("LOCA_NATE"), LOCA_GL: dtype("LOCA_GL") });

assert.equal(dtype("LOCA_GL"), "Float64"); // 2DP → Float64 (no manual cast)
assert.equal(dtype("LOCA_NATE"), "Float64"); // 2DP → Float64
assert.equal(dtype("LOCA_ID"), "Utf8"); // ID → Utf8
assert.equal(typeof loca.getChild("LOCA_GL").get(0), "number"); // real JS numbers
