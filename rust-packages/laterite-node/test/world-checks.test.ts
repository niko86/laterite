// `checkFiles` — the one check that reads state the AGS4 bytes do not contain.
//
// Rule 20 has two halves. The CONTENT half ("every FILE_FSET used is defined in the
// FILE group") is a pure function of the file. The WORLD half ("FILE/<fset>/<name>
// exists beside the .ags") stats the filesystem — someone can delete that tree
// without touching a byte of the delivery, and the verdict flips.
//
// So the WORLD half needs a path. Ask for it against a Buffer or a string and there
// is no directory to look in — the question cannot be answered. The engine used to
// answer it anyway: it dropped the request and reported Rule 20 clean. A false clean,
// with no certificate involved at all. These tests pin the fix by its OUTPUT: the
// same call that returned a clean report now throws.
import { mkdirSync, mkdtempSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { describe, expect, it } from "vitest";
import type { Report } from "../ts/report";
import { WorldCheckRequiresSourceError, read, validate } from "../ts/index";

const RULE_20 = "AGS Format Rule 20";

// Content-clean (PROJ + TRAN + UNIT + TYPE), plus a FILE group declaring one
// attachment. Rule 20's CONTENT half is satisfied — FS1 *is* defined in FILE — so the
// only thing left to say about Rule 20 is whether FILE/FS1/photo.jpg is really there.
const WITH_ATTACHMENT = [
  '"GROUP","PROJ"',
  '"HEADING","PROJ_ID","PROJ_NAME"',
  '"UNIT","",""',
  '"TYPE","ID","X"',
  '"DATA","P1","Clean minimal AGS4 fixture"',
  "",
  '"GROUP","TRAN"',
  '"HEADING","TRAN_ISNO","TRAN_DATE","TRAN_PROD","TRAN_STAT","TRAN_AGS","TRAN_RECV","TRAN_DLIM","TRAN_RCON"',
  '"UNIT","","yyyy-mm-dd","","","","","",""',
  '"TYPE","X","DT","X","X","X","X","X","X"',
  '"DATA","1","2020-08-18","ACME Drilling Ltd","Draft","4.2","ACME Consulting","|","+"',
  "",
  '"GROUP","UNIT"',
  '"HEADING","UNIT_UNIT","UNIT_DESC"',
  '"UNIT","",""',
  '"TYPE","X","X"',
  '"DATA","yyyy-mm-dd","year month day"',
  "",
  '"GROUP","TYPE"',
  '"HEADING","TYPE_TYPE","TYPE_DESC"',
  '"UNIT","",""',
  '"TYPE","X","X"',
  '"DATA","ID","Unique identifier"',
  '"DATA","X","Text"',
  '"DATA","DT","Date and time"',
  "",
  '"GROUP","FILE"',
  '"HEADING","FILE_FSET","FILE_NAME"',
  '"UNIT","",""',
  '"TYPE","X","X"',
  '"DATA","FS1","photo.jpg"',
  "",
].join("\r\n");

const DATA = Buffer.from(WITH_ATTACHMENT, "utf8");

function writeAgs(): { dir: string; src: string } {
  const dir = mkdtempSync(join(tmpdir(), "lat-world-"));
  const src = join(dir, "delivery.ags");
  writeFileSync(src, DATA);
  return { dir, src };
}

// `.report` is `Report | undefined` until `.validate()` has run; every call site here
// has, so assert it rather than littering the assertions with `?.`.
function reportOf(f: { report: Report | undefined }): Report {
  const r = f.report;
  if (r === undefined) throw new Error("validate() did not produce a report");
  return r;
}

const rulesOf = (report: Report) => Object.keys(report.byRule());

describe("checkFiles without a source path", () => {
  it("refuses a bytes read instead of reporting Rule 20 clean", () => {
    // THE BUG. Before: a clean report, Rule 20 silently unasked.
    expect(() => read(DATA).validate({ checkFiles: true })).toThrow(WorldCheckRequiresSourceError);
    try {
      read(DATA).validate({ checkFiles: true });
    } catch (e) {
      expect((e as { exitCode: number }).exitCode).toBe(5);
      expect((e as Error).message).toContain("path");
    }
  });

  it("refuses a text read too", () => {
    // The text modality is the bytes modality's twin, and had the same hole. In Node
    // a bare string is a PATH, so text arrives via `validate(undefined, {text})`.
    expect(() => validate(undefined, { text: WITH_ATTACHMENT, checkFiles: true })).toThrow(
      WorldCheckRequiresSourceError,
    );
  });
});

describe("checkFiles with a source path", () => {
  it("actually runs the on-disk check — Rule 20 fires when the tree is missing", () => {
    // The refusal above is not the engine being unable to do the check. Hand it a
    // path with no FILE/ tree beside it and Rule 20 speaks. What changed is that
    // "I cannot answer" no longer looks exactly like "nothing is wrong".
    const { src } = writeAgs();
    const rep = reportOf(read(src).validate({ checkFiles: true }));
    expect(rulesOf(rep)).toContain(RULE_20);
    expect(rep.isValid).toBe(false);
  });

  it("is clean once the attachment really exists", () => {
    // Two different verdicts over byte-identical .ags content — which is precisely
    // why a certificate (keyed on a SHA-256 of those bytes) may never vouch for
    // this half of Rule 20.
    const { dir, src } = writeAgs();
    mkdirSync(join(dir, "FILE", "FS1"), { recursive: true });
    writeFileSync(join(dir, "FILE", "FS1", "photo.jpg"), "x");

    const rep = reportOf(read(src).validate({ checkFiles: true }));
    expect(rulesOf(rep)).not.toContain(RULE_20);
    expect(rep.isValid).toBe(true);
  });
});

describe("the everyday call is unaffected", () => {
  it("bytes without checkFiles stay content-only and clean", () => {
    // The fix refuses a request that was never answerable, not one that was.
    const rep = reportOf(read(DATA).validate());
    expect(rulesOf(rep)).not.toContain(RULE_20);
    expect(rep.isValid).toBe(true);
  });
});
