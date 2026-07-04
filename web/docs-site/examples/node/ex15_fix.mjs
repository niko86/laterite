// what this shows: .fix() mechanically repairs a dirty AGS4 file, non-destructively, into a NEW handle.
import { read } from "laterite";
import assert from "node:assert/strict";

// A dirty file: the data row is SHORT — fewer fields than the HEADING row (Rule 4).
// (AGS4 lines are CRLF-terminated; keep them so the only defect is the short row.)
const dirtyText =
  '"GROUP","LOCA"\r\n' +
  '"HEADING","LOCA_ID","LOCA_TYPE","LOCA_NATE"\r\n' +
  '"UNIT","","",""\r\n' +
  '"TYPE","ID","PA","2DP"\r\n' +
  '"DATA","BH01","BH"\r\n'; // <- only 3 fields, HEADING declares 4

const dirty = read(undefined, { text: dirtyText });
const fixed = dirty.fix(); // returns a NEW Ags4File; the original is untouched

console.log(fixed.fixReport.applied[0].kind);

assert.notEqual(fixed, dirty); // non-destructive: a fresh handle
assert.equal(fixed.fixReport.applied[0].kind, "pad_short_row");
