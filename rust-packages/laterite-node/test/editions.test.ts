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
});
