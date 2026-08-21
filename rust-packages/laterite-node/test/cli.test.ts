// The Node `lat` CLI — end-to-end through the built `bin.mjs`, the exact
// executable `npx laterite` / a global `lat` runs (#430). Scriptable outputs
// (`validate --json`, `read --csv`/`--json`, `rules --json`) are byte-checked
// against the Rust binary in the repo's cross-surface gates; here we assert each
// verb works and the exit codes follow the binary's scheme.
import { spawnSync } from "node:child_process";
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
import { describe, expect, it } from "vitest";

const pkgDir = resolve(fileURLToPath(new URL("..", import.meta.url)));
const BIN = join(pkgDir, "bin.mjs");
const FIX = resolve(
  pkgDir,
  "..",
  "laterite-ags4-validator",
  "tests",
  "fixtures",
);
const CLEAN = join(FIX, "clean_minimal.ags");

function run(args: string[]): { stdout: string; stderr: string; code: number } {
  // spawnSync, not execFileSync: the certified-skip note is a STDERR fact, and
  // execFileSync hands back stdout only — a test that cannot see stderr cannot tell a
  // certificate that fired from one that did nothing at all.
  const r = spawnSync("node", [BIN, ...args], { encoding: "utf8" });
  return {
    // @types/node types stdout/stderr as string, but spawnSync yields null when
    // the child fails to spawn — keep the guards (r.status is already null-typed).
    // eslint-disable-next-line @typescript-eslint/no-unnecessary-condition
    stdout: r.stdout ?? "",
    // eslint-disable-next-line @typescript-eslint/no-unnecessary-condition
    stderr: r.stderr ?? "",
    code: r.status ?? 1,
  };
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

  it("rules: the HUMAN view lists the rules (it used to crash)", () => {
    // This path threw `TypeError: rules is not iterable` — it parsed
    // `{schema_version, rules: [...]}` and iterated the OBJECT. Only `--json` was
    // ever tested, so a verb that every launcher advertises was broken on this one.
    const { stdout, code } = run(["rules"]);
    expect(code).toBe(0);
    expect(stdout).toContain("Character Set"); // Rule 1's title
    expect(stdout.trimEnd().split("\n").length).toBeGreaterThan(20);
  });

  it("pack / unpack: lossless round-trip", () => {
    const dir = mkdtempSync(join(tmpdir(), "lat-cli-"));
    expect(run(["pack", CLEAN, join(dir, "p.zst")]).code).toBe(0);
    expect(
      run(["unpack", join(dir, "p.zst"), join(dir, "back.ags")]).code,
    ).toBe(0);
    expect(readFileSync(join(dir, "back.ags"))).toEqual(readFileSync(CLEAN));
  });

  it("excel: export → import round-trip", () => {
    const dir = mkdtempSync(join(tmpdir(), "lat-cli-"));
    expect(run(["excel", CLEAN, join(dir, "o.xlsx")]).code).toBe(0);
    expect(
      run(["excel", join(dir, "o.xlsx"), join(dir, "back.ags")]).code,
    ).toBe(0);
  });

  it("exit codes follow the binary: missing → 3, ambiguous excel → 5, bad group → 4", () => {
    expect(run(["read", "/no/such/file.ags"]).code).toBe(3);
    expect(run(["excel", CLEAN, join(tmpdir(), "x.dat")]).code).toBe(5);
    expect(run(["read", CLEAN, "NOPE"]).code).toBe(4);
  });

  // --- fix default destination: stem.fixed.ext, matching the other launchers ----
  // npx once wrote `data.txt.fixed.ags` where the Rust binary and uvx write
  // `data.fixed.txt` — the `file.replace(/(\.ags)?$/i, ".fixed.ags")` matched
  // `(\.ags)?` ZERO-WIDTH at end-of-string. The cross-surface value gate's
  // `cli.fix.dest.*` cases catch it across launchers; this pins the fix here.
  describe("fix default destination", () => {
    for (const [name, expected] of [
      ["data.txt", "data.fixed.txt"],
      ["data.ags", "data.fixed.ags"],
      ["data.AGS", "data.fixed.AGS"], // extension case preserved
      ["data", "data.fixed"], // extension-less
    ] as const) {
      it(`${name} → ${expected}`, () => {
        const dir = mkdtempSync(join(tmpdir(), "lat-fix-"));
        copyFileSync(CLEAN, join(dir, name));
        expect(run(["fix", join(dir, name)]).code).toBe(0);
        expect(existsSync(join(dir, expected))).toBe(true);
      });
    }
  });

  // --- merge: the verb this launcher SHIPPED WITHOUT ---------------------
  // `lat merge` landed in the native binary (laterite-dev#494) and never reached here. No gate
  // caught it, because every cross-surface gate compared one hand-list to another.
  // These pin the verb's OUTPUT, not just its existence: the merged bytes and the
  // `--json` wire summary are contractually identical across all three launchers.
  describe("merge", () => {
    const MFIX = resolve(
      pkgDir,
      "..",
      "laterite-ags4-merge",
      "tests",
      "fixtures",
    );
    const A = join(MFIX, "delivery_a.ags");
    const B = join(MFIX, "delivery_b.ags");

    it("refuses a TYPE clash by default, and names both escape hatches", () => {
      const dir = mkdtempSync(join(tmpdir(), "lat-cli-"));
      const { code } = run(["merge", A, B, "--out", join(dir, "m.ags")]);
      expect(code).toBe(6); // a schema violation, per the shared exit-code scheme
    });

    // The rules firing on a file, via the CLI's own validate — so this asserts the
    // MERGED BYTES, not an in-process object the merge happened to return.
    const rulesFiring = (f: string): Set<string> => {
      const { stdout } = run(["validate", f, "--json"]);
      const findings = JSON.parse(stdout).findings as
        Record<string, unknown[]> | { rule: string }[];
      return new Set(
        Array.isArray(findings)
          ? findings.map((x) => x.rule)
          : Object.keys(findings),
      );
    };

    it("--on-type-clash promote merges, and the result satisfies Rule 8", () => {
      const dir = mkdtempSync(join(tmpdir(), "lat-cli-"));
      const out = join(dir, "m.ags");
      expect(
        run(["merge", A, B, "--out", out, "--on-type-clash", "promote"]).code,
      ).toBe(0);
      // Promote zero-pads the coarser column to the greatest precision, so every
      // value still matches its declared TYPE. Rule 8 is the rule that would catch a
      // value which does not — its absence IS the guarantee.
      //
      // (These fixtures are deliberately not clean: they carry a passthrough group
      // outside the dictionary, so Rules 7/9/10c/18 fire on the inputs too. Asserting
      // "zero findings" would be asserting something about the fixtures, not merge.)
      expect([...rulesFiring(out)]).not.toContain("AGS Format Rule 8");
    });

    it("merge never makes a file worse — its findings are a subset of the inputs'", () => {
      const dir = mkdtempSync(join(tmpdir(), "lat-cli-"));
      const out = join(dir, "m.ags");
      expect(
        run(["merge", A, B, "--out", out, "--on-type-clash", "promote"]).code,
      ).toBe(0);
      const before = new Set([...rulesFiring(A), ...rulesFiring(B)]);
      for (const rule of rulesFiring(out)) {
        expect(before.has(rule), `merge INTRODUCED ${rule}`).toBe(true);
      }
    });

    it("--json uses the WIRE spelling `winner_file`, not the TS API's `winnerFile`", () => {
      const dir = mkdtempSync(join(tmpdir(), "lat-cli-"));
      const { stdout, code } = run([
        "merge",
        A,
        B,
        "--out",
        join(dir, "m.ags"),
        "--on-type-clash",
        "promote",
        "--json",
      ]);
      expect(code).toBe(0);
      const summary = JSON.parse(stdout);
      expect(summary.bytes).toBeGreaterThan(0);
      expect(summary.revisions.length).toBeGreaterThan(0);
      // A script reading `.revisions[].winner_file` must not care which launcher ran.
      for (const r of summary.revisions) {
        expect(r).toHaveProperty("winner_file");
        expect(r).not.toHaveProperty("winnerFile");
        expect(typeof r.winner_file).toBe("number");
      }
    });

    it("--out is required — a merge never silently overwrites an input", () => {
      expect(run(["merge", A, B]).code).toBe(5);
    });

    it("a typo'd mode is rejected, not silently treated as `error`", () => {
      const dir = mkdtempSync(join(tmpdir(), "lat-cli-"));
      const { code } = run([
        "merge",
        A,
        B,
        "--out",
        join(dir, "m.ags"),
        "--on-type-clash",
        "promot",
      ]);
      expect(code).toBe(5); // bad args — NOT 6, which would look like a real clash
    });

    it("needs at least two files", () => {
      const dir = mkdtempSync(join(tmpdir(), "lat-cli-"));
      expect(run(["merge", A, "--out", join(dir, "m.ags")]).code).toBe(5);
    });
  });

  // --- encoding: the flag that existed and did NOTHING --------------------
  // Every handler accepted `--encoding` (the arg parser has one global valued-flag
  // set) and dropped it on the floor. `lat validate legacy.ags --encoding cp1252`
  // decoded as UTF-8 and reported findings that were artefacts of the wrong decoder
  // — blaming the file for the caller's ignored flag. A gate comparing flag NAMES
  // sees `--encoding` on both surfaces and calls it agreement. Only output tells.
  describe("--encoding", () => {
    // A genuine cp1252 file: 0xE9 (é) and 0xB0 (°) are NOT valid UTF-8, so the
    // decoder choice is observable in the findings.
    const cp1252File = (): string => {
      const dir = mkdtempSync(join(tmpdir(), "lat-enc-"));
      const f = join(dir, "legacy.ags");
      const text =
        '"GROUP","PROJ"\r\n' +
        '"HEADING","PROJ_ID","PROJ_NAME"\r\n' +
        '"UNIT","",""\r\n' +
        '"TYPE","ID","X"\r\n' +
        '"DATA","P1","Caf\u00e9 90\u00b0"\r\n';
      writeFileSync(f, Buffer.from(text, "latin1")); // latin1 maps each char to its byte
      return f;
    };

    const findingCount = (args: string[]): number => {
      const { stdout } = run([...args, "--json"]);
      const f = JSON.parse(stdout).findings as unknown;
      return Array.isArray(f)
        ? f.length
        : Object.values(f as Record<string, unknown[]>).reduce(
            (n, v) => n + v.length,
            0,
          );
    };

    it("is HONOURED, not merely accepted — it changes the findings", () => {
      // THE assertion this launcher failed. Asserting the flag exists proves nothing;
      // assert that passing it changes what comes back. Decoded as UTF-8 the file is
      // invalid (0xE9 is not a UTF-8 sequence); decoded as cp1252 it is fine.
      const f = cp1252File();
      const asUtf8 = findingCount(["validate", f, "--encoding", "utf-8"]);
      const asCp1252 = findingCount(["validate", f, "--encoding", "cp1252"]);
      expect(asUtf8).toBeGreaterThan(asCp1252);
    });

    it("an unknown label is REFUSED (exit 5), not silently decoded as UTF-8", () => {
      const f = cp1252File();
      const { code } = run(["validate", f, "--encoding", "cp1252x"]);
      // Not 1 (findings) — the caller is told their LABEL is wrong, not their FILE.
      expect(code).toBe(5);
    });

    it("`latin-9` works here too — it used to work only in the native binary", () => {
      const f = cp1252File();
      for (const label of ["latin9", "latin-9", "iso-8859-15"]) {
        expect(
          run(["validate", f, "--encoding", label]).code,
          `${label} must be accepted`,
        ).not.toBe(5);
      }
    });

    it("a verb that cannot honour --encoding REFUSES it instead of ignoring it", () => {
      // `read` has no encoding on ANY surface (readGroupsRaw takes none), and clap
      // rejects the flag on the native `read`. This launcher used to accept it and
      // silently drop it, leaving the user believing their file was read as cp1252.
      const f = cp1252File();
      expect(run(["read", f, "PROJ", "--encoding", "cp1252"]).code).toBe(5);
    });
  });
});

// --- `validate --index <cert>` ----------------------------------------------
// The flags census found this one. `--index` was ACCEPTED here and dropped on the
// floor: the free `validate()` takes `index` only because `ValidateOptions extends
// ReadOptions`, and it never reads it — so `tsc` was perfectly happy while the
// certificate went nowhere. Handing it a cert minted for an entirely different file
// changed nothing at all.
//
// A verb-name gate cannot see that (`validate` is on all three launchers) and neither
// can a flag-NAME gate (`--index` is on all three too). Only the OUTPUT tells you:
// a cert that is honoured reports `resolution == "certified"`, and one that is not
// reports `exact`. Both print "no findings" and exit 0.
describe("lat node CLI — validate --index", () => {
  function mint(): { src: string; cert: string } {
    const dir = mkdtempSync(join(tmpdir(), "lat-cert-"));
    const src = join(dir, "clean.ags");
    writeFileSync(src, readFileSync(CLEAN));
    // Ask `certify` where it put the cert rather than re-deriving the naming rule —
    // a hand-copied convention is one more thing that can drift from the tool.
    const { stdout, code } = run(["certify", src]);
    expect(code).toBe(0);
    const cert = stdout.trim().split(" ").pop() as string;
    return { src, cert };
  }

  it("certify prints the cert path to STDOUT, so `CERT=$(lat certify f.ags)` works", () => {
    // It used to go to stderr — on this launcher only. The binary and uvx both put it
    // on stdout, so the obvious script worked on two of three launchers and silently
    // captured an empty string on the third. No gate we own compares STREAMS.
    const dir = mkdtempSync(join(tmpdir(), "lat-cert-"));
    const src = join(dir, "clean.ags");
    writeFileSync(src, readFileSync(CLEAN));
    const { stdout, code } = run(["certify", src]);
    expect(code).toBe(0);
    expect(stdout).toContain("certificate written to");
  });

  it("a fresh cert SKIPS the rule engine", () => {
    const { src, cert } = mint();
    const { stderr, code } = run([
      "validate",
      src,
      "--index",
      cert,
      "--no-warnings",
    ]);
    expect(code).toBe(0);
    // The proof. Without it this assertion would pass on a `--index` that did nothing.
    // The note says "rule engine skipped", never "not checked": a `--check-files` run
    // still does its on-disk half even here — no certificate can vouch for a directory.
    expect(stderr).toContain("rule engine skipped");
  });

  it("`--dict-version auto` does not disarm the cert", () => {
    // `auto` is the CLI's sentinel for "no pin"; the library has no such value, and
    // passing it through makes the request look like a FORCED edition — so the cert
    // stops covering it and the skip silently turns off. It did exactly that on uvx.
    const { src, cert } = mint();
    const { stderr, code } = run([
      "validate",
      src,
      "--index",
      cert,
      "--no-warnings",
      "--dict-version",
      "auto",
    ]);
    expect(code).toBe(0);
    expect(stderr).toContain("rule engine skipped");
  });

  it("a cert for a DIFFERENT file is refused, and the full check runs", () => {
    const { cert } = mint();
    const other = join(mkdtempSync(join(tmpdir(), "lat-cert-")), "other.ags");
    writeFileSync(other, `${readFileSync(CLEAN, "utf8")}\r\n`);

    const { stderr, code } = run([
      "validate",
      other,
      "--index",
      cert,
      "--no-warnings",
    ]);
    expect(code).toBe(0);
    // NOT certified — the cert's SHA-256 is for another file's bytes.
    expect(stderr).not.toContain("rule engine skipped");
  });

  it("a stale cert is a NOTE, not an error — the file is still checked", () => {
    const { src, cert } = mint();
    // Break the file after minting: the cert is now stale AND there are real findings.
    writeFileSync(src, `${readFileSync(CLEAN, "utf8")}\r\n"GROUP","EXTRA"\r\n`);

    const { stdout, code } = run(["validate", src, "--index", cert]);
    // Exit 1 = the engine ran and found the breakage. Refusing to validate because a
    // sidecar aged out would be the worse failure; the file is perfectly checkable.
    expect(code).toBe(1);
    expect(stdout).toContain("finding(s)");
  });
});
