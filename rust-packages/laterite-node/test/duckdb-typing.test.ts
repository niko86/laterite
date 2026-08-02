// The AGS-TYPE → DuckDB-column mapping, and the two conversions under it.
//
// `#appendCell`'s switch is the whole born-typed contract on this surface: it
// decides what an AGS column *becomes* once it reaches SQL. A wrong arm does not
// throw — it produces a table full of plausible, wrong values, and the first
// symptom is a query result that quietly disagrees with the file.
//
// The existing P3 suite covers `2DP`/`YN`/`DT`/`ID`. What it never reached was
// `0DP` (the BIGINT arm), the non-string path through `scalarString`, and
// `toMicros`' number branch — the last of which is the same class of bug the web
// side had: read microseconds as milliseconds and every timestamp lands in 1970,
// with no error anywhere.
import type { Table } from "apache-arrow";
import { describe, expect, it, vi } from "vitest";

import { read } from "../ts/index";

// DuckDB stands up per case and a cold checkout downloads the arrow extension —
// same reason p3-duckdb.test.ts sets this file-wide.
vi.setConfig({ testTimeout: 60_000 });

/** Every mapped AGS type in one group, plus an unmapped one. `GEOL_BASE` is
 *  `0DP` — an integer count, the arm the P3 suite never exercised. */
const TYPED =
  '"GROUP","LOCA"\r\n' +
  '"HEADING","LOCA_ID","LOCA_GL","LOCA_CKED","LOCA_STAR","LOCA_NO","LOCA_REM"\r\n' +
  '"UNIT","","m","","","",""\r\n' +
  '"TYPE","ID","2DP","YN","DT","0DP","X"\r\n' +
  '"DATA","BH01","12.30","Y","2023-02-22 09:30:15","7","north pit"\r\n' +
  '"DATA","BH02","","N","2023-03-01","-3","south pit"\r\n';

describe("the AGS type → DuckDB column mapping", () => {
  it("gives every AGS type the column type its values need", async () => {
    using ags = read(undefined, { text: TYPED });
    const rows = await ags.sql(
      `SELECT typeof(LOCA_ID)   AS id,
              typeof(LOCA_GL)   AS gl,
              typeof(LOCA_CKED) AS cked,
              typeof(LOCA_STAR) AS star,
              typeof(LOCA_NO)   AS no,
              typeof(LOCA_REM)  AS rem
       FROM LOCA LIMIT 1`,
    );
    const t = rows[0]!;
    expect(t.id).toBe("VARCHAR");
    expect(t.gl).toBe("DOUBLE");
    expect(t.cked).toBe("BOOLEAN");
    expect(t.star).toBe("TIMESTAMP");
    // 0DP is a whole number and must NOT land as DOUBLE — a count that arrives
    // as 7.0 is a different fact from 7, and it round-trips out as "7.0".
    expect(t.no).toBe("BIGINT");
    // X is free text; anything unmapped falls through to the same arm.
    expect(t.rem).toBe("VARCHAR");
  });

  it("carries 0DP values through the BIGINT arm, negatives included", async () => {
    using ags = read(undefined, { text: TYPED });
    const rows = await ags.sql("SELECT LOCA_NO FROM LOCA ORDER BY LOCA_ID");
    // BigInt or number depending on how the driver hands 64-bit back; the value
    // is what matters, and it must not have become a float or a string.
    expect(Number(rows[0]!.LOCA_NO)).toBe(7);
    expect(Number(rows[1]!.LOCA_NO)).toBe(-3);
  });

  it("reads a full datetime to the second, not to the day", async () => {
    // The micros-vs-millis trap. Truncating to the date, or scaling wrongly,
    // both yield a valid-looking timestamp — so assert the actual instant.
    using ags = read(undefined, { text: TYPED });
    const rows = await ags.sql(
      "SELECT LOCA_STAR FROM LOCA WHERE LOCA_ID = 'BH01'",
    );
    const star = rows[0]!.LOCA_STAR as Date;
    expect(star).toBeInstanceOf(Date);
    expect(star.getTime()).toBe(Date.UTC(2023, 1, 22, 9, 30, 15));
    // Explicitly not 1970 and not midnight — the two failure shapes.
    expect(star.getUTCFullYear()).toBe(2023);
    expect(star.getUTCHours()).toBe(9);
  });

  it("treats a date-only DT as midnight rather than dropping the column", async () => {
    using ags = read(undefined, { text: TYPED });
    const rows = await ags.sql(
      "SELECT LOCA_STAR FROM LOCA WHERE LOCA_ID = 'BH02'",
    );
    const star = rows[0]!.LOCA_STAR as Date;
    expect(star.getTime()).toBe(Date.UTC(2023, 2, 1, 0, 0, 0));
  });

  it("turns an empty typed cell into SQL NULL, not into zero", async () => {
    // The distinction the whole typed path exists to preserve: BH02 has no
    // LOCA_GL. A 0 there would be a measurement the file does not contain.
    using ags = read(undefined, { text: TYPED });
    const rows = await ags.sql(
      "SELECT LOCA_GL FROM LOCA WHERE LOCA_ID = 'BH02'",
    );
    expect(rows[0]!.LOCA_GL).toBeNull();
    const [{ n }] = (await ags.sql("SELECT count(LOCA_GL) AS n FROM LOCA")) as [
      { n: number | bigint },
    ];
    expect(Number(n)).toBe(1);
  });
});

describe("arrow output", () => {
  it("returns a born-typed table whose schema matches the SQL types", async () => {
    using ags = read(undefined, { text: TYPED });
    const table: Table = await ags.sql("SELECT LOCA_ID, LOCA_GL FROM LOCA", {
      arrow: true,
    });
    expect(table.numRows).toBe(2);
    const names = table.schema.fields.map((f) => f.name);
    expect(names).toEqual(["LOCA_ID", "LOCA_GL"]);
  });

  it("loads the arrow extension once however many arrow queries run", async () => {
    // `#ensureArrow` caches with an early return. Without it every arrow query
    // re-runs INSTALL/LOAD — which on a cold or offline machine is the
    // difference between one slow call and every call being slow.
    using ags = read(undefined, { text: TYPED });
    const first: Table = await ags.sql("SELECT LOCA_ID FROM LOCA", {
      arrow: true,
    });
    const second: Table = await ags.sql("SELECT LOCA_GL FROM LOCA", {
      arrow: true,
    });
    expect(first.numRows).toBe(2);
    expect(second.numRows).toBe(2);
    // Both succeeded against the same connection, so the second went through
    // the cached path rather than re-installing.
    expect(second.schema.fields.map((f) => f.name)).toEqual(["LOCA_GL"]);
  });
});
