// #1024 — `onCodeCase` and `recode` on `fix` and `lat fix`. The same engine
// as Python's `test_fix_code_case.py` and the binary's `fix --on-code-case`,
// so the same outcomes.
import { mkdtempSync, readFileSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { afterEach, describe, expect, it, vi } from "vitest";
import { main } from "../ts/cli";
import { BadDictError } from "../ts/errors";
import { fix } from "../ts/index";

/** One file with both spellings of TRIG_COND, and two SAMP rows whose KEYs
 *  differ only in the case of SAMP_TYPE. */
const file = Buffer.from(
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
    '"DATA","SAMP_TYPE","U","d"',
    '"DATA","SAMP_TYPE","u","d"',
    '"DATA","TRIG_COND","UNDISTURBED","d"',
    '"DATA","TRIG_COND","Undisturbed","d"',
    "",
    '"GROUP","SAMP"',
    '"HEADING","LOCA_ID","SAMP_TOP","SAMP_REF","SAMP_TYPE","SAMP_ID"',
    '"UNIT","","m","","",""',
    '"TYPE","ID","2DP","X","PA","ID"',
    '"DATA","BH1","1.00","1","U","S1"',
    '"DATA","BH1","1.00","1","u","S1"',
    "",
    '"GROUP","TRIG"',
    '"HEADING","LOCA_ID","SAMP_TOP","SAMP_REF","SAMP_TYPE","SAMP_ID","SPEC_REF","SPEC_DPTH","TRIG_COND"',
    '"UNIT","","m","","","","","m",""',
    '"TYPE","ID","2DP","X","PA","ID","X","2DP","PA"',
    '"DATA","BH1","1.00","1","U","S1","1","1.00","UNDISTURBED"',
    '"DATA","BH1","1.00","1","U","S1","2","1.00","Undisturbed"',
    "",
  ].join("\r\n"),
);

afterEach(() => {
  vi.restoreAllMocks();
});

describe("fix onCodeCase / recode", () => {
  it("rewrites nothing by default", () => {
    const res = fix(file);
    expect(res.applied.map((a) => a.kind)).not.toContain("recode_abbreviation");
    expect(res.skipped).toEqual([]);
    expect(res.text).toContain('"Undisturbed"');
    expect(fix(file, { onCodeCase: "keep", recode: {} }).text).toBe(res.text);
  });

  it("standard rewrites one set and skips the one that would collide", () => {
    const res = fix(file, { onCodeCase: "standard" });
    const recode = res.applied.filter((a) => a.kind === "recode_abbreviation");
    expect(recode.map((a) => a.risk)).toEqual(["safe"]);
    expect(res.text).not.toContain('"Undisturbed"');
    expect(res.text).toContain('"u","S1"');
    expect(res.skipped).toEqual([
      {
        heading: "SAMP_TYPE",
        codes: ["u"],
        target: "U",
        group: "SAMP",
        key: ["BH1", "1.00", "1", "U", "S1"],
      },
    ]);
    const named = fix(file, {
      recode: { TRIG_COND: { Undisturbed: "UNDISTURBED" } },
    });
    expect(named.text).toBe(res.text);
  });

  it("refuses a contradiction, an unknown mode and a recode it cannot honour", () => {
    expect(() => fix(file, { onCodeCase: "standard", only: ["4"] })).toThrow(
      TypeError,
    );
    expect(() =>
      fix(file, { recode: { TRIG_COND: { u: "U" } }, exclude: ["16"] }),
    ).toThrow(/rule "16"/);
    expect(
      fix(file, { onCodeCase: "standard", only: ["16"] }).applied.length,
    ).toBe(1);
    expect(() =>
      fix(file, { onCodeCase: "first" as unknown as "keep" }),
    ).toThrow(/first[\s\S]*keep, standard/);
    expect(() => fix(file, { recode: { SAMP_REF: { "1": "2" } } })).toThrow(
      BadDictError,
    );
  });

  it("lat fix --on-code-case / --recode, and --json skipped", () => {
    const dir = mkdtempSync(join(tmpdir(), "lat-fix-code-case-"));
    const src = join(dir, "a.ags");
    const out = join(dir, "out.ags");
    const recode = join(dir, "recode.json");
    writeFileSync(src, file);
    let stdout = "";
    vi.spyOn(process.stdout, "write").mockImplementation((chunk: unknown) => {
      stdout += String(chunk);
      return true;
    });
    let stderr = "";
    vi.spyOn(process.stderr, "write").mockImplementation((chunk: unknown) => {
      stderr += String(chunk);
      return true;
    });
    vi.spyOn(console, "log").mockImplementation((...a: unknown[]) => {
      stdout += a.join(" ") + "\n";
    });
    vi.spyOn(process, "exit").mockImplementation(((code?: number) => {
      throw Object.assign(new Error("__cli_exit__"), { exitCode: code ?? 0 });
    }) as typeof process.exit);
    const run = (extra: string[]): number => {
      stdout = "";
      try {
        return main(["fix", src, "--fix-out", out, ...extra]);
      } catch (e) {
        const ex = e as { message?: string; exitCode?: number };
        if (ex.message === "__cli_exit__") return ex.exitCode ?? 0;
        throw e;
      }
    };

    run(["--json"]);
    expect(JSON.parse(stdout)).not.toHaveProperty("skipped");
    run(["--json", "--on-code-case", "standard"]);
    expect(JSON.parse(stdout).skipped[0].codes).toEqual(["u"]);
    const standard = readFileSync(out, "utf8");
    expect(standard).not.toContain('"Undisturbed"');
    run(["--on-code-case", "standard"]);
    expect(stdout).toContain('left "u" under SAMP_TYPE as written');
    writeFileSync(
      recode,
      JSON.stringify({ TRIG_COND: { Undisturbed: "UNDISTURBED" } }),
    );
    run(["--recode", recode]);
    expect(readFileSync(out, "utf8")).toBe(standard);

    expect(run(["--on-code-case", "first"])).toBe(5);
    expect(stderr).toContain("--on-code-case");
    writeFileSync(recode, JSON.stringify({ SAMP_REF: { "1": "2" } }));
    expect(run(["--recode", recode])).toBe(5);
    expect(stderr).toContain("SAMP_REF");
  });
});
