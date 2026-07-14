// The modality OUTPUT gate — same bytes, three doors, one answer.
//
// Every cross-surface gate this repo owns compares knob NAMES. None of them compares
// an ANSWER. So a modality that accepts the same file and the same flags and then
// returns a *different verdict* sits in the blind spot of all of them — which is
// exactly where this bug was living:
//
//     TRAN_AGS says "4.0.3", but the file uses LOCA_NATD (a 4.0.4-only heading)
//
//     path  -> judged against 4.0.4, 3 findings   <- guard_4_0_4 (O-42) ran
//     bytes -> judged against 4.0.3, 5 findings   <- it didn't
//     text  -> judged against 4.0.3, 5 findings   <- it didn't
//
// Two phantom Rule 9 findings on every bytes read, because laterite-py, laterite-node
// and wasm each hand-assembled "resolve the edition, then run the rules" and each left
// the content guard out. The knob names matched perfectly.
import { mkdtempSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { describe, expect, it } from "vitest";
import { type Report, read, validate } from "../ts/index";

// TRAN_AGS declares 4.0.3; LOCA_NATD was introduced in 4.0.4. The O-42 content guard
// judges the file against 4.0.4 so its newer vocabulary isn't false-flagged as
// non-standard, and emits an FYI saying so.
const MISLABELLED_4_0_3 = [
  '"GROUP","PROJ"',
  '"HEADING","PROJ_ID"',
  '"UNIT",""',
  '"TYPE","ID"',
  '"DATA","P1"',
  "",
  '"GROUP","TRAN"',
  '"HEADING","TRAN_ISNO","TRAN_DATE","TRAN_PROD","TRAN_STAT","TRAN_AGS","TRAN_RECV","TRAN_DLIM","TRAN_RCON"',
  '"UNIT","","yyyy-mm-dd","","","","","",""',
  '"TYPE","X","DT","X","X","X","X","X","X"',
  '"DATA","1","2020-08-18","ACME Drilling Ltd","Draft","4.0.3","ACME Consulting","|","+"',
  "",
  '"GROUP","LOCA"',
  '"HEADING","LOCA_ID","LOCA_NATD"',
  '"UNIT","",""',
  '"TYPE","ID","X"',
  '"DATA","BH1","x"',
  "",
].join("\r\n");

// Everything a caller can observe about a verdict EXCEPT the source label (`<bytes>`
// vs the path — the one field that is *supposed* to differ).
function answer(rep: Report) {
  const findings = Object.entries(rep.byRule())
    .flatMap(([rule, items]) =>
      items.map((f) => `${rule}|${f.line ?? ""}|${f.group}|${f.desc}|${f.severity ?? ""}`),
    )
    .sort();
  return {
    dictVersion: rep.dictVersion,
    resolution: rep.resolution,
    count: rep.count,
    isValid: rep.isValid,
    findings,
  };
}

function writeAgs(text: string): string {
  const dir = mkdtempSync(join(tmpdir(), "lat-modality-"));
  const src = join(dir, "mislabelled.ags");
  writeFileSync(src, Buffer.from(text, "utf8"));
  return src;
}

describe.each([
  { tiers: "errors-only", opts: { warnings: false, fyi: false } },
  { tiers: "warnings", opts: { warnings: true, fyi: false } },
  { tiers: "warnings+fyi", opts: { warnings: true, fyi: true } },
])("$tiers", ({ opts }) => {
  it("path, text and bytes return the same verdict", () => {
    const src = writeAgs(MISLABELLED_4_0_3);

    const fromPath = validate(src, opts);
    const fromText = validate(undefined, { ...opts, text: MISLABELLED_4_0_3 });
    const fromBytes = validate(Buffer.from(MISLABELLED_4_0_3, "utf8"), opts);

    expect(answer(fromText)).toEqual(answer(fromPath));
    expect(answer(fromBytes)).toEqual(answer(fromPath));
  });
});

describe("the O-42 4.0.3 guard", () => {
  it("reaches every modality", () => {
    // Asserted by value rather than by agreement, so the gate still bites if all
    // three modalities were to break the same way.
    const src = writeAgs(MISLABELLED_4_0_3);
    const runs: Array<[string, Report]> = [
      ["path", validate(src, { fyi: true })],
      ["text", validate(undefined, { text: MISLABELLED_4_0_3, fyi: true })],
      ["bytes", validate(Buffer.from(MISLABELLED_4_0_3, "utf8"), { fyi: true })],
    ];
    for (const [label, rep] of runs) {
      expect(rep.dictVersion, `${label}: must be judged against 4.0.4`).toBe("4.0.4");
      expect(rep.resolution, `${label}`).toBe("guessed");
      const hasFyi = Object.values(rep.byRule())
        .flat()
        .some((f) => f.severity === "fyi" && (f.desc ?? "").includes("4.0.4"));
      expect(hasFyi, `${label}: the transparency FYI (#222 / O-42) is missing`).toBe(true);
    }
  });
});

describe("the chained handle agrees with the free function", () => {
  it("read(bytes).validate() lands on the same edition as read(path).validate()", () => {
    const src = writeAgs(MISLABELLED_4_0_3);
    const viaPath = read(src).validate({ fyi: true }).report;
    const viaBytes = read(Buffer.from(MISLABELLED_4_0_3, "utf8")).validate({ fyi: true }).report;
    if (!viaPath || !viaBytes) throw new Error("validate() did not produce a report");
    expect(answer(viaBytes)).toEqual(answer(viaPath));
  });
});
