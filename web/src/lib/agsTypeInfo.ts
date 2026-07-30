// "Why is this column typed as X?" — a plain-English description of an AGS4
// TYPE code for the Explore "Analyse" view. The human DESCRIPTIONS here are
// web-authored display prose: this file is their home — they live nowhere else
// (laterite-ags4-types holds the type *classification*, not these glosses). The
// CATEGORY each code falls in (numeric / date / pick-list / text) tracks
// `laterite-ags4-types::canonical_type`, the gated authority for the actual in-browser
// DuckDB column type. A drift test (agsTypeInfo.test.ts) pins every AGS type code
// in the dictionary to a real description, so a newly-added code can't silently
// fall back to "text".

/** Human description of an AGS4 TYPE code (e.g. "2DP" → "decimal, 2 places").
 *  Falls back to "text" for unknown / opaque codes (the engine treats those
 *  as strings too). */
export function typeDescription(agsType: string): string {
  const t = (agsType || "").trim().toUpperCase();
  if (t === "0DP") return "whole number";
  // The capture group is guaranteed present when the regex matches, but
  // noUncheckedIndexedAccess still types it `string | undefined`; the
  // `!== undefined` guard narrows it to a real string for the template.
  const dp = /^(\d+)DP$/.exec(t)?.[1];
  if (dp !== undefined) return `decimal, ${dp} place${dp === "1" ? "" : "s"}`;
  const sf = /^(\d+)SF$/.exec(t)?.[1];
  if (sf !== undefined) return `decimal, ${sf} significant figures`;
  const sci = /^(\d+)SCI$/.exec(t)?.[1];
  if (sci !== undefined) return `scientific notation, ${sci} d.p.`;
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
    case "RL":
      return "record link";
    case "X":
      return "text";
    case "XN":
      return "text (numeric-looking)";
    default:
      return "text";
  }
}
