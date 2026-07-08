// The Node `lat` CLI — end-to-end through the built `bin.mjs`, the exact
// executable `npx laterite` / a global `lat` runs (#430). Scriptable outputs
// (`validate --json`, `read --csv`/`--json`, `rules --json`) are byte-checked
// against the Rust binary in the repo's cross-surface gates; here we assert each
// verb works and the exit codes follow the binary's scheme.
import { execFileSync } from "node:child_process";
import { mkdtempSync, readFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { describe, expect, it } from "vitest";

const pkgDir = resolve(fileURLToPath(new URL("..", import.meta.url)));
const BIN = join(pkgDir, "bin.mjs");
const FIX = resolve(pkgDir, "..", "laterite-ags4-validator", "tests", "fixtures");
const CLEAN = join(FIX, "clean_minimal.ags");

function run(args: string[]): { stdout: string; code: number } {
  try {
    const stdout = execFileSync("node", [BIN, ...args], { encoding: "utf8" });
    return { stdout, code: 0 };
  } catch (e) {
    const err = e as { stdout?: string; status?: number };
    return { stdout: err.stdout ?? "", code: err.status ?? 1 };
  }
}

describe("lat node CLI", () => {
  it("validate: a clean file exits 0", () => {
    expect(run(["validate", CLEAN]).code).toBe(0);
  });

  it("bare file is shorthand for validate", () => {
    expect(run([CLEAN]).code).toBe(0);
  });

  it("read: lists the group codes", () => {
    const { stdout, code } = run(["read", CLEAN]);
    expect(code).toBe(0);
    expect(stdout).toContain("PROJ");
  });

  it("read --csv: header row first", () => {
    const { stdout } = run(["read", CLEAN, "PROJ", "--csv"]);
    expect(stdout.split("\n")[0]).toContain("PROJ_ID");
  });

  it("rules --json: the rules_meta catalogue", () => {
    const { stdout, code } = run(["rules", "--json"]);
    expect(code).toBe(0);
    expect(JSON.parse(stdout).rules.length).toBeGreaterThan(20);
  });

  it("pack / unpack: lossless round-trip", () => {
    const dir = mkdtempSync(join(tmpdir(), "lat-cli-"));
    expect(run(["pack", CLEAN, join(dir, "p.zst")]).code).toBe(0);
    expect(run(["unpack", join(dir, "p.zst"), join(dir, "back.ags")]).code).toBe(0);
    expect(readFileSync(join(dir, "back.ags"))).toEqual(readFileSync(CLEAN));
  });

  it("excel: export → import round-trip", () => {
    const dir = mkdtempSync(join(tmpdir(), "lat-cli-"));
    expect(run(["excel", CLEAN, join(dir, "o.xlsx")]).code).toBe(0);
    expect(run(["excel", join(dir, "o.xlsx"), join(dir, "back.ags")]).code).toBe(0);
  });

  it("exit codes follow the binary: missing → 3, ambiguous excel → 5, bad group → 4", () => {
    expect(run(["read", "/no/such/file.ags"]).code).toBe(3);
    expect(run(["excel", CLEAN, join(tmpdir(), "x.dat")]).code).toBe(5);
    expect(run(["read", CLEAN, "NOPE"]).code).toBe(4);
  });
});
