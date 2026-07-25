// The first perf harness for laterite-node. The surface had NO benchmark of any
// kind, so its read cost was invisible to CI — a standing blind spot (node was
// the only surface with no perf floor). This mirrors the Rust `typed_read_file`
// and the Python read bench: materialize every group of a forge fixture to a
// typed Arrow Table over the shipped native path (read → `tableIpc` → apache-arrow
// decode), on the 25 MB rung so the numbers sit on the same axis as the others.
//
// The default `table(code)` no longer computes the content-addressed keychain
// (`_id`/`_parent_id`): candidate #6 LANDED (T6), so a keys-less read skips it
// via `tableIpc(code, hash, withKeys=false)` instead of building-then-stripping.
// That took the default `read + table(all)` from 692 → 152 ms (−78%); the keychain
// (~96% of the native build) is now paid only by the explicit keyed variant. The
// gap between "read only" and "read + table (keys)" below is that keychain cost.
//
// Run: `npm run bench` (needs the native addon built — `npm run build` — and the
// fixture generated — `tools/gen-bench-fixtures.sh`). Not part of `vitest run`.
import { existsSync, readFileSync } from "node:fs";
import { fileURLToPath } from "node:url";

import { bench, describe } from "vitest";

import { read } from "../ts/index";

const fixture = fileURLToPath(
  new URL("../../../output/bench-fixtures/large.ags", import.meta.url),
);

describe("node/read", () => {
  if (!existsSync(fixture)) {
    // Keep the suite runnable on a clean checkout — a skipped bench, like the
    // Rust benches, rather than a hard failure.
    bench.skip(
      "large — fixture absent (run tools/gen-bench-fixtures.sh)",
      () => {},
    );
  } else {
    const bytes = readFileSync(fixture);

    // Parse only — the floor the typed materialization builds on.
    bench("read [large]", () => {
      read(bytes);
    });

    // The full default typed read: parse + `table(code)` for every group. Post-#6
    // the keychain is SKIPPED here (keys-less native build), so this is now close
    // to parse + the bare typed build.
    bench("read + table(all groups) [large]", () => {
      const f = read(bytes);
      for (const code of f.groups) f.table(code);
    });

    // The keyed variant keeps `_id`/`_parent_id`, so it still pays the keychain —
    // the ~490 ms gap to the default above is exactly that keychain, now charged
    // only when a caller actually asks for the keys (#6).
    bench("read + table(all, keys) [large]", () => {
      const f = read(bytes);
      for (const code of f.groups) f.table(code, { keys: true });
    });
  }
});
