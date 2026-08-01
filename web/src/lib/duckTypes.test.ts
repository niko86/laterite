// The cell-coercion boundary — the last thing between a DuckDB-wasm value and
// the DOM.
//
// `formatCell`'s own comment names the two values that must never reach a
// component raw: an Int64 column comes back as a JS `bigint` (rendering one in
// JSX, or `JSON.stringify`-ing it, THROWS), and a TIMESTAMP comes back as an
// integer count of MICROseconds since the epoch, not a `Date`. It calls itself
// "the single audited place those are normalised" — and it was the least-covered
// module in the web bundle, at 15% of lines.
//
// Both failures are the kind that survive review: a bigint renders fine in every
// test fixture that happens to use small integers, and a micros timestamp read as
// millis produces a plausible-looking date in the year 1970 rather than an error.
import { describe, expect, it } from "vitest";

import { formatCell, scalarText } from "./duckTypes";

describe("scalarText", () => {
  it.each([
    [null, ""],
    [undefined, ""],
    ["already a string", "already a string"],
    ["", ""],
    [0, "0"],
    [-1.5, "-1.5"],
    [true, "true"],
    [false, "false"],
  ])("coerces %o to %o", (input, expected) => {
    expect(scalarText(input)).toBe(expected);
  });

  it("renders a bigint without throwing", () => {
    // The whole reason this function exists. `String(bigint)` is safe;
    // `JSON.stringify(bigint)` throws "Do not know how to serialize a BigInt",
    // which is what a naive implementation would reach for.
    expect(scalarText(9007199254740993n)).toBe("9007199254740993");
    // Past Number.MAX_SAFE_INTEGER — proving it is not routed through Number().
    expect(scalarText(9007199254740993n)).not.toBe("9007199254740992");
  });

  it("never produces [object Object]", () => {
    // The named footgun in the module header. An unexpected object must arrive as
    // inspectable JSON, not the string every user has learned to ignore.
    expect(scalarText({ a: 1 })).toBe('{"a":1}');
    expect(scalarText([1, 2])).toBe("[1,2]");
    expect(scalarText({ a: 1 })).not.toContain("[object");
  });

  it("returns a string for every input shape", () => {
    // The invariant the callers rely on: whatever DuckDB hands back, what comes
    // out of here is safe to put in a text node.
    const inputs: unknown[] = [
      null,
      undefined,
      "s",
      1,
      0,
      true,
      1n,
      {},
      [],
      new Date(0),
    ];
    for (const v of inputs) expect(typeof scalarText(v)).toBe("string");
  });
});

describe("formatCell with a temporal column", () => {
  // 2020-08-18T09:30:15Z. Derived rather than hand-typed: a transposed digit in a
  // literal like this is invisible, and it would have to be wrong in the SAME way
  // in the expectation for the test to pass — which is how a "correct" constant
  // ends up asserting the wrong instant.
  const MICROS = Date.UTC(2020, 7, 18, 9, 30, 15) * 1000;
  const EXPECTED = "2020-08-18 09:30:15";

  it("reads micros-since-epoch from a bigint", () => {
    expect(formatCell(BigInt(MICROS), "TIMESTAMP")).toBe(EXPECTED);
  });

  it("reads micros-since-epoch from a number", () => {
    expect(formatCell(MICROS, "TIMESTAMP")).toBe(EXPECTED);
  });

  it("does not mistake micros for millis", () => {
    // The silent failure this guards. Treating micros as millis puts every
    // timestamp in 1970 — a plausible date, no error, and wrong by 50 years.
    const rendered = formatCell(MICROS, "TIMESTAMP");
    expect(rendered.startsWith("2020")).toBe(true);
    expect(rendered.startsWith("1970")).toBe(false);
  });

  it("accepts a Date, which DuckDB sometimes hands back instead", () => {
    expect(formatCell(new Date(MICROS / 1000), "TIMESTAMP")).toBe(EXPECTED);
  });

  it("drops the ISO T/Z and the fractional part", () => {
    // A flatter look than toISOString(), and tz-naive to match the Rust cast.
    const out = formatCell(MICROS, "TIMESTAMP");
    expect(out).not.toContain("T");
    expect(out).not.toContain("Z");
    expect(out).not.toMatch(/\.\d+/);
    expect(out).toMatch(/^\d{4}-\d{2}-\d{2} \d{2}:\d{2}:\d{2}$/);
  });

  it.each([
    "TIMESTAMP",
    "timestamp",
    "TimeStamp",
    "TIMESTAMP WITH TIME ZONE",
    "DATE",
  ])("recognises %s", (sqlType) => {
    // The match is `toUpperCase().startsWith(...)`, so casing and a trailing
    // qualifier must not fall through to the plain-text path.
    expect(formatCell(MICROS, sqlType)).toBe(EXPECTED);
  });

  it("falls back to plain text when the value is not a usable instant", () => {
    // A temporal COLUMN can still hold something unparseable. Better a visible
    // raw value than "Invalid Date" or a thrown RangeError inside a render.
    expect(formatCell("not a timestamp", "TIMESTAMP")).toBe("not a timestamp");
    expect(formatCell(Number.NaN, "TIMESTAMP")).toBe("NaN");
    expect(formatCell(Number.POSITIVE_INFINITY, "TIMESTAMP")).toBe("Infinity");
  });

  it("returns empty string for a null instant, not the epoch", () => {
    // A NULL timestamp must not render as 1970-01-01 — that is a value the file
    // does not contain.
    expect(formatCell(null, "TIMESTAMP")).toBe("");
    expect(formatCell(undefined, "DATE")).toBe("");
  });

  it("renders the epoch itself when the value really is zero", () => {
    // The mirror of the case above: 0 is a real instant, not a missing one, so
    // it must not be swallowed by a falsy check.
    expect(formatCell(0, "TIMESTAMP")).toBe("1970-01-01 00:00:00");
  });
});

describe("formatCell with a non-temporal column", () => {
  it("passes a BIGINT through as its full decimal string", () => {
    expect(formatCell(9007199254740993n, "BIGINT")).toBe("9007199254740993");
  });

  it.each([
    ["VARCHAR", "text", "text"],
    ["DOUBLE", 1.25, "1.25"],
    ["BOOLEAN", false, "false"],
    ["", "no type at all", "no type at all"],
    ["SOMETHING_UNMAPPED", 42, "42"],
  ])("renders a %s cell", (sqlType, value, expected) => {
    expect(formatCell(value, sqlType)).toBe(expected);
  });

  it("returns empty string for null and undefined", () => {
    expect(formatCell(null, "VARCHAR")).toBe("");
    expect(formatCell(undefined, "DOUBLE")).toBe("");
  });

  it("never returns a non-string, whatever the column claims to be", () => {
    // The property every grid depends on. A bigint escaping here throws at render
    // time, in a component, far from this module.
    const cells: [unknown, string][] = [
      [1n, "BIGINT"],
      [1n, "VARCHAR"],
      [{ nested: true }, "VARCHAR"],
      [new Date(0), "TIMESTAMP"],
      [Number.NaN, "DOUBLE"],
    ];
    for (const [value, sqlType] of cells) {
      expect(typeof formatCell(value, sqlType)).toBe("string");
    }
  });
});
