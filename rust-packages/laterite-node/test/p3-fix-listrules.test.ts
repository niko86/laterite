import { mkdtempSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { afterAll, describe, expect, it } from "vitest";
import { type FixableRule, FixResult, fix, listRules } from "../ts/index";

// LOCA_GL is typed 2DP but "12.3" carries one decimal — a SAFE mechanical fix
// (numeric reformat) → "12.30". CRLF line endings (AGS4 canonical).
const FIXABLE = [
  '"GROUP","PROJ"',
  '"HEADING","PROJ_ID"',
  '"UNIT",""',
  '"TYPE","ID"',
  '"DATA","P1"',
  '"GROUP","LOCA"',
  '"HEADING","LOCA_ID","LOCA_GL"',
  '"UNIT","","m"',
  '"TYPE","ID","2DP"',
  '"DATA","BH01","12.3"',
  "",
].join("\r\n");

describe("fix() — mechanical repair (the Node mirror of laterite.fix)", () => {
  it("repairs a fixable file and returns a FixResult", () => {
    const r: FixResult = fix(undefined, { text: FIXABLE });
    expect(r).toBeInstanceOf(FixResult);
    expect(r.fixesApplied).toBeGreaterThan(0);
    expect(r.text).toMatch(/"DATA","BH01","12\.30"/); // 12.3 → padded 2DP
    expect(Array.isArray(r.applied)).toBe(true);
    expect(r.applied[0]).toHaveProperty("kind");
    expect(r.applied[0]).toHaveProperty("risk");
    expect(Array.isArray(r.findings)).toBe(true); // residual (what couldn't be fixed)
    expect(typeof r.dictVersion).toBe("string");
  });

  it("accepts Uint8Array bytes (V8 string-cap-free door)", () => {
    const r = fix(Buffer.from(FIXABLE, "utf8"));
    expect(r.fixesApplied).toBeGreaterThan(0);
    expect(Buffer.isBuffer(r.bytes)).toBe(true);
  });

  it("residual re-validation reports at the errors+warnings tier (#294 Batch C)", () => {
    // Unrecognised TRAN_AGS "9.9" -> the O-44 Rule 14 WARNING; bare-LF endings ->
    // a Rule 2a fix so the fixer runs. The warning must survive into the residual
    // — proving errors+warnings, not the old errors-only Node default (which
    // dropped it, drifting from Python/CLI).
    const src =
      '"GROUP","PROJ"\n"HEADING","PROJ_ID"\n"UNIT",""\n"TYPE","ID"\n"DATA","P1"\n' +
      '"GROUP","TRAN"\n"HEADING","TRAN_AGS"\n"UNIT",""\n"TYPE","X"\n"DATA","9.9"\n';
    const r = fix(undefined, { text: src });
    expect(r.fixesApplied).toBeGreaterThan(0);
    const severities = new Set(r.findings.map((f) => f.severity));
    expect(severities.has("warning")).toBe(true);
    expect(r.findings.some((f) => f.rule === "Warning (Related to Rule 14)")).toBe(true);
  });
});

// #394 — per-rule selection (only/exclude) + inPlace/out write-back, the Node
// mirror of laterite-py's free fix(). A file with TWO fixable defects: Rule 2a
// (bare-LF endings) + Rule 8 (a 2DP value at 1 dp).
const TWO_DEFECTS =
  '"GROUP","LOCA"\n"HEADING","LOCA_ID","LOCA_GL"\n"UNIT","","m"\n' +
  '"TYPE","ID","2DP"\n"DATA","BH1","1.0"\n';

describe("fix() — per-rule selection (only/exclude)", () => {
  it("only:['8'] applies just the precision fix and leaves the line endings", () => {
    const r = fix(undefined, { text: TWO_DEFECTS, only: ["8"] });
    // Only the numeric reformat ran (Rule 2a's CRLF fix was NOT selected).
    expect(r.applied.map((a) => a.kind)).toEqual(["reformat_numeric"]);
    expect(r.text).toMatch(/"BH1","1\.00"/); // 1.0 → padded 2DP
  });

  it("exclude:['8'] leaves the precision value unfixed", () => {
    const r = fix(undefined, { text: TWO_DEFECTS, exclude: ["8"] });
    expect(r.applied.some((a) => a.kind === "reformat_numeric")).toBe(false);
    expect(r.text).toMatch(/"BH1","1\.0"/); // untouched
  });

  it("rejects a non-fixable rule label (mirrors laterite-py _validate_fixable)", () => {
    // The static FixableRule type already blocks these at compile time; the casts
    // deliberately bypass it to prove the RUNTIME guard also rejects a bad label
    // (e.g. a value that slipped in untyped from JSON / a JS caller).
    const bad = (labels: string[]) => labels as unknown as FixableRule[];
    expect(() => fix(undefined, { text: TWO_DEFECTS, only: bad(["99"]) })).toThrow(/not fixable/);
    expect(() => fix(undefined, { text: TWO_DEFECTS, exclude: bad(["nope"]) })).toThrow(/not fixable/);
  });
});

describe("fix() — inPlace / out write-back", () => {
  const dir = mkdtempSync(join(tmpdir(), "laterite-fix-"));
  afterAll(() => {
    try {
      rmSync(dir, { recursive: true, force: true });
    } catch {
      /* ignore */
    }
  });

  it("inPlace overwrites the source file with the repaired bytes", () => {
    const p = join(dir, "inplace.ags");
    writeFileSync(p, TWO_DEFECTS);
    const r = fix(p, { inPlace: true });
    expect(readFileSync(p, "utf8")).toBe(r.text); // disk == repaired
    expect(readFileSync(p, "utf8")).toMatch(/"BH1","1\.00"/);
  });

  it("out writes the repaired bytes elsewhere, leaving the source untouched", () => {
    const src = join(dir, "src.ags");
    const dst = join(dir, "out.ags");
    writeFileSync(src, TWO_DEFECTS);
    const r = fix(src, { out: dst });
    expect(readFileSync(src, "utf8")).toBe(TWO_DEFECTS); // source untouched
    expect(readFileSync(dst, "utf8")).toBe(r.text); // written to dst
  });

  it("inPlace and out are mutually exclusive", () => {
    expect(() => fix("x.ags", { inPlace: true, out: "y.ags" })).toThrow(/mutually exclusive/);
  });

  it("inPlace needs a path source (bytes/text have nothing to overwrite)", () => {
    expect(() => fix(undefined, { text: TWO_DEFECTS, inPlace: true })).toThrow(/path source/);
  });
});

describe("listRules() — the gated rule catalogue (mirror of laterite.list_rules)", () => {
  it("returns one typed entry per AGS4 rule", () => {
    const rules = listRules();
    expect(rules.length).toBeGreaterThan(20);
    const r1 = rules.find((r) => r.rule === "1");
    expect(r1).toBeDefined();
    expect(typeof r1?.title).toBe("string");
    expect(typeof r1?.severity).toBe("string");
    expect(typeof r1?.fixable).toBe("boolean");
    expect(Array.isArray(r1?.observations)).toBe(true);
    // no phantom rules (the web catalogue's old 12 / 16a)
    expect(rules.some((r) => r.rule === "12" || r.rule === "16a")).toBe(false);
  });
});
