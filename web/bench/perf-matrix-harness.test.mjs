// The wasm matrix harness's pure seams (#824), mirroring the node lane's
// harness tests (`laterite-node/test/perf-matrix-harness.test.ts`) where the
// seams are shared: the mistakes worth pinning are the ones that would move
// every cell at once — a cap disagreement between surfaces, a refusal a
// reader could mistake for a small number. The swap/maxRSS seams have no
// twin here: this lane's instrument is wasm linear-memory high-water, which
// no amount of host paging can move (see the harness header).
import { describe, expect, it } from "vitest";

import {
  buildOutput,
  median,
  memCell,
  memRungAllowed,
  refusalCell,
  throughputMbS,
} from "./perf-matrix.mjs";

describe("wasm perf-matrix harness seams", () => {
  it("throughput is decimal MB/s and never divides by zero", () => {
    // 5 MB in 10 ms → 500 MB/s (decimal MB, matching forge's parse_size).
    expect(throughputMbS(5_000_000, 10.0)).toBeCloseTo(500.0, 9);
    expect(throughputMbS(1_000, 0)).toBe(0);
  });

  it("median picks the middle sample, input order irrelevant", () => {
    expect(median([3, 1, 2])).toBe(2);
    expect(median([7])).toBe(7);
    // Even count: the upper-middle sample, as in the rust/node harnesses.
    expect(median([4, 1, 3, 2])).toBe(3);
  });

  it("mem cap admits the 265MB rung and refuses 524MB", () => {
    // The pinned rung sizes (tools/readme-bench-fixtures.json): epic #820
    // decision 7, in agreement with the rust/node/python harnesses.
    expect(memRungAllowed(275_510_179)).toBe(true);
    expect(memRungAllowed(549_703_139)).toBe(false);
  });

  it("mem cells carry the instrument label and never the peak-RSS key", () => {
    // The #824 two-claims rule: linear-memory high-water is a DIFFERENT
    // claim from the other surfaces' fresh-child peak RSS, so a wasm cell
    // must be labelled by instrument and must not be shaped like an RSS
    // cell — no reader (or merger) may fold it into a peak-RSS column.
    const measured = memCell(1_500_000, 1_000_000);
    expect(measured).toEqual({
      instrument: "wasm-linear-memory",
      peak_linear_memory_bytes: 1_500_000,
      x_output: 1.5,
    });
    expect(measured).not.toHaveProperty("peak_rss_bytes");
  });

  it("mem cells are shape-distinguishable from refusals", () => {
    // The schema-2 contract: a measured cell and a refusal share no keys, so
    // no reader can mistake a vetoed run for a small number.
    const refused = refusalCell("beyond-mem-cap", "too big");
    expect(refused).toEqual({ refusal: "beyond-mem-cap", detail: "too big" });
  });

  it("x_output rounds to two decimals", () => {
    expect(memCell(1_234_567, 1_000_000).x_output).toBe(1.23);
  });

  it("dropped rungs are recorded in the artifact, even as an empty list", () => {
    // The artifact states "nothing was dropped" positively, not by omission.
    const out = buildOutput(10, [], []);
    expect(out.schema).toBe(2);
    expect(out.surface).toBe("wasm");
    expect(out.skipped).toEqual([]);
    expect(JSON.stringify(out)).toContain('"skipped":[]');
  });
});
