/* The did-you-mean suggestion, folded into the error it is about (#1024).
 *
 * With FYI on, the engine raises O-61's suggestion beside every Rule 16 error
 * whose undefined value is close to a known code: `"b" under SAMP_TYPE is not
 * defined; did you mean "B" (declared in ABBR)?`. As a card of its own it
 * would read as a second verdict on the same cell, so every surface that
 * shows the error shows the suggestion UNDER it instead, and no card for it.
 *
 * Paired by what the two messages name, parsed out of their prose — the
 * value, the heading, and the group both findings are reported against — and
 * never by position: the engine's order is not a contract, and two undefined
 * values under one heading would otherwise swap explanations. The message
 * text is the contract here, as it is for `abbreviationTarget`; the e2e that
 * runs the real engine on the seeded delivery is what goes red on a rephrase.
 *
 * Pure, so the pairing is tested in the node lane (dec-web-test-altitude): the
 * components only render what this returns.
 */

import { abbreviationTarget, type Finding } from "./engine";

/** One card: a finding, and the suggestion folded under it (null when none). */
export type Shown = {
  readonly finding: Finding;
  readonly suggestion: Finding | null;
};

const SUGGESTION = /^"(.*)" under (\S+) is not defined; (did you mean .+\?)$/;

/** The value and heading a did-you-mean suggestion is about, and its "did you
 *  mean …?" clause, or null for every other finding. */
export function suggestionTarget(
  f: Finding,
): { value: string; heading: string; clause: string } | null {
  // The FYI label closes on a parenthesis: "FYI (Related to Rule 16)".
  if (f.severity !== "fyi" || !/\bRule 16\)$/.test(f.rule)) return null;
  const m = SUGGESTION.exec(f.desc);
  return m
    ? {
        value: m[1] as string,
        heading: m[2] as string,
        clause: m[3] as string,
      }
    : null;
}

/** The suggestion's own clause, capitalised to stand as a sentence under the
 *  error: "Did you mean "B" (declared in ABBR)?". */
export function suggestionText(suggestion: Finding): string {
  const clause = suggestionTarget(suggestion)?.clause ?? suggestion.desc;
  return clause.charAt(0).toUpperCase() + clause.slice(1);
}

const keyOf = (group: string, heading: string, value: string) =>
  JSON.stringify([group, heading, value]);

/** `list` as cards: each Rule 16 error carries the suggestion for its value,
 *  and a suggestion whose error is in `list` is dropped as a card of its own.
 *
 *  `pool` is where suggestions are looked up, `list` by default. A surface
 *  that shows the error without the suggestion in its own slice — the table
 *  cell's popover, whose findings are the cell's — passes the whole report so
 *  the error still carries it. A suggestion is only ever dropped when its
 *  error is on the SAME surface, so one can never vanish from a surface that
 *  is not showing what it explains. Any other finding, FYI or not, stays its
 *  own card. */
export function foldSuggestions(
  list: readonly Finding[],
  pool: readonly Finding[] = list,
): Shown[] {
  const byKey = new Map<string, Finding>();
  for (const f of pool) {
    const t = suggestionTarget(f);
    if (t) byKey.set(keyOf(f.group, t.heading, t.value), f);
  }
  const folded = new Set<Finding>();
  const cards: Shown[] = [];
  for (const f of list) {
    const t = abbreviationTarget(f);
    const suggestion = t
      ? (byKey.get(keyOf(f.group, t.heading, t.value)) ?? null)
      : null;
    if (suggestion) folded.add(suggestion);
    cards.push({ finding: f, suggestion });
  }
  return cards.filter((c) => !folded.has(c.finding));
}
