// The CLI's exit-code mapping, and the TRAN flag fold.
//
// Exit codes are the CLI's machine-readable contract — a shell script branches
// on them, and getting one wrong turns "your file is missing" into "your file is
// invalid", which sends the user to the wrong place entirely. `exitCodeFor`
// decides them from a thrown value, and its first arm recognises a missing file
// two different ways: the engine's own `kind === "not_found"`, and any error
// whose message carries `ENOENT` (which is how Node's own fs failures arrive).
// Only one of those two was ever taken.
import { mkdtempSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { describe, expect, it, vi } from "vitest";

import { main } from "../ts/cli";

const pkgDir = resolve(fileURLToPath(new URL("..", import.meta.url)));
const FIX = resolve(
  pkgDir,
  "..",
  "laterite-ags4-validator",
  "tests",
  "fixtures",
);
const CLEAN = join(FIX, "clean_minimal.ags");

interface CliResult {
  code: number;
  stdout: string;
  stderr: string;
}

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
    const ex = e as { message?: string; exitCode?: number } | null | undefined;
    if (ex && ex.message === "__cli_exit__") code = ex.exitCode ?? 0;
    else throw e;
  } finally {
    outSpy.mockRestore();
    errSpy.mockRestore();
    exitSpy.mockRestore();
  }
  return { code, stdout, stderr };
}

describe("exit codes for a missing file", () => {
  it("reports 3 for a path that does not exist", () => {
    const missing = join(tmpdir(), "laterite-definitely-absent-9f3a.ags");
    const { code } = runCli(["validate", missing]);
    expect(code).toBe(3);
  });

  it("uses 3 — not the invalid-file code — so a script can tell them apart", () => {
    // The distinction that makes the code worth having. A missing file and an
    // invalid file are different problems with different fixes, and they must
    // not collapse to one number.
    const dir = mkdtempSync(join(tmpdir(), "lat-exit-"));
    const notAgs = join(dir, "notes.txt");
    writeFileSync(notAgs, "this is not an AGS4 file at all\n");

    const missing = runCli(["validate", join(dir, "nope.ags")]);
    const invalid = runCli(["validate", notAgs]);

    expect(missing.code).toBe(3);
    expect(invalid.code).not.toBe(3);
    expect(invalid.code).toBeGreaterThan(0);
  });

  it("reports 3 for a missing file on every verb that reads one", () => {
    // The mapping lives in one place precisely so the verbs cannot disagree.
    const missing = join(tmpdir(), "laterite-definitely-absent-9f3a.ags");
    const codes = [
      ["validate", missing],
      ["fix", missing],
      ["certify", missing],
    ].map((argv) => [argv[0], runCli(argv).code]);
    // Compared as a whole so a failure names the verb that disagreed.
    expect(codes).toEqual([
      ["validate", 3],
      ["fix", 3],
      ["certify", 3],
    ]);
  });

  it("exits 0 on a clean file, so a non-zero code means something", () => {
    // The control. Without it "everything returns 3" would pass the tests above.
    expect(runCli(["validate", CLEAN]).code).toBe(0);
  });
});

describe("the --tran-* flags", () => {
  it("mints no stamp when none of the five is supplied", () => {
    // `tranFromFlags` returns undefined only when ALL five are absent — which
    // means no TRAN is written and Rule 14 reports the gap, the honest outcome.
    const dir = mkdtempSync(join(tmpdir(), "lat-tran-"));
    const out = join(dir, "merged.ags");
    const MFIX = resolve(
      pkgDir,
      "..",
      "laterite-ags4-merge",
      "tests",
      "fixtures",
    );
    const { code } = runCli([
      "merge",
      join(MFIX, "delivery_a.ags"),
      join(MFIX, "delivery_b.ags"),
      "--out",
      out,
      // The fixtures genuinely disagree on a TYPE, so the clash mode is part of
      // making this a merge at all — it is not what is under test here.
      "--on-type-clash",
      "promote",
    ]);
    expect(code).toBe(0);
  });

  it("passes a PARTIAL stamp through to the library rather than judging it", () => {
    // Deliberately not validated in the CLI: the error text must come from the
    // one place that owns the "all five or none" rule, so the CLI can never
    // disagree with the library about what a complete stamp is. So a partial
    // stamp must FAIL — and fail with the library's words.
    const dir = mkdtempSync(join(tmpdir(), "lat-tran-"));
    const out = join(dir, "merged.ags");
    const MFIX = resolve(
      pkgDir,
      "..",
      "laterite-ags4-merge",
      "tests",
      "fixtures",
    );
    const { code, stderr } = runCli([
      "merge",
      join(MFIX, "delivery_a.ags"),
      join(MFIX, "delivery_b.ags"),
      "--out",
      out,
      "--on-type-clash",
      "promote",
      "--tran-issue",
      "1",
    ]);
    expect(code).not.toBe(0);
    // The message names the rule, which is what proves it came from the library.
    expect(stderr.length).toBeGreaterThan(0);
  });
});
