// what this shows: the ParsedDataset lifecycle — read() once, pull each group
// off it, then free it. Getting the order wrong is the one way to misuse this
// API.
import { readFileSync } from "node:fs";
import { fileURLToPath } from "node:url";
import assert from "node:assert/strict";
import init, { read } from "@laterite/ags4-wasm";

await init({
  module_or_path: readFileSync(
    fileURLToPath(import.meta.resolve("@laterite/ags4-wasm/ags4_wasm_bg.wasm")),
  ),
});

// `read` returns a handle into wasm memory, not a copy of your data.
const dataset = read(new Uint8Array(readFileSync("examples/sample_site.ags")));

try {
  console.log(dataset.group_codes().join(" "));

  // `meta` describes the columns, `rows_json` carries the values, and the two
  // are POSITIONAL against each other: headings[i] names rows[r][i]. Each
  // group is built LAZILY on the call and dropped on return, so the dataset has
  // to outlive every pull — hold it, don't chain off the call.
  const meta = dataset.meta("LOCA");
  const rows = JSON.parse(dataset.rows_json("LOCA"));

  // Values arrive TYPED, off the file's own TYPE row — a `2DP` heading is a
  // JSON number here, not the source text, and a blank cell is null. The cast
  // is the same one the Python wheel and the DuckDB extension apply.
  const nate = meta.headings.indexOf("LOCA_NATE");
  console.log(`${meta.headings[nate]} is ${meta.types[nate]}:`, rows[0][nate]);

  assert.ok(dataset.group_codes().includes("LOCA"));
  assert.equal(typeof rows[0][nate], "number");
} finally {
  // Free before the next parse, or wasm memory holds both datasets at once.
  // `using dataset = read(...)` does this for you where `Symbol.dispose` is
  // supported; the explicit call is the portable form.
  dataset.free();
}
