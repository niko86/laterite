// The matrix harness's pure seams (#823), mirroring the rust bin's unit tests
// (`laterite-ags4-perf`): the mistakes worth pinning here are the ones that
// would move every cell at once — a units slip, a cap disagreement between
// surfaces, a refusal a reader could mistake for a small number.
import { describe, expect, it } from "vitest";

import {
  buildOutput,
  maxRssToBytes,
  median,
  memCell,
  memRungAllowed,
  parseMeminfoSwap,
  parseSwapUsedDarwin,
  refusalCell,
  throughputMbS,
} from "../bench/perf-matrix.mjs";

describe("perf-matrix harness seams", () => {
  it("throughput is decimal MB/s and never divides by zero", () => {
    // 5 MB in 10 ms → 500 MB/s (decimal MB, matching forge's parse_size).
    expect(throughputMbS(5_000_000, 10.0)).toBeCloseTo(500.0, 9);
    expect(throughputMbS(1_000, 0)).toBe(0);
  });

  it("median picks the middle sample, input order irrelevant", () => {
    expect(median([3, 1, 2])).toBe(2);
    expect(median([7])).toBe(7);
    // Even count: the upper-middle sample, as in the rust bin (len/2).
    expect(median([4, 1, 3, 2])).toBe(3);
  });

  it("maxRSS converts from kibibytes on every platform", () => {
    // libuv normalises `ru_maxrss` to KB everywhere (darwin's raw bytes are
    // divided down inside uv_getrusage), so unlike the rust bin there is no
    // OS branch — one slip here would move every cell 1024×.
    expect(maxRssToBytes(1_024)).toBe(1_048_576);
    expect(maxRssToBytes(0)).toBe(0);
  });

  it("mem cap admits the 265MB rung and refuses 524MB", () => {
    // The pinned rung sizes (tools/readme-bench-fixtures.json): epic #820
    // decision 7, in agreement with the rust and python harnesses.
    expect(memRungAllowed(276_462_834)).toBe(true);
    expect(memRungAllowed(551_560_078)).toBe(false);
  });

  it("darwin swap parse reads the used field", () => {
    const text =
      "total = 2048.00M  used = 512.50M  free = 1535.50M  (encrypted)";
    expect(parseSwapUsedDarwin(text)).toBe(537_395_200);
    expect(parseSwapUsedDarwin("garbage")).toBeNull();
  });

  it("meminfo swap is total minus free", () => {
    const text =
      "MemTotal: 100 kB\nSwapTotal:     2048 kB\nSwapFree:      1024 kB\n";
    expect(parseMeminfoSwap(text)).toBe(1024 * 1024);
    expect(parseMeminfoSwap("MemTotal: 1 kB")).toBeNull();
  });

  it("mem cells are shape-distinguishable", () => {
    // The schema-2 contract: a measured cell and a refusal share no keys, so
    // no reader can mistake a vetoed run for a small number.
    const measured = memCell(1_500_000, 1_000_000);
    expect(measured).toEqual({ peak_rss_bytes: 1_500_000, x_output: 1.5 });
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
    expect(out.surface).toBe("node");
    expect(out.skipped).toEqual([]);
    expect(JSON.stringify(out)).toContain('"skipped":[]');
  });
});
