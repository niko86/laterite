// The first perf harness for laterite-node. The surface had NO benchmark of any
// kind, so its read cost was invisible to CI — a standing blind spot (node was
// the only surface with no perf floor). This mirrors the Rust `typed_read_file`
// and the Python read bench: materialize every group of a forge fixture to a
// typed Arrow Table over the shipped native path (read → `tableIpc` → apache-arrow
// decode), on the 25 MB rung so the numbers sit on the same axis as the others.
//
// `table(code)` computes the content-addressed keychain (`_id`/`_parent_id`) and
// then strips it on the default (keys-less) call, so that keychain cost rides
// inside these numbers — it is exactly candidate #6, which a `withKeys=false`
// escape on `tableIpc` would remove. This harness makes that rankable: the gap
// between "read only" and "read + table" is the typed-build + keychain cost.
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

    // The full default typed read: parse + `table(code)` for every group. The
    // keychain is computed and then stripped (#6) — this is the number that
    // candidate improves.
    bench("read + table(all groups) [large]", () => {
      const f = read(bytes);
      for (const code of f.groups) f.table(code);
    });

    // The keyed variant keeps `_id`/`_parent_id` rather than stripping them. Both
    // pay the keychain today; the gap to the default is only the strip, so this
    // pins that the keychain — not the strip — is the cost #6 targets.
    bench("read + table(all, keys) [large]", () => {
      const f = read(bytes);
      for (const code of f.groups) f.table(code, { keys: true });
    });
  }
});
