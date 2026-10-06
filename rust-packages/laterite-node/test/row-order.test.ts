// #1008 — `rowOrder` on `buildAgs4`, `merge` and `lat merge --row-order`. The
// same engine as Python's `test_row_order.py` and the binary's `--row-order`,
// so the same values.
import { mkdtempSync, readFileSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { afterEach, describe, expect, it, vi } from "vitest";
import { main } from "../ts/cli";
import { buildAgs4, merge } from "../ts/index";

/** One delivery: LOCA and SAMP, rows in the order given. */
const delivery = (samp: Array<[string, string, string, string]>) =>
  [
    '"GROUP","PROJ"',
    '"HEADING","PROJ_ID"',
    '"UNIT",""',
    '"TYPE","ID"',
    '"DATA","P1"',
    "",
    '"GROUP","LOCA"',
    '"HEADING","LOCA_ID"',
    '"UNIT",""',
    '"TYPE","ID"',
    ...[...new Set(samp.map((r) => r[0]))].map((id) => `"DATA","${id}"`),
    "",
    '"GROUP","SAMP"',
    '"HEADING","LOCA_ID","SAMP_TOP","SAMP_REF","SAMP_TYPE","SAMP_ID"',
    '"UNIT","","m","","",""',
    '"TYPE","ID","2DP","X","PA","ID"',
    ...samp.map((r) => `"DATA","${r.join('","')}",""`),
    "",
  ].join("\r\n");

// The issue's deliveries: arrival order interleaves them.
const a = Buffer.from(
  delivery([
    ["BH10", "2.00", "2", "D"],
    ["BH2", "1.00", "1", "B"],
  ]),
);
const b = Buffer.from(
  delivery([
    ["BH2", "0.50", "1", "ES"],
    ["BH1", "3.00", "3", "B"],
  ]),
);

/** The DATA rows of one group, as written. */
const rows = (text: string, code: string) => {
  const lines = text.split("\r\n");
  const out: string[] = [];
  for (const line of lines.slice(lines.indexOf(`"GROUP","${code}"`) + 1)) {
    if (line === "") break;
    if (line.startsWith('"DATA"')) out.push(line);
  }
  return out;
};

afterEach(() => {
  vi.restoreAllMocks();
});

describe("rowOrder", () => {
  it("merge key sorts each group by its KEY headings", () => {
    const sorted = merge([a, b], { rowOrder: "key" });
    expect(rows(sorted.text, "LOCA")).toEqual([
      '"DATA","BH1"',
      '"DATA","BH2"',
      '"DATA","BH10"',
    ]);
    expect(rows(sorted.text, "SAMP")).toEqual([
      '"DATA","BH1","3.00","3","B",""',
      '"DATA","BH2","0.50","1","ES",""',
      '"DATA","BH2","1.00","1","B",""',
      '"DATA","BH10","2.00","2","D",""',
    ]);
  });

  it("input is the default and changes nothing merge reconciled", () => {
    const plain = merge([a, b]);
    expect(merge([a, b], { rowOrder: "input" }).text).toBe(plain.text);
    expect(rows(plain.text, "LOCA")[0]).toBe('"DATA","BH10"');
    const sorted = merge([a, b], { rowOrder: "key" });
    expect(sorted.warnings).toEqual(plain.warnings);
    expect(sorted.revisions).toEqual(plain.revisions);
    for (const code of ["LOCA", "SAMP"]) {
      expect([...rows(sorted.text, code)].sort()).toEqual(
        [...rows(plain.text, code)].sort(),
      );
    }
  });

  it("buildAgs4 key sorts naturally and the default keeps input order", () => {
    const groups = new Map<string, Array<Record<string, unknown>>>([
      ["PROJ", [{ PROJ_ID: "P1" }]],
      ["LOCA", [{ LOCA_ID: "BH10" }, { LOCA_ID: "BH2" }, { LOCA_ID: "BH1" }]],
    ]);
    const plain = buildAgs4(groups, { mode: "report" });
    expect(rows(plain.text, "LOCA")[0]).toBe('"DATA","BH10"');
    expect(buildAgs4(groups, { mode: "report", rowOrder: "input" }).text).toBe(
      plain.text,
    );
    const sorted = buildAgs4(groups, { mode: "report", rowOrder: "key" });
    expect(rows(sorted.text, "LOCA")).toEqual([
      '"DATA","BH1"',
      '"DATA","BH2"',
      '"DATA","BH10"',
    ]);
  });

  it("refuses an unknown value", () => {
    expect(() =>
      merge([a, b], { rowOrder: "sorted" as unknown as "key" }),
    ).toThrow(/unknown row_order "sorted"/);
  });

  it("lat merge --row-order key", () => {
    const dir = mkdtempSync(join(tmpdir(), "lat-row-order-"));
    const pa = join(dir, "a.ags");
    const pb = join(dir, "b.ags");
    const out = join(dir, "out.ags");
    writeFileSync(pa, a);
    writeFileSync(pb, b);
    vi.spyOn(process.stdout, "write").mockImplementation(() => true);
    let stderr = "";
    vi.spyOn(process.stderr, "write").mockImplementation((chunk: unknown) => {
      stderr += String(chunk);
      return true;
    });
    vi.spyOn(process, "exit").mockImplementation(((code?: number) => {
      throw Object.assign(new Error("__cli_exit__"), { exitCode: code ?? 0 });
    }) as typeof process.exit);
    const run = (argv: string[]): number => {
      try {
        return main(argv);
      } catch (e) {
        const ex = e as { message?: string; exitCode?: number };
        if (ex.message === "__cli_exit__") return ex.exitCode ?? 0;
        throw e;
      }
    };
    expect(run(["merge", pa, pb, "--out", out, "--row-order", "key"])).toBe(0);
    expect(rows(readFileSync(out, "utf8"), "LOCA")[0]).toBe('"DATA","BH1"');
    expect(run(["merge", pa, pb, "--out", out, "--row-order", "sorted"])).toBe(
      5,
    );
    expect(stderr).toContain("--row-order");
  });
});
