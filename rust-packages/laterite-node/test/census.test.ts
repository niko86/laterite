// The npx launcher's half of the surface census.
//
// No single CI job builds all three `lat` launchers (the python job builds the wheel
// and the Rust binary; this job builds `dist/`). Rather than add a job, the committed
// `surface-census.json` IS the shared contract, and each job pins the surfaces it
// already has: the python job runs `tools/gen_census.py --check` over the native
// binary + uvx, and this file pins npx.
//
// Together they close the loop. Change `HANDLERS` without regenerating the census and
// this fails; let the census drift from the native binary and the python job fails.
// Neither can be dodged by touching only one surface — which is exactly how `lat merge`
// shipped in the binary (#494) and never arrived here, with every gate still green.
import { execFileSync } from "node:child_process";
import { readFileSync } from "node:fs";
import { join, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { describe, expect, it } from "vitest";

import { census } from "../ts/cli";

const pkgDir = resolve(fileURLToPath(new URL("..", import.meta.url)));
const repo = resolve(pkgDir, "..", "..");
const SSOT = JSON.parse(readFileSync(join(repo, "surface-census.json"), "utf8")) as {
  authority: string;
  surfaces: Record<
    string,
    {
      verbs: string[];
      editions: string[];
      fallback_edition: string;
      encodings: Record<string, string | null>;
    }
  >;
};

const verbsOf = (c: unknown): string[] =>
  (c as { verbs: { verb: string }[] }).verbs.map((v) => v.verb).sort();

describe("surface census: npx", () => {
  it("the committed census records what this launcher actually dispatches", () => {
    const recorded = SSOT.surfaces["cli-npx"]?.verbs;
    expect(recorded, "surface-census.json has no cli-npx row").toBeDefined();
    expect(verbsOf(census())).toEqual([...(recorded ?? [])].sort());
  });

  it("this launcher accepts exactly the authority's dictionary editions", () => {
    // The second census table. `--dict-version` goes straight to the engine here, so
    // this launcher keeps no edition list of its own — the ideal end state. Elsewhere
    // the set was hand-copied ~9 times, and the Rust CLI's copy generated its
    // rejection MESSAGE from the real list while hand-writing the arms that did the
    // accepting: bundling `4.3` would have rejected `4.3` while advertising `4.3`.
    const c = census() as { editions: string[]; fallback_edition: string };
    expect(c.editions).toEqual(SSOT.surfaces[SSOT.authority]?.editions);
    expect(c.fallback_edition).toBe(SSOT.surfaces[SSOT.authority]?.fallback_edition);
  });

  it("every bundled edition is actually ACCEPTED, not merely listed", () => {
    // Exit 5 is "bad arguments" — what the CLI returns for an edition it does not
    // know. Passes trivially today; written for the day the dictionary grows.
    const BIN = join(pkgDir, "bin.mjs");
    const CLEAN = resolve(
      pkgDir,
      "..",
      "laterite-ags4-validator",
      "tests",
      "fixtures",
      "clean_minimal.ags",
    );
    for (const edition of (census() as { editions: string[] }).editions) {
      let code = 0;
      try {
        execFileSync("node", [BIN, "validate", CLEAN, "--dict-version", edition], {
          encoding: "utf8",
          stdio: "pipe",
        });
      } catch (e) {
        code = (e as { status?: number }).status ?? 1;
      }
      expect(code, `npx rejected --dict-version ${edition}, which the dictionary bundles`).not.toBe(
        5,
      );
    }
  });

  it("this launcher implements every verb the native binary does", () => {
    // The authority's row, minus `census` itself — a hidden machine door, not a
    // user-facing verb the launchers must mirror.
    const authority = (SSOT.surfaces[SSOT.authority]?.verbs ?? []).filter(
      (v) => v !== "census",
    );
    expect(authority.length).toBeGreaterThan(0);
    const mine = new Set(verbsOf(census()));
    const missing = authority.filter((v) => !mine.has(v));
    expect(missing, `npx is missing verb(s) the native binary ships: ${missing}`).toEqual(
      [],
    );
  });

  it("every advertised verb is a real door — and none of them CRASHES", () => {
    // The census is only worth something if it reads the same table `main` dispatches
    // through. Prove it end-to-end, through the built `bin.mjs`.
    //
    // Two distinct failures, and the discriminator for each:
    //
    //  1. THE DOOR ISN'T THERE. There is no "unknown subcommand" error to look for —
    //     an unrecognised token is spliced to `validate` and treated as a FILENAME
    //     (`lat foo.ags` ≡ `lat validate foo.ags`). So a verb the CLI does not know
    //     announces itself as `error: <verb>: not found`. That exact string is the
    //     tell, and it is what `merge` would have printed before #494 reached here.
    //
    //  2. THE DOOR IS THERE AND FALLS OVER. An uncaught throw prints a V8 stack trace
    //     — which is how `lat rules` was found crashing on this launcher (it iterated
    //     `{schema_version, rules: []}` as if it were an array; only `--json`, the one
    //     path the tests covered, worked). A verb every launcher HAS but one of them
    //     cannot RUN is invisible to a verb-table diff. Running each one is what sees it.
    //
    // Exiting non-zero for want of arguments is expected and fine — what is asserted
    // is that the verb is reachable and fails on its own terms.
    const BIN = join(pkgDir, "bin.mjs");
    for (const verb of verbsOf(census())) {
      let stderr = "";
      try {
        execFileSync("node", [BIN, verb], { encoding: "utf8", stdio: "pipe" });
      } catch (e) {
        stderr = (e as { stderr?: string }).stderr ?? "";
      }
      expect(stderr, `census advertises '${verb}', but the CLI took it for a filename`).not.toContain(
        `error: ${verb}: not found`,
      );
      expect(stderr, `'${verb}' CRASHED (uncaught throw):\n${stderr}`).not.toMatch(
        /^\s*(TypeError|ReferenceError|RangeError|SyntaxError):/m,
      );
    }
  });
});

// --- the encoding table: the bug a knob-NAME gate structurally cannot see -----
//
// `--encoding` existed on this launcher and on the native binary, spelled the same,
// and did NOTHING here: every handler accepted the flag and dropped it. A census
// that compares flag NAMES sees agreement. Only comparing what the surfaces PRODUCE
// finds it. That is the whole thesis of the output-value gate, and this is the bug
// that proved it.
describe("surface census: encodings", () => {
  it("resolves every probe label exactly as the authority does", () => {
    const recorded = SSOT.surfaces[SSOT.authority]?.encodings;
    expect(recorded, "surface-census.json has no authority encodings row").toBeDefined();
    const mine = (census() as { encodings: Record<string, string | null> }).encodings;
    expect(mine).toEqual(recorded);
  });

  it("an unknown label resolves to NOTHING — never a silent UTF-8 fallback", () => {
    // The policy pin. This surface used to answer "UTF-8" here, so a caller who typed
    // `cp1252x` was handed text decoded by an encoding they never asked for, with no
    // error. `C3 A9` is `é` in UTF-8 and `Ã©` in cp1252 — both decode cleanly, so the
    // file then "validated" with the wrong text in it.
    const mine = (census() as { encodings: Record<string, string | null> }).encodings;
    expect(mine["cp1252x"]).toBeNull();
  });

  it("`latin-9` resolves here, not only in the native binary", () => {
    // These two labels lived in a PRIVATE table inside the `lat` binary, so
    // `--encoding latin-9` worked there and was rejected by every other surface.
    const mine = (census() as { encodings: Record<string, string | null> }).encodings;
    expect(mine["latin9"]).toBe("ISO-8859-15");
    expect(mine["latin-9"]).toBe("ISO-8859-15");
  });
});
