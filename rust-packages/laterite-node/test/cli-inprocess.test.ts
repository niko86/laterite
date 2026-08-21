// The Node `lat` CLI — driven IN-PROCESS, calling `main(argv)` and `census()`
// from `ts/cli.ts` directly. This is the sibling of `test/cli.test.ts`, which
// spawns `bin.mjs` as a subprocess: that end-to-end check is the real proof the
// executable works, but v8's in-process coverage instrumentation cannot see a
// child process, so `cli.ts` reads as ~7% covered there. Here every verb runs in
// the test's own process so the coverage counter sees each branch it takes.
//
// The CLI's process side-effects are mocked and restored per test:
//   * `process.exit` (reached via `fail()`) throws a tagged error carrying the
//     exit code, so an error path becomes a catchable value instead of tearing
//     down the test runner.
//   * `process.stdout`/`process.stderr` `write` are captured into strings so the
//     verb's output can be asserted, exactly as `cli.test.ts` reads the child's
//     stdout/stderr.
import {
  copyFileSync,
  existsSync,
  mkdtempSync,
  readFileSync,
  writeFileSync,
} from "node:fs";
import { tmpdir } from "node:os";
import { join, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { afterEach, describe, expect, it, vi } from "vitest";

import { census, main } from "../ts/cli";
import { validate } from "../ts/index";
import { BuildResult } from "../ts/build-result";
import { parseValue } from "../ts/ags-types";
import { ancestorChain, inheritedKeyNames } from "../ts/registry";

const pkgDir = resolve(fileURLToPath(new URL("..", import.meta.url)));
const FIX = resolve(
  pkgDir,
  "..",
  "laterite-ags4-validator",
  "tests",
  "fixtures",
);
const CLEAN = join(FIX, "clean_minimal.ags");
const DIRTY = join(FIX, "rule2_no_data_rows.ags"); // a fixture with real findings
const MFIX = resolve(pkgDir, "..", "laterite-ags4-merge", "tests", "fixtures");
const MERGE_A = join(MFIX, "delivery_a.ags");
const MERGE_B = join(MFIX, "delivery_b.ags");

/** A fresh temp dir for outputs — each verb that writes (`--out`, `fix`, `certify`,
 *  `merge`, `transport`, `excel`) needs somewhere off the fixtures tree. */
const tmp = (): string => mkdtempSync(join(tmpdir(), "lat-inproc-"));

interface CliResult {
  code: number;
  stdout: string;
  stderr: string;
}

/** Run `main(argv)` with the process side-effects mocked. Returns the exit code
 *  (the value `main` returns on a success path, or the code `fail()` handed
 *  `process.exit` on an error path) plus the captured stdout/stderr. */
function runCli(argv: string[]): CliResult {
  let stdout = "";
  let stderr = "";
  const outSpy = vi
    .spyOn(process.stdout, "write")
    .mockImplementation((chunk: unknown) => {
      stdout += String(chunk);
      return true;
    });
  const errSpy = vi
    .spyOn(process.stderr, "write")
    .mockImplementation((chunk: unknown) => {
      stderr += String(chunk);
      return true;
    });
  const exitSpy = vi.spyOn(process, "exit").mockImplementation(((
    code?: number,
  ) => {
    throw Object.assign(new Error("__cli_exit__"), { exitCode: code ?? 0 });
  }) as typeof process.exit);

  let code: number;
  try {
    code = main(argv);
  } catch (e) {
    // A thrown value really can be null/undefined, so keep the type nullable.
    const ex = e as { message?: string; exitCode?: number } | null | undefined;
    if (ex && ex.message === "__cli_exit__") code = ex.exitCode ?? 0;
    else throw e; // a real error, not a mocked exit — surface it
  } finally {
    outSpy.mockRestore();
    errSpy.mockRestore();
    exitSpy.mockRestore();
  }
  return { code, stdout, stderr };
}

/** Mint a certificate for `src` via the CLI and return the `.ags.idx` path it
 *  printed — reused by the `validate --index` cases rather than re-deriving the
 *  naming convention (the same trick `cli.test.ts` uses). */
function mintCert(src: string): string {
  const { stdout, code } = runCli(["certify", src]);
  expect(code).toBe(0);
  return stdout.trim().split(" ").pop() as string;
}

afterEach(() => {
  vi.restoreAllMocks();
});

// ---- census ----------------------------------------------------------------
describe("cli (in-process): census", () => {
  it("`census` verb prints the surface's tables as JSON", () => {
    const { code, stdout } = runCli(["census"]);
    expect(code).toBe(0);
    const c = JSON.parse(stdout);
    expect(c.surface).toBe("cli-npx");
    expect(c.authority).toBe(false);
    expect(c.documented_verbs).toContain("validate");
    expect(c.documented_verbs).toContain("merge");
  });

  it("census() reflects SPECS: per-verb args, positionals, editions, encodings", () => {
    const c = census() as {
      census_version: number;
      verbs: { verb: string; args: { name: string; takes_value: boolean }[] }[];
      global_args: { name: string }[];
      editions: string[];
      fallback_edition: string;
      encodings: Record<string, string | null>;
    };
    expect(c.census_version).toBeGreaterThan(0);
    // A valued flag reports takes_value:true; a positional reports as `<name>`.
    const validateVerb = c.verbs.find((v) => v.verb === "validate")!;
    const dictVersionArg = validateVerb.args.find(
      (a) => a.name === "--dict-version",
    )!;
    expect(dictVersionArg.takes_value).toBe(true);
    expect(validateVerb.args.some((a) => a.name === "<file>")).toBe(true);
    expect(c.global_args.some((a) => a.name === "--quiet")).toBe(true);
    expect(c.editions.length).toBeGreaterThan(0);
    expect(c.fallback_edition).toMatch(/^4\./);
    // `cp1252x` is a bogus label the wrapper must resolve to null (the wrapper-bug
    // regression laterite-dev#555 pins this).
    expect(c.encodings["cp1252x"]).toBeNull();
    expect(c.encodings["utf-8"]).not.toBeNull();
  });
});

// ---- read ------------------------------------------------------------------
describe("cli (in-process): read", () => {
  it("bare read lists the group codes", () => {
    const { code, stdout } = runCli(["read", CLEAN]);
    expect(code).toBe(0);
    expect(stdout).toContain("PROJ");
  });

  it("read refuses a duplicate heading, and --recover-duplicate-headings keeps both", () => {
    // Rows are keyed by heading name, so before the guard the second LOCA_ID
    // overwrote the first and this file read back as ["SECOND","1.00","SECOND"]
    // -- FIRST gone, SECOND duplicated into its column, silently. `read` runs no
    // rule engine, so Rule 7 never fired here.
    const dup = join(tmp(), "dup.ags");
    writeFileSync(
      dup,
      [
        '"GROUP","LOCA"',
        '"HEADING","LOCA_ID","LOCA_GL","LOCA_ID"',
        '"UNIT","","m",""',
        '"TYPE","ID","2DP","ID"',
        '"DATA","FIRST","1.00","SECOND"',
      ].join("\r\n") + "\r\n",
    );

    const refused = runCli(["read", dup, "LOCA", "--json"]);
    expect(refused.code).not.toBe(0);
    expect(refused.stderr).toContain("duplicate heading");

    const { code, stdout } = runCli([
      "read",
      dup,
      "LOCA",
      "--json",
      "--recover-duplicate-headings",
    ]);
    expect(code).toBe(0);
    const body = JSON.stringify(JSON.parse(stdout));
    expect(body).toContain("LOCA_ID__2");
    // Both cells survive, in their own columns.
    expect(body).toContain("FIRST");
    expect(body).toContain("SECOND");
  });

  it("read --json lists the group order as a JSON array", () => {
    const { code, stdout } = runCli(["read", CLEAN, "--json"]);
    expect(code).toBe(0);
    expect(JSON.parse(stdout)).toContain("PROJ");
  });

  it("read <group> renders the human table", () => {
    const { code, stdout } = runCli(["read", CLEAN, "PROJ"]);
    expect(code).toBe(0);
    expect(stdout).toContain("PROJ_ID");
    expect(stdout).toContain("---"); // the table's separator row
  });

  it("read <group> --csv renders CSV (header first)", () => {
    const { code, stdout } = runCli(["read", CLEAN, "PROJ", "--csv"]);
    expect(code).toBe(0);
    expect(stdout.split("\n")[0]).toContain("PROJ_ID");
  });

  it("read <group> --json renders the engine JSON body", () => {
    const { code, stdout } = runCli(["read", CLEAN, "PROJ", "--json"]);
    expect(code).toBe(0);
    expect(() => JSON.parse(stdout)).not.toThrow();
  });

  it("read --out <file> writes the body to disk and notes it", () => {
    const out = join(tmp(), "groups.txt");
    const { code, stderr } = runCli(["read", CLEAN, "--out", out]);
    expect(code).toBe(0);
    expect(existsSync(out)).toBe(true);
    expect(readFileSync(out, "utf8")).toContain("PROJ");
    expect(stderr).toContain(`written to ${out}`);
  });

  it("read of a file with no groups says so and exits 0", () => {
    const empty = join(tmp(), "empty.ags");
    writeFileSync(empty, "");
    const { code, stderr } = runCli(["read", empty]);
    expect(code).toBe(0);
    expect(stderr).toContain("no groups in the file");
  });

  it("read with no file → exit 5", () => {
    expect(runCli(["read"]).code).toBe(5);
  });

  it("read of a missing file → exit 3", () => {
    expect(runCli(["read", "/no/such/file.ags"]).code).toBe(3);
  });

  it("read of an unknown group → exit 4, names the present groups", () => {
    const { code, stderr } = runCli(["read", CLEAN, "NOPE"]);
    expect(code).toBe(4);
    expect(stderr).toContain("present:");
  });

  it("read of non-UTF-8 bytes surfaces the engine error (exit 6)", () => {
    const g = join(tmp(), "garbage.bin");
    writeFileSync(g, Buffer.from([0x00, 0x01, 0xff, 0xfe, 0x02]));
    expect(runCli(["read", g]).code).toBe(6);
  });

  it("read <group> on a file with no groups → exit 4, present: none", () => {
    const empty = join(tmp(), "empty.ags");
    writeFileSync(empty, "");
    const { code, stderr } = runCli(["read", empty, "PROJ"]);
    expect(code).toBe(4);
    expect(stderr).toContain("none");
  });
});

// ---- validate --------------------------------------------------------------
describe("cli (in-process): validate", () => {
  it("a clean file → exit 0, human summary says clean", () => {
    const { code, stdout } = runCli(["validate", CLEAN, "--no-warnings"]);
    expect(code).toBe(0);
    expect(stdout).toContain("clean");
  });

  it("bare `lat <file>` is shorthand for validate", () => {
    expect(runCli([CLEAN, "--no-warnings"]).code).toBe(0);
  });

  it("a file with findings → non-zero exit, lists the findings", () => {
    const { code, stdout } = runCli(["validate", DIRTY]);
    expect(code).toBe(1);
    expect(stdout).toContain("finding(s)");
  });

  it("--json emits the report JSON with a trailing newline", () => {
    const { code, stdout } = runCli(["validate", CLEAN, "--json"]);
    expect(code).toBe(0);
    expect(stdout.endsWith("\n")).toBe(true);
    expect(() => JSON.parse(stdout)).not.toThrow();
  });

  it("--ndjson emits ndjson", () => {
    const { code, stdout } = runCli(["validate", DIRTY, "--ndjson"]);
    expect(code).toBe(1);
    // ndjson is one JSON object per line — the first line parses on its own.
    expect(() => JSON.parse(stdout.trimEnd().split("\n")[0]!)).not.toThrow();
  });

  it("--json-out writes the JSON report to a file", () => {
    const out = join(tmp(), "report.json");
    const { code } = runCli(["validate", CLEAN, "--json-out", out]);
    expect(code).toBe(0);
    expect(() => JSON.parse(readFileSync(out, "utf8"))).not.toThrow();
  });

  it("--out writes the human summary to a file", () => {
    const out = join(tmp(), "summary.txt");
    const { code, stderr } = runCli([
      "validate",
      CLEAN,
      "--no-warnings",
      "--out",
      out,
    ]);
    expect(code).toBe(0);
    expect(existsSync(out)).toBe(true);
    expect(stderr).toContain(`written to ${out}`);
  });

  it("--dict-version auto is the no-pin sentinel (does not fail)", () => {
    expect(
      runCli(["validate", CLEAN, "--no-warnings", "--dict-version", "auto"])
        .code,
    ).toBe(0);
  });

  it("--json and --ndjson together → exit 5", () => {
    expect(runCli(["validate", CLEAN, "--json", "--ndjson"]).code).toBe(5);
  });

  it("an unknown flag is rejected → exit 5", () => {
    expect(runCli(["validate", CLEAN, "--bogus"]).code).toBe(5);
  });

  it("a bad --dict-version → exit 5", () => {
    expect(runCli(["validate", CLEAN, "--dict-version", "9.9"]).code).toBe(5);
  });

  it("a malformed --dict overlay → exit 5", () => {
    const bad = join(tmp(), "bad-dict.json");
    writeFileSync(bad, "{ not valid json ");
    expect(runCli(["validate", CLEAN, "--dict", bad]).code).toBe(5);
  });

  it("validate with no file → exit 5", () => {
    expect(runCli(["validate"]).code).toBe(5);
  });

  it("validate of a missing file → exit 3", () => {
    expect(runCli(["validate", "/no/such/file.ags"]).code).toBe(3);
  });

  it("validate of a non-AGS text file → exit 4 (not parseable)", () => {
    const g = join(tmp(), "notags.txt");
    writeFileSync(g, "this is plainly not an AGS4 file\n");
    expect(runCli(["validate", g]).code).toBe(4);
  });

  it("empty argv → 'a subcommand or input file is required' (exit 5)", () => {
    const { code, stderr } = runCli([]);
    expect(code).toBe(5);
    expect(stderr).toContain("required");
  });

  it("`--` ends option parsing; the rest are positionals", () => {
    // pickVerb hits its `--` break (no verb before it → validate), and parseArgs
    // pushes everything after `--` as positionals.
    expect(runCli(["--", CLEAN]).code).toBe(0);
  });

  it("`--flag=value` form parses (the '=' branch)", () => {
    expect(
      runCli(["validate", CLEAN, "--no-warnings", "--dict-version=auto"]).code,
    ).toBe(0);
  });

  it("a valued flag BEFORE the verb is skipped by pickVerb", () => {
    // `--dict-version auto` precedes the file; pickVerb must step over the value to
    // find that the first bare token (the path) is not a subcommand → validate.
    expect(
      runCli(["--dict-version", "auto", CLEAN, "--no-warnings"]).code,
    ).toBe(0);
  });
});

// ---- validate --index (certificate skip) -----------------------------------
describe("cli (in-process): validate --index", () => {
  it("a fresh cert SKIPS the rule engine", () => {
    const dir = tmp();
    const src = join(dir, "clean.ags");
    writeFileSync(src, readFileSync(CLEAN));
    const cert = mintCert(src);
    const { code, stderr } = runCli([
      "validate",
      src,
      "--index",
      cert,
      "--no-warnings",
    ]);
    expect(code).toBe(0);
    expect(stderr).toContain("rule engine skipped");
  });

  it("a stale cert is a NOTE, not an error — the file is still checked", () => {
    const dir = tmp();
    const src = join(dir, "clean.ags");
    writeFileSync(src, readFileSync(CLEAN));
    const cert = mintCert(src);
    // Break the file after minting: the cert is stale AND there are real findings.
    writeFileSync(src, `${readFileSync(CLEAN, "utf8")}\r\n"GROUP","EXTRA"\r\n`);
    const { code, stderr } = runCli(["validate", src, "--index", cert]);
    expect(code).toBe(1); // the engine ran and found the breakage
    expect(stderr).toContain("not used");
  });
});

// ---- fix -------------------------------------------------------------------
describe("cli (in-process): fix", () => {
  it("writes a sibling .fixed file and returns 0 on a clean file", () => {
    const dir = tmp();
    const src = join(dir, "data.ags");
    copyFileSync(CLEAN, src);
    const { code, stderr } = runCli(["fix", src]);
    expect(code).toBe(0);
    expect(existsSync(join(dir, "data.fixed.ags"))).toBe(true);
    expect(stderr).toContain("→");
  });

  it("--fix-out writes to the named path", () => {
    const dir = tmp();
    const src = join(dir, "data.ags");
    copyFileSync(CLEAN, src);
    const out = join(dir, "custom.ags");
    const { code } = runCli(["fix", src, "--fix-out", out]);
    expect(code).toBe(0);
    expect(existsSync(out)).toBe(true);
  });

  it("--in-place rewrites the input", () => {
    const dir = tmp();
    const src = join(dir, "data.ags");
    copyFileSync(CLEAN, src);
    const { code } = runCli(["fix", src, "--in-place"]);
    expect(code).toBe(0);
    expect(existsSync(src)).toBe(true);
  });

  it("--json emits the machine-readable fix report", () => {
    const dir = tmp();
    const src = join(dir, "data.ags");
    copyFileSync(CLEAN, src);
    const { code, stdout } = runCli(["fix", src, "--json"]);
    expect(code).toBe(0);
    const report = JSON.parse(stdout);
    expect(report).toHaveProperty("file");
    expect(report).toHaveProperty("dest");
    expect(report).toHaveProperty("applied");
    expect(report).toHaveProperty("residual");
  });

  it("an extension-less file fixes to `<stem>.fixed`", () => {
    const dir = tmp();
    const src = join(dir, "data"); // no extension
    copyFileSync(CLEAN, src);
    expect(runCli(["fix", src]).code).toBe(0);
    expect(existsSync(join(dir, "data.fixed"))).toBe(true);
  });

  it("a fixable-but-not-clean file: applies a fix, leaves residual, exits 1", () => {
    const dir = tmp();
    const src = join(dir, "d.ags");
    copyFileSync(join(FIX, "rule8_dp_wrong_precision.ags"), src);
    // Non-JSON: residual findings remain, so the human path returns 1.
    expect(runCli(["fix", src]).code).toBe(1);
  });

  it("--json on a file with an applied fix and residual findings", () => {
    const dir = tmp();
    const src = join(dir, "d.ags");
    copyFileSync(join(FIX, "rule8_dp_wrong_precision.ags"), src);
    const { code, stdout } = runCli(["fix", src, "--json"]);
    expect(code).toBe(1); // residual > 0
    const report = JSON.parse(stdout);
    expect(report.applied.length).toBeGreaterThan(0); // the `applied` map ran
    expect(report.residual).toBeGreaterThan(0);
  });

  it("fix with no file → exit 5", () => {
    expect(runCli(["fix"]).code).toBe(5);
  });

  it("fix of a missing file → exit 3", () => {
    expect(runCli(["fix", "/no/such/file.ags"]).code).toBe(3);
  });
});

// ---- diff ------------------------------------------------------------------
describe("cli (in-process): diff", () => {
  it("identical files → 'no differences'", () => {
    const { code, stdout } = runCli(["diff", CLEAN, CLEAN]);
    expect(code).toBe(0);
    expect(stdout).toContain("no differences");
  });

  it("differing files → per-group +/-/~ lines", () => {
    // The two merge deliveries genuinely differ, so this exercises the changed-group
    // human output (`CODE: +a -r ~c`), not the 'no differences' short-circuit.
    const { code, stdout } = runCli(["diff", MERGE_A, MERGE_B]);
    expect(code).toBe(0);
    expect(stdout).toMatch(/[A-Z]{4}: \+\d+ -\d+ ~\d+/);
  });

  it("--json emits the delta as JSON", () => {
    const { code, stdout } = runCli(["diff", CLEAN, CLEAN, "--json"]);
    expect(code).toBe(0);
    expect(JSON.parse(stdout)).toHaveProperty("groups");
  });

  it("diff with one file → exit 5", () => {
    expect(runCli(["diff", CLEAN]).code).toBe(5);
  });

  it("diff of a missing file → exit 3", () => {
    expect(runCli(["diff", CLEAN, "/no/such/file.ags"]).code).toBe(3);
  });
});

// ---- merge -----------------------------------------------------------------
describe("cli (in-process): merge", () => {
  it("--on-type-clash promote merges → exit 0, writes the file, human summary", () => {
    const out = join(tmp(), "merged.ags");
    const { code, stdout } = runCli([
      "merge",
      MERGE_A,
      MERGE_B,
      "--out",
      out,
      "--on-type-clash",
      "promote",
    ]);
    expect(code).toBe(0);
    expect(existsSync(out)).toBe(true);
    expect(stdout).toContain("merged");
  });

  it("--json uses the WIRE spelling `winner_file`", () => {
    const out = join(tmp(), "merged.ags");
    const { code, stdout } = runCli([
      "merge",
      MERGE_A,
      MERGE_B,
      "--out",
      out,
      "--on-type-clash",
      "promote",
      "--json",
    ]);
    expect(code).toBe(0);
    const summary = JSON.parse(stdout);
    expect(summary.bytes).toBeGreaterThan(0);
    for (const r of summary.revisions) {
      expect(r).toHaveProperty("winner_file");
      expect(r).not.toHaveProperty("winnerFile");
    }
  });

  it("a TYPE clash under the default (error) mode → exit 6", () => {
    const out = join(tmp(), "merged.ags");
    expect(runCli(["merge", MERGE_A, MERGE_B, "--out", out]).code).toBe(6);
  });

  it("a typo'd mode is rejected → exit 5 (not treated as `error`)", () => {
    const out = join(tmp(), "merged.ags");
    expect(
      runCli([
        "merge",
        MERGE_A,
        MERGE_B,
        "--out",
        out,
        "--on-type-clash",
        "promot",
      ]).code,
    ).toBe(5);
  });

  it("needs at least two files → exit 5", () => {
    const out = join(tmp(), "merged.ags");
    expect(runCli(["merge", MERGE_A, "--out", out]).code).toBe(5);
  });

  it("--out is required → exit 5", () => {
    expect(runCli(["merge", MERGE_A, MERGE_B]).code).toBe(5);
  });

  it("a missing input → exit 3", () => {
    const out = join(tmp(), "merged.ags");
    expect(
      runCli(["merge", MERGE_A, "/no/such/file.ags", "--out", out]).code,
    ).toBe(3);
  });
});

// ---- certify ---------------------------------------------------------------
describe("cli (in-process): certify", () => {
  it("mints a certificate and prints its path to stdout → exit 0", () => {
    const dir = tmp();
    const src = join(dir, "clean.ags");
    writeFileSync(src, readFileSync(CLEAN));
    const { code, stdout } = runCli(["certify", src]);
    expect(code).toBe(0);
    expect(stdout).toContain("certificate written to");
  });

  it("--out places the certificate at the named path", () => {
    const dir = tmp();
    const src = join(dir, "clean.ags");
    writeFileSync(src, readFileSync(CLEAN));
    const cert = join(dir, "custom.ags.idx");
    const { code } = runCli(["certify", src, "--out", cert]);
    expect(code).toBe(0);
    expect(existsSync(cert)).toBe(true);
  });

  it("a file with findings cannot be certified → exit 1", () => {
    const dir = tmp();
    const src = join(dir, "dirty.ags");
    writeFileSync(src, readFileSync(DIRTY));
    const { code, stderr } = runCli(["certify", src]);
    expect(code).toBe(1);
    expect(stderr).toContain("cannot certify");
  });

  it("certify of a non-AGS file → the engine error is mapped (exit 4)", () => {
    // Not a 'cannot certify' finding-count refusal — the file cannot be parsed at all,
    // so this exercises certify's OTHER catch arm (exitCodeFor → not_ags4 → 4).
    const g = join(tmp(), "notags.txt");
    writeFileSync(g, "this is not an AGS4 file\n");
    expect(runCli(["certify", g]).code).toBe(4);
  });

  it("certify with no file → exit 5", () => {
    expect(runCli(["certify"]).code).toBe(5);
  });

  it("certify of a missing file → exit 3", () => {
    expect(runCli(["certify", "/no/such/file.ags"]).code).toBe(3);
  });
});

// ---- rules -----------------------------------------------------------------
describe("cli (in-process): rules", () => {
  it("the HUMAN view lists the rules", () => {
    const { code, stdout } = runCli(["rules"]);
    expect(code).toBe(0);
    expect(stdout).toContain("Rule");
    expect(stdout.trimEnd().split("\n").length).toBeGreaterThan(20);
  });

  it("--json emits the rules_meta catalogue", () => {
    const { code, stdout } = runCli(["rules", "--json"]);
    expect(code).toBe(0);
    expect(JSON.parse(stdout).rules.length).toBeGreaterThan(20);
  });
});

// ---- transport -------------------------------------------------------------
describe("cli (in-process): transport", () => {
  it("pack → unpack round-trips losslessly", () => {
    const dir = tmp();
    const packed = join(dir, "p.zst");
    const back = join(dir, "back.ags");
    expect(runCli(["pack", CLEAN, packed]).code).toBe(0);
    expect(runCli(["unpack", packed, back]).code).toBe(0);
    expect(readFileSync(back)).toEqual(readFileSync(CLEAN));
  });

  it(
    "lock → unlock round-trips with a --password-file",
    { timeout: 60_000 },
    () => {
      const dir = tmp();
      const pw = join(dir, "pw.txt");
      writeFileSync(pw, "correct horse battery staple\n");
      const locked = join(dir, "l.age");
      const back = join(dir, "back.ags");
      // A low --log-n keeps scrypt fast for the test (unlock reads the stored params).
      expect(
        runCli(["lock", CLEAN, locked, "--password-file", pw, "--log-n", "10"])
          .code,
      ).toBe(0);
      expect(runCli(["unlock", locked, back, "--password-file", pw]).code).toBe(
        0,
      );
      expect(readFileSync(back)).toEqual(readFileSync(CLEAN));
    },
  );

  it(
    "lock reads the passphrase from $LAT_TRANSPORT_PASSWORD when no file is given",
    { timeout: 60_000 },
    () => {
      const dir = tmp();
      const locked = join(dir, "l.age");
      const prev = process.env.LAT_TRANSPORT_PASSWORD;
      process.env.LAT_TRANSPORT_PASSWORD = "env-secret";
      try {
        expect(runCli(["lock", CLEAN, locked, "--log-n", "10"]).code).toBe(0);
      } finally {
        if (prev === undefined) delete process.env.LAT_TRANSPORT_PASSWORD;
        else process.env.LAT_TRANSPORT_PASSWORD = prev;
      }
      expect(existsSync(locked)).toBe(true);
    },
  );

  it("lock with neither a file nor $LAT_TRANSPORT_PASSWORD fails", () => {
    // resolvePassword's fail(5) is the real branch under test. In the SUBPROCESS
    // (cli.test.ts, the exit-code authority) the real process.exit(5) halts there;
    // in-process the exit mock THROWS, and that throw is caught by runTransport's own
    // try/catch, which re-fails with 6 — a mock artifact, not the shipped behaviour.
    // So assert only that it fails (the fail branch is exercised either way).
    const dir = tmp();
    const prev = process.env.LAT_TRANSPORT_PASSWORD;
    delete process.env.LAT_TRANSPORT_PASSWORD;
    try {
      expect(runCli(["lock", CLEAN, join(dir, "l.age")]).code).toBeGreaterThan(
        0,
      );
    } finally {
      if (prev !== undefined) process.env.LAT_TRANSPORT_PASSWORD = prev;
    }
  });

  it("pack with a missing operand → exit 5", () => {
    expect(runCli(["pack", CLEAN]).code).toBe(5);
  });

  it("pack of a missing input → exit 3", () => {
    expect(
      runCli(["pack", "/no/such/file.ags", join(tmp(), "p.zst")]).code,
    ).toBe(3);
  });

  it("unpack of a non-archive → exit 6", () => {
    const dir = tmp();
    const notZst = join(dir, "notzst.bin");
    writeFileSync(notZst, "definitely not a zstd frame");
    expect(runCli(["unpack", notZst, join(dir, "out.ags")]).code).toBe(6);
  });
});

// ---- excel -----------------------------------------------------------------
describe("cli (in-process): excel", () => {
  it("export → import round-trips (direction inferred from the extension)", () => {
    const dir = tmp();
    const xlsx = join(dir, "out.xlsx");
    const back = join(dir, "back.ags");
    expect(runCli(["excel", CLEAN, xlsx]).code).toBe(0);
    expect(existsSync(xlsx)).toBe(true);
    expect(runCli(["excel", xlsx, back]).code).toBe(0);
    expect(existsSync(back)).toBe(true);
  });

  it("--export forces the AGS4 → xlsx direction", () => {
    const dir = tmp();
    const out = join(dir, "forced.data"); // extension would otherwise be ambiguous
    const { code } = runCli(["excel", CLEAN, out, "--export"]);
    expect(code).toBe(0);
    expect(existsSync(out)).toBe(true);
  });

  it("an ambiguous output extension → exit 5", () => {
    expect(runCli(["excel", CLEAN, join(tmp(), "x.dat")]).code).toBe(5);
  });

  it("excel with a missing operand → exit 5", () => {
    expect(runCli(["excel", CLEAN]).code).toBe(5);
  });

  it("excel of a missing input → exit 3", () => {
    expect(
      runCli(["excel", "/no/such/file.ags", join(tmp(), "o.xlsx")]).code,
    ).toBe(3);
  });

  it("import of a file that is not a real workbook → exit 6", () => {
    // `--import` forces the .xlsx → .ags direction; fromExcel throws on the bogus
    // input, which the verb maps to 6 (the excel catch arm).
    const dir = tmp();
    const bogus = join(dir, "bogus.xlsx");
    writeFileSync(bogus, "not really a workbook");
    expect(
      runCli(["excel", bogus, join(dir, "out.ags"), "--import"]).code,
    ).toBe(6);
  });
});

// ---- small wrapper gaps (non-cli) ------------------------------------------
// Focused asserts to lift the branch axis on the tiny helper modules whose
// error / format branches the surface tests miss.
describe("wrapper helper gaps", () => {
  it("Report.toString and .file (report.ts)", () => {
    const report = validate(CLEAN);
    expect(report.file).toBe(CLEAN);
    expect(report.toString()).toContain("Report");
    // A file WITH findings takes toString's other arm ("N finding(s)", not "valid").
    expect(validate(DIRTY).toString()).toContain("finding(s)");
  });

  it("BuildResult.save / .text / .toString (build-result.ts)", () => {
    const r = new BuildResult(Buffer.from("hello world"), [], [], 0);
    const out = join(tmp(), "built.ags");
    expect(r.save(out)).toBe(out);
    expect(readFileSync(out, "utf8")).toBe("hello world");
    expect(r.text).toBe("hello world");
    expect(r.toString()).toContain("BuildResult");
  });

  it("parseValue stringifies a non-primitive input (ags-types.ts)", () => {
    // An object is neither string/number/boolean/bigint, so scalarString takes the
    // JSON.stringify fallback; an unknown type code passes the string through.
    expect(parseValue({ a: 1 }, "ZZZ")).toBe('{"a":1}');
  });

  it("ancestorChain / inheritedKeyNames reject an unknown code (registry.ts)", () => {
    expect(() => ancestorChain("ZZZZ")).toThrow();
    expect(() => inheritedKeyNames("ZZZZ")).toThrow();
  });
});
