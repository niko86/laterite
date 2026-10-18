// `maxFindingsPerRule` (#1089): a per-rule cap on what a report KEEPS, never on
// what it CONCLUDES. Every assertion compares a capped run with the uncapped run
// of the same file, so the uncapped engine is the oracle.
import { mkdtempSync, readFileSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { describe, expect, it, vi } from "vitest";

import { main } from "../ts/cli";
import { read, validate } from "../ts/index";

const RULE_2A = "AGS Format Rule 2a";
const pkgDir = resolve(fileURLToPath(new URL("..", import.meta.url)));
const CLEAN = resolve(
  pkgDir,
  "..",
  "laterite-ags4-validator",
  "tests",
  "fixtures",
  "clean_minimal.ags",
);

// The clean fixture with LF line endings: one Rule 2a finding per line.
function lfFile(): string {
  const dir = mkdtempSync(join(tmpdir(), "lat-cap-"));
  const p = join(dir, "lf.ags");
  writeFileSync(p, readFileSync(CLEAN, "utf8").replaceAll("\r\n", "\n"));
  return p;
}

const tiers = (r: ReturnType<typeof validate>) => [
  r.errors,
  r.warnings,
  r.fyi,
  r.count,
  r.isValid,
  r.exitCode,
];

describe("validate({ maxFindingsPerRule })", () => {
  const file = lfFile();
  const full = validate(file);
  const total = full.byRule()[RULE_2A]?.length ?? 0;

  it("has a rule worth cutting", () => {
    expect(total).toBeGreaterThan(2);
    expect(full.truncated).toEqual({});
  });

  it("keeps each rule's first N and names exactly the rules it cut", () => {
    const capped = validate(file, { maxFindingsPerRule: 2 });
    expect(capped.byRule()[RULE_2A]).toEqual(
      full.byRule()[RULE_2A]?.slice(0, 2),
    );
    const expected = Object.fromEntries(
      Object.entries(full.byRule())
        .filter(([, v]) => v.length > 2)
        .map(([k, v]) => [k, { shown: 2, total: v.length }]),
    );
    expect(capped.truncated).toEqual(expected);
    expect(tiers(capped)).toEqual(tiers(full));
  });

  it("0 keeps no findings and still counts them all", () => {
    const zero = validate(file, { maxFindingsPerRule: 0 });
    expect(zero.findings).toEqual([]);
    expect(zero.truncated[RULE_2A]).toEqual({ shown: 0, total });
    expect(tiers(zero)).toEqual(tiers(full));
    expect(JSON.parse(zero.toJson()).truncated[RULE_2A]).toEqual({
      shown: 0,
      total,
    });
    const last = zero.toNdjson().trimEnd().split("\n").at(-1) ?? "";
    expect(JSON.parse(last)).toEqual({ truncated: zero.truncated });
  });

  it("reaches the handle's validate too", () => {
    const r = read(file).validate({ maxFindingsPerRule: 1 }).report;
    expect(r?.truncated[RULE_2A]).toEqual({ shown: 1, total });
  });

  it("refuses a cap that is not a count", () => {
    for (const bad of [-1, 1.5, Number.NaN])
      expect(() => validate(file, { maxFindingsPerRule: bad })).toThrow(
        RangeError,
      );
  });
});

describe("lat validate --max-findings-per-rule", () => {
  function runCli(argv: string[]): { code: number; stdout: string } {
    let stdout = "";
    const out = vi
      .spyOn(process.stdout, "write")
      .mockImplementation((chunk: unknown) => {
        stdout += String(chunk);
        return true;
      });
    const err = vi
      .spyOn(process.stderr, "write")
      .mockImplementation(() => true);
    const exit = vi.spyOn(process, "exit").mockImplementation(((
      code?: number,
    ) => {
      throw Object.assign(new Error("__cli_exit__"), { exitCode: code ?? 0 });
    }) as typeof process.exit);
    let code: number;
    try {
      code = main(argv);
    } catch (e) {
      const ex = e as { message?: string; exitCode?: number };
      if (ex.message !== "__cli_exit__") throw e;
      code = ex.exitCode ?? 0;
    } finally {
      out.mockRestore();
      err.mockRestore();
      exit.mockRestore();
    }
    return { code, stdout };
  }

  it("passes the cap through and keeps the exit code", () => {
    const file = lfFile();
    const plain = runCli(["validate", "--json", file]);
    const capped = runCli([
      "validate",
      "--json",
      "--max-findings-per-rule",
      "1",
      file,
    ]);
    expect(capped.code).toBe(plain.code);
    const v = JSON.parse(capped.stdout) as {
      findings: Record<string, unknown[]>;
      truncated: Record<string, { shown: number }>;
    };
    expect(v.findings[RULE_2A]).toHaveLength(1);
    expect(v.truncated[RULE_2A]?.shown).toBe(1);
  });

  it("states each cut rule in the plain report", () => {
    const r = runCli(["validate", "--max-findings-per-rule", "0", lfFile()]);
    expect(r.stdout).toMatch(/truncated: AGS Format Rule 2a — 0 of \d+ shown/);
  });

  it("refuses a value that is not a count", () => {
    expect(
      runCli(["validate", "--max-findings-per-rule", "-1", lfFile()]).code,
    ).toBe(5);
  });
});
