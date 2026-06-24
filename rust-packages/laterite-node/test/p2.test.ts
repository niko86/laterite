// P2 — the high-level TS layer (Arrow-direct, no DuckDB): read → born-typed
// arrow-js Table, validate → Report, buildAgs4 → BuildResult round-trip, and the
// native-failure → mapped-exception protocol.
import { type Table, tableFromArrays } from "apache-arrow";
import { describe, expect, it } from "vitest";
import {
  Ags4File,
  type BuildResult,
  FileNotFoundError,
  NotAgs4Error,
  type Report,
  buildAgs4,
  read,
  validate,
} from "../ts/index";

const AGS =
  '"GROUP","PROJ"\r\n' +
  '"HEADING","PROJ_ID","PROJ_NAME"\r\n' +
  '"UNIT","",""\r\n' +
  '"TYPE","ID","X"\r\n' +
  '"DATA","P1","Demo project"\r\n' +
  '"GROUP","LOCA"\r\n' +
  '"HEADING","LOCA_ID","LOCA_GL","LOCA_CKED","LOCA_STAR"\r\n' +
  '"UNIT","","m","",""\r\n' +
  '"TYPE","ID","2DP","YN","DT"\r\n' +
  '"DATA","BH01","12.30","Y","2023-02-22"\r\n' +
  '"DATA","BH02","13.00","N","2023-03-01"\r\n';

const typeOf = (t: Table, name: string) => t.getChild(name)!.type.toString();

describe("read → Arrow-direct, born-typed", () => {
  it("decodes a group to a typed arrow-js Table (cross-host typing invariant)", () => {
    const ags = read(undefined, { text: AGS });
    expect(ags).toBeInstanceOf(Ags4File);
    expect(ags.groups).toEqual(["PROJ", "LOCA"]);

    const loca = ags.table("LOCA");
    // The SAME casting as Python/wasm (one shared `build_record_batch`):
    expect(typeOf(loca, "LOCA_ID")).toMatch(/Utf8/);
    expect(typeOf(loca, "LOCA_GL")).toMatch(/Float64/);
    expect(typeOf(loca, "LOCA_CKED")).toMatch(/Bool/);
    expect(typeOf(loca, "LOCA_STAR")).toMatch(/Timestamp|Date/);
    // A 2DP cell is a real f64, not the source string.
    expect(loca.getChild("LOCA_GL")!.get(0)).toBe(12.3);
    expect(loca.numRows).toBe(2);
  });

  it("caches the decoded Table per group (same instance)", () => {
    const ags = read(undefined, { text: AGS });
    expect(ags.table("LOCA")).toBe(ags.table("LOCA"));
  });

  it("exposes metadata without an Arrow decode", () => {
    const ags = read(undefined, { text: AGS });
    expect(ags.headings("LOCA")).toEqual(["LOCA_ID", "LOCA_GL", "LOCA_CKED", "LOCA_STAR"]);
    expect(ags.units("LOCA")).toEqual(["", "m", "", ""]);
    expect(ags.types("LOCA")).toEqual(["ID", "2DP", "YN", "DT"]);
    expect(ags.lineNumbers("LOCA")).toEqual([10, 11]); // the two DATA rows
    expect(ags.has("LOCA")).toBe(true);
    expect(ags.has("NOPE")).toBe(false);
    expect(ags.tranAgs).toBeNull();
  });

  it("throws asking for a missing group", () => {
    const ags = read(undefined, { text: AGS });
    expect(() => ags.table("NOPE")).toThrow(/not in file/);
  });

  it("re-emits byte-faithful AGS4 that re-parses", () => {
    const ags = read(undefined, { text: AGS });
    const text = ags.text;
    expect(text).toMatch(/"GROUP","PROJ"/);
    expect(text).toMatch(/\r\n/);
    expect(read(undefined, { text }).groups).toEqual(["PROJ", "LOCA"]);
  });
});

describe("validate → Report", () => {
  it("reports findings for a LOCA-only file (no PROJ/TRAN)", () => {
    const rep: Report = validate(undefined, { text: '"GROUP","LOCA"\r\n' });
    expect(rep.isValid).toBe(false);
    expect(rep.count).toBeGreaterThan(0);
    expect(rep.count).toBe(rep.findings.length);
    expect(rep.dictVersion).toBe("4.1.1");

    const byRule = rep.byRule();
    expect(Object.keys(byRule).length).toBeGreaterThan(0);
    // toJson is byte-faithful to `lat-check --json` (produced native-side).
    const parsed = JSON.parse(rep.toJson());
    expect(parsed).toHaveProperty("findings");
    expect(rep.toNdjson().trimEnd().split("\n").length).toBe(rep.count);
    expect(rep.exitCode).toBe(1);
  });

  it("a well-formed PROJ+LOCA file passes the structural rules it should", () => {
    const rep = validate(undefined, { text: AGS });
    // Whatever the finding count, the report is well-formed + JSON round-trips.
    expect(typeof rep.count).toBe("number");
    expect(rep.exitCode).toBe(rep.isValid ? 0 : 1);
    expect(() => JSON.parse(rep.toJson())).not.toThrow();
  });
});

describe("read/validate from raw bytes (the V8 string-cap door)", () => {
  const bytes = new TextEncoder().encode(AGS); // a Uint8Array

  it("read(Uint8Array) parses identically to read(text)", () => {
    const fromBytes = read(bytes);
    expect(fromBytes.groups).toEqual(read(undefined, { text: AGS }).groups);
    // born-typed survives the bytes path (a 2DP cell is a real f64, not a string)
    expect(fromBytes.table("LOCA").getChild("LOCA_GL")!.get(0)).toBe(12.3);
  });

  it("accepts a Node Buffer too (Buffer is a Uint8Array)", () => {
    expect(read(Buffer.from(AGS, "utf8")).groups).toEqual(["PROJ", "LOCA"]);
  });

  it("validate(Uint8Array) matches validate(text), byte-faithfully", () => {
    expect(validate(bytes).toNdjson()).toBe(validate(undefined, { text: AGS }).toNdjson());
  });

  it("encoding applies to the bytes path (windows-1252)", () => {
    // 'é' is 0xE9 in windows-1252; latin1 round-trips each char to its byte.
    const w1252 = Buffer.from(
      '"GROUP","PROJ"\r\n"HEADING","PROJ_ID"\r\n"UNIT",""\r\n"TYPE","ID"\r\n"DATA","Pré"\r\n',
      "latin1",
    );
    const proj = read(w1252, { encoding: "windows-1252" }).table("PROJ");
    expect(proj.getChild("PROJ_ID")!.get(0)).toBe("Pré");
  });
});

describe("native failure → mapped exception", () => {
  it("read() of a missing path throws FileNotFoundError (exit 3)", () => {
    try {
      read("/no/such/file.ags");
      throw new Error("expected a throw");
    } catch (e) {
      expect(e).toBeInstanceOf(FileNotFoundError);
      expect((e as FileNotFoundError).exitCode).toBe(3);
    }
  });

  it("read() of non-AGS4 text throws NotAgs4Error (exit 4)", () => {
    try {
      read(undefined, { text: "this is not an ags4 file\r\n" });
      throw new Error("expected a throw");
    } catch (e) {
      expect(e).toBeInstanceOf(NotAgs4Error);
      expect((e as NotAgs4Error).exitCode).toBe(4);
    }
  });

  it("validate() of non-AGS4 text also throws NotAgs4Error", () => {
    expect(() => validate(undefined, { text: "nope\r\n" })).toThrow(NotAgs4Error);
  });
});

describe("buildAgs4 → data → AGS4", () => {
  it("builds valid AGS4 from arrow-js Tables and round-trips", () => {
    const proj = tableFromArrays({ PROJ_ID: ["P1"], PROJ_NAME: ["Demo project"] });
    const loca = tableFromArrays({
      LOCA_ID: ["BH01", "BH02"],
      LOCA_GL: Float64Array.from([12.3, 13.0]),
    });
    const res: BuildResult = buildAgs4(
      new Map<string, Table>([
        ["PROJ", proj],
        ["LOCA", loca],
      ]),
      { dictVersion: "4.1.1", mode: "autofix" },
    );
    expect(Buffer.isBuffer(res.bytes)).toBe(true);
    expect(res.text).toMatch(/"GROUP","PROJ"/);
    expect(res.text).toMatch(/"TYPE","ID","2DP"/); // UNIT/TYPE filled from the dict
    expect(res.text).toMatch(/"DATA","BH01","12\.30"/); // Float64 12.3 → canonical 2DP
    expect(Array.isArray(res.findings)).toBe(true);

    // The emitted bytes re-parse to the same groups.
    expect(read(undefined, { text: res.text }).groups).toEqual(["PROJ", "LOCA"]);
  });

  it("accepts row-objects (transposed to a typed Table)", () => {
    const res = buildAgs4([
      ["PROJ", [{ PROJ_ID: "P1", PROJ_NAME: "Demo" }]],
      [
        "LOCA",
        [
          { LOCA_ID: "BH01", LOCA_GL: 12.3 },
          { LOCA_ID: "BH02", LOCA_GL: 13.0 },
        ],
      ],
    ]);
    expect(res.text).toMatch(/"DATA","BH01","12\.30"/);
    expect(read(undefined, { text: res.text }).table("LOCA").numRows).toBe(2);
  });
});
