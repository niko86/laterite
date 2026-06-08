// "Why is this column typed as X?" — a plain-English description of an AGS4
// TYPE code, mirroring the canonical categories `ags5-types::canonical_type`
// uses to decide the in-browser DuckDB column type. Kept in sync with
// rust-packages/ags5-types/src/lib.rs (the authority) by hand; it's a small,
// stable mapping (the AGS type vocabulary rarely changes).

/** Human description of an AGS4 TYPE code (e.g. "2DP" → "decimal, 2 places").
 *  Falls back to "text" for unknown / opaque codes (the engine treats those
 *  as strings too). */
export function typeDescription(agsType: string): string {
  const t = (agsType || "").trim().toUpperCase();
  if (t === "0DP") return "whole number";
  let m = t.match(/^(\d+)DP$/);
  if (m) return `decimal, ${m[1]} place${m[1] === "1" ? "" : "s"}`;
  m = t.match(/^(\d+)SF$/);
  if (m) return `decimal, ${m[1]} significant figures`;
  m = t.match(/^(\d+)SCI$/);
  if (m) return `scientific notation, ${m[1]} d.p.`;
  switch (t) {
    case "DT":
      return "date / time";
    case "YN":
      return "yes / no (boolean)";
    case "ID":
      return "identifier";
    case "PA":
      return "picklist (abbreviation)";
    case "PT":
      return "picklist (free, with abbreviations)";
    case "PU":
      return "picklist (units)";
    case "T":
      return "elapsed time";
    case "U":
      return "unitless / unit string";
    case "DMS":
      return "degrees:minutes:seconds";
    case "MC":
      return "moisture-condition value";
    case "X":
      return "text";
    case "XN":
      return "text (numeric-looking)";
    default:
      return "text";
  }
}
