// #1011 — DICT rows restating the standard dictionary: the advisory, and the
// `dictRows` option on `buildAgs4` and `merge`. The same engine as Python's
// `test_dict_redeclaration.py` and `lat merge --dict-rows`, so the same values.
import { describe, expect, it } from "vitest";
import { buildAgs4, merge, validate } from "../ts/index";

const FYI18 = "FYI (Related to Rule 18)";
const WARN18 = "Warning (Related to Rule 18)";

// One delivery: LOCA keyed on LOCA_ID, plus a DICT whose rows restate the
// standard LOCA_ID (as OTHER — dropping its KEY) and LOCA_GL, and declare one
// user heading the file really carries.
const delivery = (gl: string) =>
  [
    '"GROUP","PROJ"',
    '"HEADING","PROJ_ID"',
    '"UNIT",""',
    '"TYPE","ID"',
    '"DATA","P1"',
    "",
    '"GROUP","LOCA"',
    '"HEADING","LOCA_ID","LOCA_GL","LOCA_XTRA"',
    '"UNIT","","m",""',
    '"TYPE","ID","2DP","X"',
    `"DATA","BH1","${gl}","x"`,
    "",
    '"GROUP","DICT"',
    '"HEADING","DICT_TYPE","DICT_GRP","DICT_HDNG","DICT_STAT","DICT_DTYP"',
    '"UNIT","","","","",""',
    '"TYPE","X","X","X","X","X"',
    '"DATA","HEADING","LOCA","LOCA_ID","OTHER","ID"',
    '"DATA","HEADING","LOCA","LOCA_GL","OTHER","2DP"',
    '"DATA","HEADING","LOCA","LOCA_XTRA","OTHER","X"',
    "",
  ].join("\r\n");

const dictRows = (text: string) => {
  const lines = text.split("\r\n");
  const start = lines.indexOf('"GROUP","DICT"');
  if (start < 0) return [];
  const out: string[] = [];
  for (const line of lines.slice(start + 1)) {
    if (line === "") break;
    if (line.startsWith('"DATA"')) out.push(line);
  }
  return out;
};

const reported = (bytes: Uint8Array) =>
  new Set(
    validate(bytes, { fyi: true }).findings.map((f) => `${f.rule}|${f.desc}`),
  );

describe("the redeclaration advisory", () => {
  it("names a restated heading as an FYI and a dropped KEY as a warning", () => {
    const r = validate(Buffer.from(delivery("10.00")), { fyi: true });
    const fyi = r.findings.filter((f) => f.rule === FYI18);
    const warn = r.findings.filter((f) => f.rule === WARN18);
    expect(fyi.map((f) => f.desc)).toEqual([
      "DICT declares LOCA.LOCA_GL, a standard heading in 4.1.1; the standard definition applies.",
    ]);
    expect(warn).toHaveLength(1);
    expect(warn[0]?.desc).toContain("LOCA.LOCA_ID");
    expect(warn[0]?.desc).toContain("drops KEY");
    expect(r.isValid).toBe(validate(Buffer.from(delivery("10.00"))).isValid);
  });
});

describe("dictRows", () => {
  const a = Buffer.from(delivery("10.00"));
  const b = Buffer.from(delivery("11.50"));

  it("merge prune keeps only the user-defined row and adds no finding", () => {
    const keep = merge([a, b], { onMissingTran: "reconcile" });
    const prune = merge([a, b], { dictRows: "prune" });
    expect(dictRows(keep.text)).toHaveLength(3);
    expect(dictRows(prune.text)).toEqual([
      '"DATA","HEADING","LOCA","LOCA_XTRA","OTHER","X"',
    ]);
    const before = reported(keep.bytes);
    for (const f of reported(prune.bytes)) expect(before.has(f)).toBe(true);
    expect(merge([a, b], { dictRows: "keep" }).text).toBe(keep.text);
  });

  it("buildAgs4 prune omits a DICT left with no row", () => {
    const groups = new Map<string, Array<Record<string, unknown>>>([
      ["PROJ", [{ PROJ_ID: "P1" }]],
      [
        "DICT",
        [
          {
            DICT_TYPE: "HEADING",
            DICT_GRP: "PROJ",
            DICT_HDNG: "PROJ_ID",
            DICT_STAT: "KEY",
          },
        ],
      ],
    ]);
    const keep = buildAgs4(groups, { mode: "report" });
    const prune = buildAgs4(groups, { mode: "report", dictRows: "prune" });
    expect(keep.text).toContain('"GROUP","DICT"');
    expect(prune.text).not.toContain('"GROUP","DICT"');
    expect(buildAgs4(groups, { mode: "report", dictRows: "keep" }).text).toBe(
      keep.text,
    );
  });

  it("refuses an unknown value", () => {
    expect(() =>
      merge([a, b], { dictRows: "trim" as unknown as "keep" }),
    ).toThrow(/unknown dict_rows "trim"/);
  });
});
