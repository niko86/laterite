// #1010 — `onCodeCase` and `recode` on `merge` and `lat merge`. The same engine
// as Python's `test_merge_code_case.py` and the binary's `--on-code-case`, so
// the same outcomes.
import { mkdtempSync, readFileSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { afterEach, describe, expect, it, vi } from "vitest";
import { main } from "../ts/cli";
import { BadDictError, MergeConflictError } from "../ts/errors";
import { merge } from "../ts/index";

/** One delivery: ABBR declares `cond` under TRIG_COND, and TRIG uses it on a
 *  specimen of a sample whose type is `sampType`. */
const delivery = (cond: string, sampType = "U") =>
  Buffer.from(
    [
      '"GROUP","PROJ"',
      '"HEADING","PROJ_ID"',
      '"UNIT",""',
      '"TYPE","ID"',
      '"DATA","P1"',
      "",
      '"GROUP","ABBR"',
      '"HEADING","ABBR_HDNG","ABBR_CODE","ABBR_DESC"',
      '"UNIT","","",""',
      '"TYPE","X","X","X"',
      `"DATA","TRIG_COND","${cond}","d"`,
      `"DATA","SAMP_TYPE","${sampType}","s"`,
      "",
      '"GROUP","SAMP"',
      '"HEADING","LOCA_ID","SAMP_TOP","SAMP_REF","SAMP_TYPE","SAMP_ID"',
      '"UNIT","","m","","",""',
      '"TYPE","ID","2DP","X","PA","ID"',
      `"DATA","BH1","1.00","1","${sampType}","S1"`,
      "",
      '"GROUP","TRIG"',
      '"HEADING","LOCA_ID","SAMP_TOP","SAMP_REF","SAMP_TYPE","SAMP_ID","SPEC_REF","SPEC_DPTH","TRIG_COND"',
      '"UNIT","","m","","","","","m",""',
      '"TYPE","ID","2DP","X","PA","ID","X","2DP","PA"',
      `"DATA","BH1","1.00","1","${sampType}","S1","1","1.00","${cond}"`,
      "",
    ].join("\r\n"),
  );

// The non-standard spelling comes last, so it wins unless rewritten.
const a = delivery("UNDISTURBED");
const b = delivery("Undisturbed");

afterEach(() => {
  vi.restoreAllMocks();
});

describe("onCodeCase / recode", () => {
  it("warns by default with the set, and changes nothing else", () => {
    const res = merge([a, b]);
    const w = res.warnings.find((w) => w.kind === "abbr_code_case");
    expect(w?.heading).toBe("TRIG_COND");
    expect(w?.case).toEqual({
      heading: "TRIG_COND",
      spellings: [
        { code: "UNDISTURBED", inputs: [0] },
        { code: "Undisturbed", inputs: [1] },
      ],
      suggested: "UNDISTURBED",
      resolved: null,
    });
    expect(res.text).toContain('"Undisturbed"');
    expect(merge([a, b], { onCodeCase: "keep" }).text).toBe(res.text);
  });

  it("standard and an explicit recode settle on the standard spelling", () => {
    const std = merge([a, b], { onCodeCase: "standard" });
    expect(std.text).not.toContain('"Undisturbed"');
    expect(std.revisions.filter((r) => r.group === "TRIG")).toEqual([]);
    const named = merge([a, b], {
      recode: { TRIG_COND: { Undisturbed: "UNDISTURBED" } },
    });
    expect(named.text).toBe(std.text);
  });

  it("refuses a rewrite that would merge two rows", () => {
    expect(() =>
      merge([delivery("UNDISTURBED", "U"), delivery("UNDISTURBED", "u")], {
        onCodeCase: "standard",
      }),
    ).toThrow(MergeConflictError);
  });

  it("refuses an unknown mode and a recode it cannot honour, by name", () => {
    expect(() =>
      merge([a, b], { onCodeCase: "first" as unknown as "keep" }),
    ).toThrow(/first[\s\S]*keep, standard/);
    expect(() => merge([a, b], { recode: { SAMP_REF: { "1": "2" } } })).toThrow(
      BadDictError,
    );
    expect(() =>
      merge([a, b], { recode: { TRIG_COND: { Disturbed: "D" } } }),
    ).toThrow(/Disturbed/);
  });

  it("lat merge --on-code-case / --recode", () => {
    const dir = mkdtempSync(join(tmpdir(), "lat-code-case-"));
    const pa = join(dir, "a.ags");
    const pb = join(dir, "b.ags");
    const out = join(dir, "out.ags");
    const recode = join(dir, "recode.json");
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
    const run = (extra: string[]): number => {
      try {
        return main(["merge", pa, pb, "--out", out, ...extra]);
      } catch (e) {
        const ex = e as { message?: string; exitCode?: number };
        if (ex.message === "__cli_exit__") return ex.exitCode ?? 0;
        throw e;
      }
    };
    expect(run(["--on-code-case", "standard"])).toBe(0);
    const standard = readFileSync(out, "utf8");
    expect(standard).not.toContain('"Undisturbed"');
    writeFileSync(
      recode,
      JSON.stringify({ TRIG_COND: { Undisturbed: "UNDISTURBED" } }),
    );
    expect(run(["--recode", recode])).toBe(0);
    expect(readFileSync(out, "utf8")).toBe(standard);

    expect(run(["--on-code-case", "first"])).toBe(5);
    expect(stderr).toContain("--on-code-case");
    writeFileSync(recode, JSON.stringify({ SAMP_REF: { "1": "2" } }));
    expect(run(["--recode", recode])).toBe(5);
    expect(stderr).toContain("SAMP_REF");
    writeFileSync(recode, "[1, 2]");
    expect(run(["--recode", recode])).toBe(5);
  });
});
