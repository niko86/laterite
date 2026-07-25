import { describe, expect, it } from "vitest";
import type { Table } from "apache-arrow";
import { buildAgs4, read } from "../ts/index";

// Content-addressed `_id` / `_parent_id` keys in the Node surface (#303 Phase 4).
// The relational `sql()`/`at()` layer always carries them (so cross-group joins
// work); the `table()` accessor strips them by default; emit never writes them.
//
// SAME fixture + golden UUIDv8s as the Python `test_content_keys.py` — the ids
// come from the one shared Rust keychain, so matching goldens here IS a
// cross-surface parity proof (Node == Python == the DuckDB extension).
const AGS =
  '"GROUP","PROJ"\r\n"HEADING","PROJ_ID"\r\n"UNIT",""\r\n"TYPE","ID"\r\n"DATA","P1"\r\n' +
  '"GROUP","LOCA"\r\n"HEADING","LOCA_ID","PROJ_ID"\r\n' +
  '"UNIT","",""\r\n"TYPE","ID","ID"\r\n"DATA","BH1","P1"\r\n';
const PROJ_ID = "ac30a95d-e0ca-85f9-83c8-37a64af2762b";
const LOCA_ID = "a7025a6f-d9b8-83b6-8fad-81c0c744edbc";

const names = (t: Table): string[] => t.schema.fields.map((f) => f.name);

describe("content-addressed _id/_parent_id (#303, Node)", () => {
  it("table() strips the key columns by default", () => {
    const ags = read(undefined, { text: AGS });
    expect(names(ags.table("PROJ"))).not.toContain("_id");
    expect(names(ags.table("PROJ"))).not.toContain("_parent_id");
  });

  it("table(code, { keys: true }) adds exactly the two key columns", () => {
    const ags = read(undefined, { text: AGS });
    const plain = new Set(names(ags.table("PROJ")));
    const added = names(ags.table("PROJ", { keys: true })).filter(
      (n) => !plain.has(n),
    );
    expect(added.sort()).toEqual(["_id", "_parent_id"]);
  });

  it("golden UUIDv8 values match Python + the extension; child links to parent", () => {
    const ags = read(undefined, { text: AGS });
    const proj = ags.table("PROJ", { keys: true });
    const loca = ags.table("LOCA", { keys: true });
    expect(proj.getChild("_id")!.get(0)).toBe(PROJ_ID);
    expect(loca.getChild("_id")!.get(0)).toBe(LOCA_ID);
    expect(loca.getChild("_parent_id")!.get(0)).toBe(PROJ_ID); // child → parent
    expect(proj.getChild("_parent_id")!.get(0)).toBeNull(); // root → NULL
  });

  it("ids are deterministic across reads", () => {
    const id = (): unknown =>
      read(undefined, { text: AGS })
        .table("LOCA", { keys: true })
        .getChild("_id")!
        .get(0);
    expect(id()).toBe(id());
    expect(id()).toBe(LOCA_ID);
  });

  it("sql() exposes the keys — the cross-group join links child → parent", async () => {
    using ags = read(undefined, { text: AGS });
    const rows = await ags.sql(
      "SELECT l.LOCA_ID AS loca, p.PROJ_ID AS parent FROM LOCA l JOIN PROJ p ON l._parent_id = p._id",
    );
    expect(rows).toEqual([{ loca: "BH1", parent: "P1" }]);
  });

  it("at() frames strip the keys but the join behind them still resolved", async () => {
    using ags = read(undefined, { text: AGS });
    const rows = await ags.at("LOCA", ["BH1"]).table("LOCA");
    expect(Object.keys(rows[0]!)).not.toContain("_id");
    expect(Object.keys(rows[0]!)).not.toContain("_parent_id");
  });

  it("emit never leaks a synthetic key", () => {
    const ags = read(undefined, { text: AGS });
    expect(ags.text).not.toContain('"_id"'); // handle emit (retained parse)
    const out = buildAgs4(
      new Map([
        ["PROJ", ags.table("PROJ", { keys: true })],
        ["LOCA", ags.table("LOCA", { keys: true })],
      ]),
    );
    expect(out.text).not.toContain('"_id"');
    expect(out.text).not.toContain('"_parent_id"');
  });
});

// Candidate #6 (T6): the DEFAULT table() builds a keys-less frame with the
// native keychain SKIPPED (not built-then-stripped) — the keychain is ~96% of
// the native build. The risk is the cache: a single cache would then hand that
// keyless table to the relational layer and break joins. These pin that the
// two-cache split keeps sql()/at() correct even when a plain table() primed the
// group first, and that the default frame's CONTENTS are unchanged.
describe("keychain skipped on the default read (#6/T6, Node)", () => {
  it("sql() still joins after a prior plain table() on both groups", async () => {
    using ags = read(undefined, { text: AGS });
    // Prime the keys-less cache for both groups.
    expect(names(ags.table("PROJ"))).not.toContain("_id");
    expect(names(ags.table("LOCA"))).not.toContain("_parent_id");
    // The keyed join must still resolve child → parent — the keyed table is
    // built fresh for the engine, NOT served from the keys-less cache.
    const rows = await ags.sql(
      "SELECT l.LOCA_ID AS loca, p.PROJ_ID AS parent FROM LOCA l JOIN PROJ p ON l._parent_id = p._id",
    );
    expect(rows).toEqual([{ loca: "BH1", parent: "P1" }]);
  });

  it("at() still filters after a prior plain table() on the same group", async () => {
    using ags = read(undefined, { text: AGS });
    ags.table("LOCA"); // prime the keys-less cache
    const rows = await ags.at("LOCA", ["BH1"]).table("LOCA");
    expect(rows).toHaveLength(1);
    expect(rows[0]!).toMatchObject({ LOCA_ID: "BH1" });
  });

  it("keys:true after a prior plain table() still returns the keyed columns", () => {
    const ags = read(undefined, { text: AGS });
    expect(names(ags.table("LOCA"))).not.toContain("_id"); // keyless cache primed
    const keyed = ags.table("LOCA", { keys: true }); // keyed cache built fresh
    expect(keyed.getChild("_id")!.get(0)).toBe(LOCA_ID);
    expect(keyed.getChild("_parent_id")!.get(0)).toBe(PROJ_ID);
  });

  it("the default frame equals the keyed frame minus the two key columns", () => {
    // #6 changed HOW the default frame is built (native skip vs JS strip), not
    // WHAT it holds: same columns and values as table({keys:true}) less the keys.
    const ags = read(undefined, { text: AGS });
    const def = ags.table("LOCA");
    const keyed = ags.table("LOCA", { keys: true });
    const keyedNonKey = names(keyed).filter(
      (n) => n !== "_id" && n !== "_parent_id",
    );
    expect(names(def)).toEqual(keyedNonKey);
    for (const col of keyedNonKey) {
      expect(def.getChild(col)!.get(0)).toEqual(keyed.getChild(col)!.get(0));
    }
  });
});
