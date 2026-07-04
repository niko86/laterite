import { describe, expect, it } from "vitest";
import { FixResult, fix, listRules } from "../ts/index";

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
