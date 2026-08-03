// what this shows: the ParsedDataset lifecycle — read() once, pull each group's
// Arrow IPC off it, then free it. Getting the order wrong is the one way to
// misuse this API.
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

  // Each group's batch is built LAZILY here and dropped on return, so the
  // dataset has to outlive every pull — hold it, don't chain off the call.
  // `keys: true` prepends the content-addressed `_id`/`_parent_id` columns (the
  // same UUIDv8s the wheel, Node and DuckDB produce), which is what makes a
  // cross-group join resolve when you feed these batches to duckdb-wasm. Leave
  // it off for a plain typed frame.
  const loca = dataset.arrow_ipc("LOCA", true, false);
  console.log("LOCA arrow ipc bytes:", loca.byteLength > 0);

  assert.ok(dataset.group_codes().includes("LOCA"));
  assert.ok(loca.byteLength > 0);
} finally {
  // Free before the next parse, or wasm memory holds both datasets at once.
  // `using dataset = read(...)` does this for you where `Symbol.dispose` is
  // supported; the explicit call is the portable form.
  dataset.free();
}
