// Value-domain gate: the accepted AGS4 edition set + the "unknown edition"
// message are single-sourced from the Rust `DictVersion::ALL` (via the shared
// `editions_joined`). The Node crate can't host Rust unit tests (napi-3 link),
// so this vitest pins it: every bundled edition resolves, and a bogus one is
// rejected with a message naming EVERY bundled edition (proving the list isn't a
// stale hand-written copy).
import { readFileSync } from "node:fs";
import { describe, expect, it } from "vitest";
import { BadDictError, validate } from "../ts/index";

// Read the bundled editions from the SAME JSON the dictionary is generated from.
const DICT = JSON.parse(
  readFileSync(
    new URL(
      "../../laterite-ags4-reference/data/ags_dictionary.json",
      import.meta.url,
    ),
    "utf8",
  ),
);
const EDITIONS: string[] = DICT.editions;

const AGS =
  '"GROUP","PROJ"\r\n"HEADING","PROJ_ID"\r\n"UNIT",""\r\n"TYPE","ID"\r\n"DATA","P1"\r\n';

describe("edition value domain", () => {
  it("accepts every bundled edition, plus auto / undefined", () => {
    expect(EDITIONS.length).toBeGreaterThanOrEqual(5);
    for (const ed of EDITIONS) {
      expect(() =>
        validate(undefined, { text: AGS, dictVersion: ed }),
      ).not.toThrow();
    }
    expect(() =>
      validate(undefined, { text: AGS, dictVersion: "auto" }),
    ).not.toThrow();
    expect(() => validate(undefined, { text: AGS })).not.toThrow();
  });

  it("rejects an unknown edition with a message listing every bundled edition", () => {
    let caught: unknown;
    try {
      validate(undefined, { text: AGS, dictVersion: "9.9" });
    } catch (e) {
      caught = e;
    }
    expect(caught).toBeInstanceOf(BadDictError);
    const msg = (caught as Error).message;
    for (const ed of EDITIONS) {
      expect(msg, `message must name ${ed}`).toContain(ed);
    }
  });

  it("every TSDoc edition list in ts/ matches the dictionary (#927)", () => {
    // TSDoc prose cannot derive from a runtime call, so the full-list
    // spellings (`"4.0.3" | "4.0.4" | …`) are necessarily hand-written — the
    // last unreached row of the editions single-source sweep. This makes the
    // hand-list safe: bundling a new edition reddens every stale doc line, by
    // file and count, instead of shipping TSDoc advertising the old set.
    const expected = EDITIONS.map((e) => `"${e}"`).join(" | ");
    for (const file of ["index.ts", "registry.ts"]) {
      const src = readFileSync(
        new URL(`../ts/${file}`, import.meta.url),
        "utf8",
      );
      const lists = src.match(/"4\.\d[^`\n]*?"(?:\s*\|\s*"[\d.]+")+/g) ?? [];
      expect(
        lists.length,
        `${file}: the full-list TSDoc spellings should still exist`,
      ).toBeGreaterThan(0);
      for (const found of lists) {
        expect(found, `${file}: stale TSDoc edition list`).toBe(expected);
      }
      // The abbreviated ranges (`` `"4.0.3"`…`"4.2"` ``) pin only their
      // endpoints; the ellipsis may wrap across a comment line.
      const first = EDITIONS[0];
      const last = EDITIONS[EDITIONS.length - 1];
      const range = /`"([\d.]+)"`…\s*(?:\n\s*\*\s*)?`"([\d.]+)"`/g;
      for (const [, lo, hi] of src.matchAll(range)) {
        expect(lo, `${file}: range start`).toBe(first);
        expect(hi, `${file}: range end`).toBe(last);
      }
      // And a documented numeric default is the dictionary's own fallback —
      // the doc twin of the hard-coded `unwrap_or(V4_1_1)` class #923 retired
      // from the code.
      for (const [, dflt] of src.matchAll(/\(default `"([\d.]+)"`\)/g)) {
        expect(dflt, `${file}: stale documented default`).toBe(
          DICT.fallback_edition,
        );
      }
    }
  });
});
