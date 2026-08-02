// `_filteredRows` — the SQL that backs `AgsSubset.table()`.
//
// Two of its decisions are invisible from the outside until they are wrong:
//
//   * the WHERE clause collapses to `TRUE` when no filter applies to the group,
//     so a `.at()` filter naming a key the group does not carry must return
//     every row rather than none. Getting that backwards silently empties a
//     table, which reads as "no data" rather than as a bug;
//   * the SELECT strips the synthetic `_id`/`_parent_id` columns from the FRAME
//     surface while the engine table keeps them for joins — and a
//     custom/passthrough group has no such columns at all, so it takes the plain
//     `*` arm. `EXCLUDE` on a column that isn't there is a SQL error, so the two
//     arms are not interchangeable.
import { describe, expect, it, vi } from "vitest";

import { read } from "../ts/index";

vi.setConfig({ testTimeout: 60_000 });

/** LOCA + SAMP (dictionary groups, so both get `_id`), plus XTRA — a bespoke
 *  group the dictionary has never heard of, which therefore carries no keys. */
const AGS =
  '"GROUP","LOCA"\r\n' +
  '"HEADING","LOCA_ID","LOCA_GL"\r\n' +
  '"UNIT","","m"\r\n' +
  '"TYPE","ID","2DP"\r\n' +
  '"DATA","BH01","12.30"\r\n' +
  '"DATA","BH02","14.00"\r\n' +
  '"GROUP","SAMP"\r\n' +
  '"HEADING","LOCA_ID","SAMP_ID","SAMP_TOP"\r\n' +
  '"UNIT","","","m"\r\n' +
  '"TYPE","ID","ID","2DP"\r\n' +
  '"DATA","BH01","S1","1.50"\r\n' +
  '"DATA","BH02","S2","2.00"\r\n' +
  '"GROUP","XTRA"\r\n' +
  '"HEADING","XTRA_NO","XTRA_VAL"\r\n' +
  '"UNIT","",""\r\n' +
  '"TYPE","ID","2DP"\r\n' +
  '"DATA","X1","9.99"\r\n';

describe("the filtered-frame WHERE clause", () => {
  it("filters a group that carries the key", async () => {
    using ags = read(undefined, { text: AGS });
    const rows = (await ags.at("LOCA", ["BH01"]).table("SAMP")) as Record<
      string,
      unknown
    >[];
    expect(rows.map((r) => r.SAMP_ID)).toEqual(["S1"]);
  });

  it("passes every row of a group the filter key does not apply to", async () => {
    // XTRA has no LOCA_ID, so the LOCA filter cannot narrow it and the clause
    // list stays empty → `WHERE TRUE`. The alternative — treating "no
    // applicable filter" as "nothing matches" — would make a custom group
    // vanish from an otherwise ordinary subset.
    using ags = read(undefined, { text: AGS });
    const rows = (await ags.at("LOCA", ["BH01"]).table("XTRA")) as Record<
      string,
      unknown
    >[];
    expect(rows).toHaveLength(1);
    expect(rows[0]!.XTRA_NO).toBe("X1");
  });

  it("matches nothing for an explicitly empty selection", async () => {
    // The other end of the same decision: an empty value list is a real
    // selection meaning "none of them", not an absent filter.
    using ags = read(undefined, { text: AGS });
    const rows = (await ags.at("LOCA", []).table("SAMP")) as Record<
      string,
      unknown
    >[];
    expect(rows).toEqual([]);
  });
});

describe("the synthetic key columns", () => {
  it("keeps _id out of a dictionary group's frame", async () => {
    // The engine table carries `_id`/`_parent_id` so cross-group joins resolve;
    // the frame surface is AGS data and must not leak them to a caller
    // iterating columns.
    using ags = read(undefined, { text: AGS });
    const rows = (await ags.at("LOCA", ["BH01"]).table("SAMP")) as Record<
      string,
      unknown
    >[];
    const cols = Object.keys(rows[0]!);
    expect(cols).not.toContain("_id");
    expect(cols).not.toContain("_parent_id");
    expect(cols).toContain("SAMP_ID");
  });

  it("selects a custom group with a plain star, since it has no keys to exclude", async () => {
    // A passthrough group never gets `_id`, and `EXCLUDE (_id, _parent_id)` on
    // a table without them is a SQL error — so this arm is not cosmetic.
    using ags = read(undefined, { text: AGS });
    const rows = (await ags.at("LOCA", ["BH01"]).table("XTRA")) as Record<
      string,
      unknown
    >[];
    expect(Object.keys(rows[0]!).sort()).toEqual(["XTRA_NO", "XTRA_VAL"]);
  });

  it("still joins on the keys it hid from the frame", async () => {
    // Proving the columns are hidden from the FRAME, not dropped from the
    // engine — otherwise the exclusion would have broken every cross-group join.
    using ags = read(undefined, { text: AGS });
    const rows = await ags.sql(
      "SELECT s.SAMP_ID FROM SAMP s JOIN LOCA l ON s._parent_id = l._id ORDER BY s.SAMP_ID",
    );
    expect(rows.map((r) => r.SAMP_ID)).toEqual(["S1", "S2"]);
  });
});
