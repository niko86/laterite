// P3 — laterite.agsTypes: canonicalType / displayHint / parseValue, the typed
// face over the native AGS type engine (the same parsing the read path uses).
import { describe, expect, it } from "vitest";
import { Ags4Error, agsTypes } from "../ts/index";

describe("agsTypes.canonicalType", () => {
  it("maps AGS type codes to their canonical category", () => {
    expect(agsTypes.canonicalType("ID")).toBe("string");
    expect(agsTypes.canonicalType("PA")).toBe("string"); // PA (pick-list) is a string type
    expect(agsTypes.canonicalType("2DP")).toBe("decimal");
    // RL is a Record Link — `GROUP|KEY1|KEY2`, split on TRAN_DLIM (AGS Rule 11).
    // The dictionary's own type list says so: {"code":"RL","description":"Record link"}.
    // This line asserted "decimal", which made sql_type DOUBLE and parse_value Null:
    // every record link was destroyed on read (laterite-dev#503). Third language to pin it.
    expect(agsTypes.canonicalType("RL")).toBe("string");
    expect(agsTypes.canonicalType("0DP")).toBe("integer");
    expect(agsTypes.canonicalType("DT")).toBe("datetime");
    expect(agsTypes.canonicalType("YN")).toBe("bool");
  });

  it("throws for an unknown code", () => {
    expect(() => agsTypes.canonicalType("NOPE")).toThrow(Ags4Error);
  });
});

describe("agsTypes.displayHint", () => {
  it("gives a printf hint for numeric types, null otherwise", () => {
    expect(agsTypes.displayHint("2DP")).toBe("%.2f");
    expect(agsTypes.displayHint("3SF")).toBe("%.3g");
    expect(agsTypes.displayHint("ID")).toBeNull();
  });
});

describe("agsTypes.parseValue", () => {
  it("parses to native JS values (same engine as the read path)", () => {
    expect(agsTypes.parseValue("12.30", "2DP")).toBe(12.3);
    expect(agsTypes.parseValue("5.0", "0DP")).toBe(5); // int tolerates "5.0"
    expect(agsTypes.parseValue("Y", "YN")).toBe(true);
    expect(agsTypes.parseValue("N", "YN")).toBe(false);
    expect(agsTypes.parseValue("BH01", "ID")).toBe("BH01");
    // datetime/date come back as the canonical string (engine shape).
    expect(agsTypes.parseValue("2023-02-22", "DT")).toBe("2023-02-22 00:00:00");
    expect(agsTypes.parseValue("2023-02-22", "DATE")).toBe("2023-02-22");
  });

  it("is permissive: empty / unparseable → null", () => {
    expect(agsTypes.parseValue("", "2DP")).toBeNull();
    expect(agsTypes.parseValue(null, "ID")).toBeNull();
    expect(agsTypes.parseValue("not-a-number", "2DP")).toBeNull();
  });

  it("stringifies non-string input and passes unknown codes through", () => {
    expect(agsTypes.parseValue(42, "2DP")).toBe(42);
    expect(agsTypes.parseValue("anything", "ZZZ")).toBe("anything"); // unknown code → trimmed string
  });
});
