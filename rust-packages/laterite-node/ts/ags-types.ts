// laterite.agsTypes — the AGS4 type system, the Node port of laterite-py's
// `ags_types`. The parsing LOGIC is native (one shared engine across hosts);
// this is just the typed TS face over it.
import { Ags4Error } from "./errors";
import {
  canonicalType as nativeCanonicalType,
  displayHint,
  parseValue as nativeParseValue,
} from "./native";

/** Cross-system target categories — the lowercase labels the engine returns. */
export type CanonicalType =
  | "string"
  | "integer"
  | "decimal"
  | "datetime"
  | "date"
  | "time"
  | "bool"
  | "enum";

/** A parsed AGS value: number (integer/decimal), boolean (YN), string
 * (text/enum **and** the canonical datetime/date/time strings), or null. */
export type AgsValue = string | number | boolean | null;

/**
 * AGS spec type code → canonical category. Throws for unknown codes (mirrors
 * Python's `ValueError`), so the engine's permissive `null` never leaks into
 * caller code as a silent miss.
 *
 * @param agsType - An AGS4 spec type code (e.g. `"2DP"`, `"ID"`, `"YN"`, `"DT"`).
 * @returns The lowercase canonical category the engine maps that code to.
 * @throws {Ags4Error} If the code is not a recognised AGS type.
 */
export function canonicalType(agsType: string): CanonicalType {
  const label = nativeCanonicalType(agsType);
  if (label === null)
    throw new Ags4Error(`unknown AGS type code: ${JSON.stringify(agsType)}`);
  return label as CanonicalType;
}

export { displayHint };

/** Coerce a non-null scalar to a string without the `[object Object]` footgun
 *  (an unexpected object → JSON; AGS cells are primitives, so that branch is
 *  defensive only). */
function scalarString(value: unknown): string {
  if (typeof value === "string") return value;
  if (
    typeof value === "number" ||
    typeof value === "boolean" ||
    typeof value === "bigint"
  )
    return String(value);
  return JSON.stringify(value);
}

/**
 * Parse an AGS4-shaped raw value into its canonical JS value (empty /
 * unparseable → null). datetime/date/time come back as the canonical **string**
 * (engine shape; `new Date(s)` if you want a Date). Non-string input is
 * stringified first (matches the Python wrapper).
 *
 * @param raw - The raw cell value; non-string input is stringified, and `null`/`undefined` short-circuit to `null`.
 * @param agsType - The AGS4 spec type code governing how `raw` is interpreted.
 * @returns The canonical value: a number (integer/decimal), boolean (YN), string (text/enum and the datetime/date/time strings), or `null`.
 */
export function parseValue(raw: unknown, agsType: string): AgsValue {
  const s = raw === null || raw === undefined ? null : scalarString(raw);
  return nativeParseValue(s, agsType) as AgsValue;
}
