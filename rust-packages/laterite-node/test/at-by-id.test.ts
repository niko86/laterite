// `at()` never filters silently-nothing, and can key on row identity — the Node
// port of laterite-py's #1093 change. `at(group, values)` filters `{group}_ID`;
// a file in which no group carries that heading used to come back whole, so it
// throws. `{ by: "_id" }` keys on the synthetic identity instead: the group's own
// rows by `_id`, its direct children by `_parent_id`.
import { afterEach, describe, expect, it, vi } from "vitest";
import { Ags4Error, type Ags4File, read } from "../ts/index";

vi.setConfig({ testTimeout: 60_000 });

const q = (...cells: string[]) => cells.map((c) => `"${c}"`).join(",");

// SAMP_ID left blank (it is optional); LLPL, a DICT-declared XSMP and a
// DICT-declared keyless XNKY hang off SAMP; GEOL carries no GEOL_ID and is a
// child of LOCA, not SAMP. The same file as laterite-py's test_at_by_id.py.
const AGS = [
  q("GROUP", "PROJ"),
  q("HEADING", "PROJ_ID"),
  q("UNIT", ""),
  q("TYPE", "ID"),
  q("DATA", "P1"),
  "",
  q("GROUP", "DICT"),
  q(
    "HEADING",
    "DICT_TYPE",
    "DICT_GRP",
    "DICT_HDNG",
    "DICT_STAT",
    "DICT_DTYP",
    "DICT_DESC",
    "DICT_UNIT",
    "DICT_PGRP",
  ),
  q("UNIT", "", "", "", "", "", "", "", ""),
  q("TYPE", "X", "X", "X", "X", "X", "X", "X", "X"),
  q("DATA", "GROUP", "XSMP", "", "", "", "Bespoke sample child", "", "SAMP"),
  q("DATA", "HEADING", "XSMP", "LOCA_ID", "KEY", "ID", "", "", ""),
  q("DATA", "HEADING", "XSMP", "SAMP_TOP", "KEY", "2DP", "", "m", ""),
  q("DATA", "HEADING", "XSMP", "SAMP_REF", "KEY", "X", "", "", ""),
  q("DATA", "HEADING", "XSMP", "SAMP_TYPE", "KEY", "PA", "", "", ""),
  q("DATA", "HEADING", "XSMP", "SAMP_ID", "KEY", "ID", "", "", ""),
  q("DATA", "HEADING", "XSMP", "XSMP_REF", "KEY", "X", "", "", ""),
  q("DATA", "GROUP", "XNKY", "", "", "", "Keyless sample child", "", "SAMP"),
  q("DATA", "HEADING", "XNKY", "XNKY_TXT", "OTHER", "X", "", "", ""),
  "",
  q("GROUP", "LOCA"),
  q("HEADING", "LOCA_ID"),
  q("UNIT", ""),
  q("TYPE", "ID"),
  q("DATA", "BH01"),
  q("DATA", "BH02"),
  "",
  q("GROUP", "GEOL"),
  q("HEADING", "LOCA_ID", "GEOL_TOP", "GEOL_BASE"),
  q("UNIT", "", "m", "m"),
  q("TYPE", "ID", "2DP", "2DP"),
  q("DATA", "BH01", "0.00", "1.00"),
  q("DATA", "BH02", "0.00", "2.00"),
  "",
  q("GROUP", "SAMP"),
  q("HEADING", "LOCA_ID", "SAMP_TOP", "SAMP_REF", "SAMP_TYPE", "SAMP_ID"),
  q("UNIT", "", "m", "", "", ""),
  q("TYPE", "ID", "2DP", "X", "PA", "ID"),
  q("DATA", "BH01", "1.00", "1", "U", ""),
  q("DATA", "BH01", "2.00", "2", "U", ""),
  q("DATA", "BH02", "1.00", "1", "U", ""),
  "",
  q("GROUP", "LLPL"),
  q(
    "HEADING",
    "LOCA_ID",
    "SAMP_TOP",
    "SAMP_REF",
    "SAMP_TYPE",
    "SAMP_ID",
    "SPEC_REF",
    "SPEC_DPTH",
    "LLPL_LL",
  ),
  q("UNIT", "", "m", "", "", "", "", "m", "%"),
  q("TYPE", "ID", "2DP", "X", "PA", "ID", "X", "2DP", "0DP"),
  q("DATA", "BH01", "1.00", "1", "U", "", "A", "1.00", "45"),
  q("DATA", "BH01", "1.00", "1", "U", "", "B", "1.10", "46"),
  q("DATA", "BH01", "2.00", "2", "U", "", "A", "2.00", "47"),
  q("DATA", "BH02", "1.00", "1", "U", "", "A", "1.00", "48"),
  "",
  q("GROUP", "XSMP"),
  q(
    "HEADING",
    "LOCA_ID",
    "SAMP_TOP",
    "SAMP_REF",
    "SAMP_TYPE",
    "SAMP_ID",
    "XSMP_REF",
  ),
  q("UNIT", "", "m", "", "", "", ""),
  q("TYPE", "ID", "2DP", "X", "PA", "ID", "X"),
  q("DATA", "BH01", "1.00", "1", "U", "", "x1"),
  q("DATA", "BH02", "1.00", "1", "U", "", "x2"),
  "",
  q("GROUP", "XNKY"),
  q("HEADING", "XNKY_TXT"),
  q("UNIT", ""),
  q("TYPE", "X"),
  q("DATA", "no identity"),
  "",
].join("\r\n");

async function idOf(
  ags: Ags4File,
  group: string,
  where: string,
): Promise<string> {
  const rows = await ags.sql(`SELECT _id FROM ${group} WHERE ${where}`);
  expect(rows).toHaveLength(1);
  return rows[0]!._id as string;
}

const sampId = (ags: Ags4File, loca: string, top: number) =>
  idOf(ags, "SAMP", `LOCA_ID = '${loca}' AND SAMP_TOP = ${top}`);

afterEach(() => {
  vi.restoreAllMocks();
});

describe("at() — the default by: {group}_ID", () => {
  it("throws on a heading no group carries", () => {
    using ags = read(undefined, { text: AGS });
    expect(() => ags.at("GEOL", ["anything"])).toThrow(Ags4Error);
    expect(() => ags.at("GEOL", ["anything"])).toThrow(/GEOL_ID/);
  });

  it("throws on a heading unknown to the dictionary", () => {
    using ags = read(undefined, { text: AGS });
    expect(() => ags.at("NOPE", ["x"])).toThrow(/NOPE_ID/);
  });

  it("throws from a chain too", () => {
    using ags = read(undefined, { text: AGS });
    expect(() => ags.at("LOCA", ["BH01"]).at("GEOL", ["x"])).toThrow(/GEOL_ID/);
  });

  it("still filters on a heading some group carries", async () => {
    using ags = read(undefined, { text: AGS });
    const sub = ags.at("SAMP", ["S1"]); // SAMP_ID present (blank): matches nothing
    expect(await sub.table("SAMP")).toEqual([]);
    expect(await sub.table("LLPL")).toEqual([]);
  });

  it("warns on a null value, toward by: _id", () => {
    using ags = read(undefined, { text: AGS });
    const warn = vi.spyOn(process, "emitWarning").mockImplementation(() => {});
    ags.at("SAMP", [null]);
    ags.at("LOCA", ["BH01"]).at("SAMP", [undefined]);
    expect(warn).toHaveBeenCalledTimes(2);
    expect(String(warn.mock.calls[0]![0])).toMatch(/by: "_id"/);
  });

  it("does not warn without a null value, or under by: _id", () => {
    using ags = read(undefined, { text: AGS });
    const warn = vi.spyOn(process, "emitWarning").mockImplementation(() => {});
    ags.at("SAMP", ["S1"]);
    ags.at("SAMP", [null], { by: "_id" });
    expect(warn).not.toHaveBeenCalled();
  });

  it("passes unreached groups through when only heading filters are present", async () => {
    using ags = read(undefined, { text: AGS });
    expect(await ags.at("LOCA", ["BH01"]).table("PROJ")).toHaveLength(1);
  });
});

describe("at(…, { by: '_id' }) — the group's rows, then its direct children", () => {
  it("keeps the rows and their direct children", async () => {
    using ags = read(undefined, { text: AGS });
    const sub = ags.at("SAMP", [await sampId(ags, "BH01", 1)], { by: "_id" });
    const samp = await sub.table("SAMP");
    expect(samp.map((r) => [r.LOCA_ID, r.SAMP_TOP])).toEqual([["BH01", 1]]);
    expect((await sub.table("LLPL")).map((r) => r.SPEC_REF)).toEqual([
      "A",
      "B",
    ]);
  });

  it("reaches a DICT-declared child", async () => {
    using ags = read(undefined, { text: AGS });
    const sub = ags.at("SAMP", [await sampId(ags, "BH01", 1)], { by: "_id" });
    expect((await sub.table("XSMP")).map((r) => r.XSMP_REF)).toEqual(["x1"]);
  });

  it("lists the group and its keyed direct children", async () => {
    using ags = read(undefined, { text: AGS });
    const sub = ags.at("SAMP", [await sampId(ags, "BH01", 1)], { by: "_id" });
    expect(sub.groups).toEqual(["SAMP", "LLPL", "XSMP"]);
    expect(Object.keys(await sub.frames())).toEqual(["SAMP", "LLPL", "XSMP"]);
  });

  it("gives a keyless child, asked for by name, no rows", async () => {
    using ags = read(undefined, { text: AGS });
    const sub = ags.at("SAMP", [await sampId(ags, "BH01", 1)], { by: "_id" });
    expect(await sub.table("XNKY")).toEqual([]);
  });

  it("throws on a group no filter reaches", async () => {
    // LOCA is SAMP's parent, GEOL its sibling: either handed back whole would
    // mix every borehole into "the rows at this sample".
    using ags = read(undefined, { text: AGS });
    const sub = ags.at("SAMP", [await sampId(ags, "BH01", 1)], { by: "_id" });
    await expect(sub.table("LOCA")).rejects.toThrow(/LOCA/);
    await expect(sub.table("GEOL")).rejects.toThrow(/GEOL/);
  });

  it("does not reach grandchildren through the parent", async () => {
    using ags = read(undefined, { text: AGS });
    const bh01 = await idOf(ags, "LOCA", "LOCA_ID = 'BH01'");
    const sub = ags.at("LOCA", [bh01], { by: "_id" });
    expect(sub.groups).toEqual(["LOCA", "GEOL", "SAMP"]);
    expect(await sub.table("SAMP")).toHaveLength(2);
    await expect(sub.table("LLPL")).rejects.toThrow(/LLPL/);
  });

  it("chains with a heading filter", async () => {
    using ags = read(undefined, { text: AGS });
    const ids = [await sampId(ags, "BH01", 1), await sampId(ags, "BH02", 1)];
    const sub = ags.at("SAMP", ids, { by: "_id" }).at("LOCA", ["BH02"]);
    expect((await sub.table("LLPL")).map((r) => Number(r.LLPL_LL))).toEqual([
      48,
    ]);
    expect((await sub.table("LOCA")).map((r) => r.LOCA_ID)).toEqual(["BH02"]);
  });

  it("with a heading filter, reaches what either reaches", async () => {
    using ags = read(undefined, { text: AGS });
    const sub = ags
      .at("LOCA", ["BH01"])
      .at("SAMP", [await sampId(ags, "BH01", 1)], { by: "_id" });
    expect(await sub.table("GEOL")).toHaveLength(1);
    await expect(sub.table("PROJ")).rejects.toThrow(/PROJ/);
  });

  it("matches nothing on an empty value list", async () => {
    using ags = read(undefined, { text: AGS });
    const sub = ags.at("SAMP", [], { by: "_id" });
    expect(await sub.table("SAMP")).toEqual([]);
    expect(await sub.table("LLPL")).toEqual([]);
  });

  it("throws when neither the group nor a direct child is in the file", () => {
    using ags = read(undefined, { text: AGS });
    expect(() => ags.at("NOPE", ["x"], { by: "_id" })).toThrow(/NOPE/);
  });

  it("throws on an unknown by", () => {
    using ags = read(undefined, { text: AGS });
    // @ts-expect-error — an unknown `by` is a type error AND a runtime throw
    expect(() => ags.at("SAMP", ["x"], { by: "SAMP_REF" })).toThrow(/by/);
  });
});
