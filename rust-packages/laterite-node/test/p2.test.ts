// P2 — the high-level TS layer (Arrow-direct, no DuckDB): read → born-typed
// arrow-js Table, validate → Report, buildAgs4 → BuildResult round-trip, and the
// native-failure → mapped-exception protocol.
import {
  mkdirSync,
  mkdtempSync,
  readFileSync,
  readdirSync,
  rmSync,
} from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { type Table, tableFromArrays } from "apache-arrow";
import { describe, expect, it } from "vitest";
import {
  Ags4File,
  type BuildResult,
  BuildSaved,
  FileNotFoundError,
  type GroupData,
  NotAgs4Error,
  type Report,
  buildAgs4,
  buildAgs4Unchecked,
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
    expect(ags.headings("LOCA")).toEqual([
      "LOCA_ID",
      "LOCA_GL",
      "LOCA_CKED",
      "LOCA_STAR",
    ]);
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
    expect(validate(bytes).toNdjson()).toBe(
      validate(undefined, { text: AGS }).toNdjson(),
    );
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

  it("the hyphenated 'latin-1' label resolves to windows-1252", () => {
    // 'latin-1' is not a WHATWG label, so it used to fall through to a silent
    // UTF-8 fallback here (mojibaking 0xE9) — the cross-surface divergence the
    // shared resolver closes. It must now decode as windows-1252, same as the
    // 'windows-1252' / 'latin1' spellings.
    const w1252 = Buffer.from(
      '"GROUP","PROJ"\r\n"HEADING","PROJ_ID"\r\n"UNIT",""\r\n"TYPE","ID"\r\n"DATA","Pré"\r\n',
      "latin1",
    );
    const proj = read(w1252, { encoding: "latin-1" }).table("PROJ");
    expect(proj.getChild("PROJ_ID")!.get(0)).toBe("Pré");
  });
});

describe("native failure → mapped exception", () => {
  it("read() of a missing path throws FileNotFoundError (exit 3)", () => {
    let caught: unknown;
    try {
      read("/no/such/file.ags");
    } catch (e) {
      caught = e;
    }
    expect(caught).toBeInstanceOf(FileNotFoundError);
    expect((caught as FileNotFoundError).exitCode).toBe(3);
  });

  it("read() of non-AGS4 text throws NotAgs4Error (exit 4)", () => {
    let caught: unknown;
    try {
      read(undefined, { text: "this is not an ags4 file\r\n" });
    } catch (e) {
      caught = e;
    }
    expect(caught).toBeInstanceOf(NotAgs4Error);
    expect((caught as NotAgs4Error).exitCode).toBe(4);
  });

  it("validate() of non-AGS4 text also throws NotAgs4Error", () => {
    expect(() => validate(undefined, { text: "nope\r\n" })).toThrow(
      NotAgs4Error,
    );
  });
});

describe("buildAgs4 → data → AGS4", () => {
  it("builds valid AGS4 from arrow-js Tables and round-trips", () => {
    const proj = tableFromArrays({
      PROJ_ID: ["P1"],
      PROJ_NAME: ["Demo project"],
    });
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

    // Exactly the data groups come back — metadata synthesis is opt-in, so
    // nothing appears that the caller did not supply.
    expect(read(undefined, { text: res.text }).groups).toEqual([
      "PROJ",
      "LOCA",
    ]);
  });

  it("out= writes the judged file and returns a BuildSaved with no bytes (#855)", () => {
    const dir = mkdtempSync(join(tmpdir(), "laterite-build-"));
    try {
      const proj = tableFromArrays({ PROJ_ID: ["P1"] });
      const loca = tableFromArrays({ LOCA_ID: ["BH1"], LOCA_GL: ["1.0"] });
      const groups = new Map<string, Table>([
        ["PROJ", proj],
        ["LOCA", loca],
      ]);
      const dest = join(dir, "built.ags");
      const saved: BuildSaved = buildAgs4(groups, { out: dest });
      expect(saved).toBeInstanceOf(BuildSaved);
      expect(saved.path).toBe(dest);
      expect("bytes" in saved).toBe(false);
      // The file on disk is byte-identical to the bytes-carrying door's
      // output for the same input — the rider changes where, never what.
      const plain = buildAgs4(groups);
      expect(readFileSync(dest).equals(plain.bytes)).toBe(true);
      expect(saved.findings).toEqual(plain.findings);
      expect(saved.fixesApplied).toBe(plain.fixesApplied);
      // No staging debris beside the destination.
      expect(readdirSync(dir)).toEqual(["built.ags"]);
      expect(String(saved)).toContain("BuildSaved");
      expect(String(saved)).toContain(dest);
    } finally {
      rmSync(dir, { recursive: true, force: true });
    }
  });

  it("out= under strict leaves the destination untouched on refusal", () => {
    const dir = mkdtempSync(join(tmpdir(), "laterite-build-"));
    try {
      // No PROJ/TRAN → error-severity findings → strict throws, and the
      // destination path must never hold the unjudged bytes.
      const loca = tableFromArrays({ LOCA_ID: ["BH1"] });
      const dest = join(dir, "refused.ags");
      expect(() =>
        buildAgs4(new Map<string, Table>([["LOCA", loca]]), {
          mode: "strict",
          out: dest,
        }),
      ).toThrow(/strict/);
      expect(readdirSync(dir)).toEqual([]);
    } finally {
      rmSync(dir, { recursive: true, force: true });
    }
  });

  it("out= failure cleans its staging file and surfaces the original error", () => {
    const dir = mkdtempSync(join(tmpdir(), "laterite-build-"));
    try {
      const proj = tableFromArrays({ PROJ_ID: ["P1"] });
      const loca = tableFromArrays({ LOCA_ID: ["BH1"], LOCA_GL: ["1.0"] });
      const groups = new Map<string, Table>([
        ["PROJ", proj],
        ["LOCA", loca],
      ]);
      // A destination that IS an existing directory fails at the rename, after
      // the staging write succeeded — the staging file must not survive it.
      const sub = join(dir, "sub");
      mkdirSync(sub);
      expect(() => buildAgs4(groups, { out: sub })).toThrow();
      expect(readdirSync(dir)).toEqual(["sub"]);
      // A destination whose parent does not exist fails at the staging write
      // itself; the best-effort unlink of a never-created staging file must
      // not mask the original error.
      expect(() =>
        buildAgs4(groups, { out: join(dir, "missing", "built.ags") }),
      ).toThrow(/ENOENT/);
    } finally {
      rmSync(dir, { recursive: true, force: true });
    }
  });

  it("BuildResult.applied is the safe-fix ledger (#294 F#7)", () => {
    // A string "1.0" under the 2DP LOCA_GL heading is a safe Rule 8 reformat
    // AutoFix applies during the build — so `applied` carries its record (same
    // {kind,label,rule,line,risk} shape FixResult.applied uses), and
    // fixesApplied is exactly its length.
    const proj = tableFromArrays({ PROJ_ID: ["P1"] });
    const loca = tableFromArrays({ LOCA_ID: ["BH1"], LOCA_GL: ["1.0"] });
    const groups = new Map<string, Table>([
      ["PROJ", proj],
      ["LOCA", loca],
    ]);
    const res = buildAgs4(groups, { dictVersion: "4.1.1", mode: "autofix" });
    expect(res.fixesApplied).toBe(res.applied.length);
    expect(res.applied.length).toBeGreaterThan(0);
    expect(res.applied[0]).toMatchObject({
      rule: "AGS Format Rule 8",
      risk: "safe",
    });
    expect(typeof res.applied[0]?.kind).toBe("string");
    // report mode touches nothing -> empty ledger.
    const rep = buildAgs4(groups, { mode: "report" });
    expect(rep.applied).toEqual([]);
    expect(rep.fixesApplied).toBe(0);
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

  it("units/types override per heading (#294 F#9)", () => {
    // LOCA_XTRA is a custom heading the dictionary doesn't know; the override
    // gives it a real UNIT/TYPE while LOCA_GL still fills from the dict.
    const groups: Array<[string, GroupData]> = [
      ["PROJ", [{ PROJ_ID: "P1" }]],
      ["LOCA", [{ LOCA_ID: "BH1", LOCA_GL: 1.0, LOCA_XTRA: "9" }]],
    ];
    const res = buildAgs4(groups, {
      mode: "report",
      units: { LOCA: { LOCA_XTRA: "kPa" } },
      types: { LOCA: { LOCA_XTRA: "3DP" } },
    });
    const loca = res.text.split('"GROUP","LOCA"')[1] ?? "";
    expect(loca).toContain('"UNIT","","m","kPa"'); // LOCA_GL from dict, LOCA_XTRA overridden
    expect(loca).toContain('"TYPE","ID","2DP","3DP"');
  });

  it("units/types reject an unknown group or heading (#294 F#9)", () => {
    const groups: Array<[string, GroupData]> = [["LOCA", [{ LOCA_ID: "BH1" }]]];
    expect(() => buildAgs4(groups, { units: { NOPE: { X: "m" } } })).toThrow(
      /unknown group/,
    );
    expect(() =>
      buildAgs4(groups, { types: { LOCA: { NOSUCH: "3DP" } } }),
    ).toThrow(/no heading/);
  });
});

describe("buildAgs4Unchecked → the no-verdict door (#881)", () => {
  const clean = (): Array<[string, GroupData]> => [
    ["PROJ", [{ PROJ_ID: "P1", PROJ_NAME: "Demo" }]],
    ["LOCA", [{ LOCA_ID: "BH01", LOCA_GL: 12.3 }]],
  ];
  // No PROJ, a non-canonical string under the 2DP dict fill — report objects.
  const dirty = (): Array<[string, GroupData]> => [
    ["LOCA", [{ LOCA_ID: "BH01", LOCA_GL: "12.3" }]],
  ];

  it("returns exactly the judged report build's bytes, clean and dirty", () => {
    const judged = buildAgs4(clean(), { mode: "report" });
    expect(buildAgs4Unchecked(clean()).equals(judged.bytes)).toBe(true);

    const judgedDirty = buildAgs4(dirty(), { mode: "report" });
    expect(judgedDirty.findings.length).toBeGreaterThan(0); // falsifiability
    expect(buildAgs4Unchecked(dirty()).equals(judgedDirty.bytes)).toBe(true);
  });

  it("returns a plain Buffer, deliberately not a BuildResult", () => {
    const raw = buildAgs4Unchecked(clean());
    expect(Buffer.isBuffer(raw)).toBe(true);
    expect("findings" in raw).toBe(false);
  });

  it("keeps the data-shaping knobs and refuses the judge-coupled ones", () => {
    const groups: Array<[string, GroupData]> = [
      ["PROJ", [{ PROJ_ID: "P1" }]],
      ["LOCA", [{ LOCA_ID: "BH1", LOCA_GL: 1.0, LOCA_XTRA: "9" }]],
    ];
    const raw = buildAgs4Unchecked(groups, {
      dictVersion: "4.2",
      units: { LOCA: { LOCA_XTRA: "kPa" } },
      types: { LOCA: { LOCA_XTRA: "3DP" } },
    });
    const loca = raw.toString("utf-8").split('"GROUP","LOCA"')[1] ?? "";
    expect(loca).toContain('"UNIT","","m","kPa"');
    expect(loca).toContain('"TYPE","ID","2DP","3DP"');
    expect(() =>
      buildAgs4Unchecked(groups, { units: { NOPE: { X: "m" } } }),
    ).toThrow(/buildAgs4Unchecked.*unknown group/);
    // Gone, not defaulted — a JS caller passing a judged-door knob is refused,
    // never silently ignored.
    expect(() =>
      buildAgs4Unchecked(groups, {
        mode: "report",
      } as unknown as Parameters<typeof buildAgs4Unchecked>[1]),
    ).toThrow(/mode/);
  });

  it("out= stages the write and returns the path — with no verdict gate", () => {
    const dir = mkdtempSync(join(tmpdir(), "laterite-unchecked-"));
    try {
      const dest = join(dir, "delivery.ags");
      const ret = buildAgs4Unchecked(clean(), { out: dest });
      expect(ret).toBe(dest);
      expect(readFileSync(dest).equals(buildAgs4Unchecked(clean()))).toBe(true);
      expect(readdirSync(dir)).toEqual(["delivery.ags"]); // no staging debris

      // THE difference from buildAgs4({ out }): nothing judges, so a file the
      // strict door refuses still lands — the caller chose unchecked.
      const refused = join(dir, "dirty.ags");
      expect(() =>
        buildAgs4(dirty(), { mode: "strict", out: refused }),
      ).toThrow();
      expect(readdirSync(dir)).toEqual(["delivery.ags"]);
      buildAgs4Unchecked(dirty(), { out: refused });
      expect(readdirSync(dir).sort()).toEqual(["delivery.ags", "dirty.ags"]);
    } finally {
      rmSync(dir, { recursive: true, force: true });
    }
  });
});
