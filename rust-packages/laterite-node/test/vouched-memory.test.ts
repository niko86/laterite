// A vouched `validate(file, { index })` holds a buffer, not the file (#1131) — through
// the library, and through `lat validate --index`, which used to build a handle first
// and so read, parse and hash the whole file before the validate hashed it again.
//
// The instrument is a child process's own peak resident set: the addon's memory is
// native, where no JS heap figure can see it, and this process's own peak carries
// every test that ran before. A `--require` hook prints the child's peak on exit.
import { spawnSync } from "node:child_process";
import { mkdtempSync, statSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { describe, expect, it, vi } from "vitest";
import { read } from "../ts/index";

// Each test spawns three children over a multi-MB file; under CI coverage that
// overruns vitest's stock 5 s. The timeout is a ceiling, not a delay.
vi.setConfig({ testTimeout: 60_000 });

const pkgDir = resolve(fileURLToPath(new URL("..", import.meta.url)));
const BIN = join(pkgDir, "bin.mjs");
const LIB = join(pkgDir, "dist", "index.cjs");

const FILE_MIB = 64;

/** Error-clean AGS4 of `FILE_MIB` MiB, so it can be certified: the size is one long
 *  PROJ_NAME, which keeps the engine runs short. */
function bigClean(): string {
  return [
    '"GROUP","PROJ"',
    '"HEADING","PROJ_ID","PROJ_NAME"',
    '"UNIT","",""',
    '"TYPE","ID","X"',
    `"DATA","P1","${"A".repeat(FILE_MIB << 20)}"`,
    '"GROUP","TRAN"',
    '"HEADING","TRAN_ISNO","TRAN_DATE","TRAN_PROD","TRAN_STAT","TRAN_AGS","TRAN_RECV"',
    '"UNIT","","yyyy-mm-dd","","","",""',
    '"TYPE","X","DT","X","X","X","X"',
    '"DATA","1","2026-08-05","Producer","Draft","4.1.1","Recipient"',
    '"GROUP","UNIT"',
    '"HEADING","UNIT_UNIT","UNIT_DESC"',
    '"UNIT","",""',
    '"TYPE","X","X"',
    '"DATA","yyyy-mm-dd","Date"',
    '"GROUP","TYPE"',
    '"HEADING","TYPE_TYPE","TYPE_DESC"',
    '"UNIT","",""',
    '"TYPE","X","X"',
    '"DATA","ID","Unique identifier"',
    '"DATA","X","Text"',
    '"DATA","DT","Date"',
    "",
  ].join("\r\n");
}

const dir = mkdtempSync(join(tmpdir(), "laterite-vouched-"));
const HOOK = join(dir, "peak.cjs");
// `writeSync`, not `process.stderr.write`: a pipe write is asynchronous on macOS, and
// nothing queued in an exit handler is flushed.
writeFileSync(
  HOOK,
  'process.on("exit", () => require("node:fs").writeSync(2, ' +
    '"\\npeak-rss-kib=" + process.resourceUsage().maxRSS + "\\n"));\n',
);

/** Run `node <args>` under the hook; its exit code, stderr, and peak RSS in bytes. */
function peak(args: string[]): {
  code: number;
  stdout: string;
  stderr: string;
  rss: number;
} {
  const r = spawnSync("node", ["--require", HOOK, ...args], {
    encoding: "utf8",
  });
  const m = /peak-rss-kib=(\d+)/.exec(r.stderr);
  if (!m) throw new Error(`no peak reported: ${r.stderr}`);
  return {
    code: r.status ?? 1,
    stdout: r.stdout,
    stderr: r.stderr,
    rss: Number(m[1]) * 1024,
  };
}

describe("a vouched validate holds a buffer, not the file (#1131)", () => {
  const ags = join(dir, "big.ags");
  writeFileSync(ags, bigClean());
  const size = statSync(ags).size;
  const idx = read(ags).certify();

  it("through the library", () => {
    const run = (call: string) =>
      peak([
        "-e",
        `const { validate } = require(${JSON.stringify(LIB)});` +
          `const r = ${call};` +
          'if (!r.isValid) throw new Error("not valid");' +
          "process.stdout.write(String(r.certified));",
      ]);
    const file = JSON.stringify(ags);
    const idle = peak(["-e", `require(${JSON.stringify(LIB)})`]);
    const vouched = run(`validate(${file}, { index: ${JSON.stringify(idx)} })`);
    expect(vouched.code).toBe(0);
    expect(vouched.stdout).toBe("true");
    // The positive control: no cert, so the engine runs and the bytes are held.
    // Half the file, not all of it: the engine holds about one copy, so a floor
    // at the full size sat on the edge and the idle child's own noise flipped it
    // (#1156's first CI run). Half still clears the vouched ceiling of a quarter.
    const engine = run(`validate(${file})`);
    expect(engine.code).toBe(0);
    expect(engine.rss - idle.rss).toBeGreaterThanOrEqual(size / 2);
    expect(vouched.rss).toBeLessThan(idle.rss + size / 4);
  });

  it("through `lat validate --index`", () => {
    const idle = peak([BIN, "rules"]);
    const vouched = peak([BIN, "validate", ags, "--index", idx]);
    expect(vouched.code).toBe(0);
    expect(vouched.stderr).toContain("rule engine skipped");
    const engine = peak([BIN, "validate", ags]);
    expect(engine.code).toBe(0);
    // Half the file, for the reason given in the library case above.
    expect(engine.rss - idle.rss).toBeGreaterThanOrEqual(size / 2);
    expect(vouched.rss).toBeLessThan(idle.rss + size / 4);
  });
});
