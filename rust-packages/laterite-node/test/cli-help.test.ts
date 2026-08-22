// `lat <verb> --help` on the Node launcher (#509).
//
// This launcher had no `--help` path at all: the flag reached `rejectUnknownFlags`
// and exited 5 with ``error: `certify` does not accept --help``, while
// `README-cli.md`'s Usage block — the guide the other two launchers print —
// promises `lat <command> --help`. So the one document telling a reader the flag
// exists was shipped by a program that refused it.
//
// The Python twin of this file is `tests/test_cli_verb_help.py`; the two assert
// the same contract against the same text, because `gen_cli_readme.py` mirrors one
// authority into both packages and `tests/test_cli_readme_mirrors.py` holds them
// byte-identical. Neither launcher writes its own help prose.
import { readFileSync } from "node:fs";
import { resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { describe, expect, it, vi } from "vitest";

import { census, main } from "../ts/cli";

const pkgDir = resolve(fileURLToPath(new URL("..", import.meta.url)));
const README = readFileSync(resolve(pkgDir, "README-cli.md"), "utf8");

interface CliResult {
  code: number;
  stdout: string;
}

function runCli(argv: string[]): CliResult {
  let stdout = "";
  const outSpy = vi
    .spyOn(process.stdout, "write")
    .mockImplementation((chunk: unknown) => {
      stdout += String(chunk);
      return true;
    });
  const errSpy = vi
    .spyOn(process.stderr, "write")
    .mockImplementation(() => true);
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
  return { code, stdout };
}

// Read from the census rather than restated: the census dumps the dispatch table
// itself, so a verb added to `SPECS` arrives here already under test. A hand-kept
// list beside a dispatch table is the defect `census()` exists to catch.
const VERBS: string[] = (census() as { documented_verbs: string[] })
  .documented_verbs;

describe("lat <verb> --help", () => {
  it("has verbs to check at all", () => {
    // Zero is a bad witness: an empty list makes every case below vacuous.
    expect(VERBS.length).toBeGreaterThan(0);
  });

  it.each(VERBS)("scopes --help to `%s`", (verb) => {
    const { code, stdout } = runCli([verb, "--help"]);
    expect(code).toBe(0);
    // The whole point: not the 203-line guide the reader used --help to avoid.
    expect(stdout.trim()).not.toBe(README.trim());
    expect(stdout.split("\n").length).toBeLessThan(README.split("\n").length);
    expect(stdout).toContain(verb);
  });

  it.each(VERBS)("carries the global options under `%s`", (verb) => {
    // clap lists them under every verb; hiding them sends the reader back to the
    // document they were avoiding.
    expect(runCli([verb, "--help"]).stdout).toContain("--quiet");
  });

  it.each(["pack", "unpack", "lock", "unlock"])(
    "resolves `%s` through the shared transport section",
    (verb) => {
      // The one heading whose first word is not a verb. A first-token lookup
      // passes every other case and silently falls back to the full guide here.
      const { stdout } = runCli([verb, "--help"]);
      expect(stdout).toContain("transport");
      expect(stdout.split("\n").length).toBeLessThan(60);
    },
  );

  it("prints the whole guide for a bare --help, -h and --readme", () => {
    // One assertion over the whole set, so a failure names the flag that broke.
    const printed = ["--help", "-h", "--readme"].map((flag) => {
      const { code, stdout } = runCli([flag]);
      return { flag, code, whole: stdout.trim() === README.trim() };
    });
    expect(printed).toEqual([
      { flag: "--help", code: 0, whole: true },
      { flag: "-h", code: 0, whole: true },
      { flag: "--readme", code: 0, whole: true },
    ]);
  });

  it("scopes a bare file's --help to validate", () => {
    // `lat <file> --help` is `lat validate <file> --help`, the shorthand this
    // launcher already applies to dispatch. Measured against the binary, which
    // answers it with validate's help — its argv pre-scan splices the default verb
    // in before clap sees the flag. Falling back to the whole guide here would be
    // a launcher divergence introduced BY the fix for launcher divergence.
    const { code, stdout } = runCli(["delivery.ags", "--help"]);
    expect(code).toBe(0);
    expect(stdout.trim()).not.toBe(README.trim());
    expect(stdout).toContain("validate");
  });

  it("answers --help before refusing a missing argument", () => {
    // `certify` with no file exits 5 on the work path. --help must beat that, the
    // way clap orders it — otherwise the flag is unreachable exactly when a reader
    // needs it.
    const { code, stdout } = runCli(["certify", "--help"]);
    expect(code).toBe(0);
    expect(stdout).toContain("certify");
  });

  it("does not let --help fall through to the unknown-flag refusal", () => {
    // The original defect, stated as its own case: `rejectUnknownFlags` does not
    // know `help`, so anything that reaches it exits 5. Collected rather than
    // asserted one at a time, so a failure names every verb that regressed
    // instead of only the first.
    const refused = VERBS.filter((v) => runCli([v, "--help"]).code !== 0);
    expect(refused).toEqual([]);
  });
});
