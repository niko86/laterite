import type { Finding, ValidationReport } from "./native";

/** `maxFindingsPerRule`, checked: a count, so a negative or fractional one throws
 *  rather than reaching the native `u32` and wrapping. */
export function findingsCap(n: number | undefined): number | undefined {
  if (n === undefined) return undefined;
  if (!Number.isInteger(n) || n < 0 || n > 0xffffffff)
    throw new RangeError(
      `maxFindingsPerRule must be a non-negative integer, not ${String(n)}`,
    );
  return n;
}

/** One finding without its `rule` key — the value shape `byRule()` groups. */
export type RuleFinding = Omit<Finding, "rule">;

/**
 * The verdict from {@link validate} — the Node port of laterite-py's `Report`.
 *
 * `validate()` runs the numbered-rule engine over an AGS4 source and hands back
 * one of these: an immutable wrapper over the native `ValidationReport`. It is a
 * pure *result* object (it does not carry the file's bytes — that is what the
 * read/build outputs are for); what it carries is the answer to "is this file
 * conformant, and if not, why".
 *
 * Start with the headline getters. `isValid` is the one most callers branch on —
 * it is the engine's VERDICT, which is deliberately *distinct* from the native
 * `ok` flag (`ok` only means the source was parseable enough to validate) and,
 * since #321, from "zero findings": a warning is reported without being fatal,
 * so a valid file can carry findings. `count` is the number of findings —
 * `errors`/`warnings`/`fyi` split it by tier — and `exitCode` mirrors what the
 * `lat-check` binary would return for the same file, so a CLI wrapper can pass it
 * straight through. The provenance getters — `file`, `dictVersion` (the AGS
 * edition the rules were drawn from), and `resolution` — say *what* was checked
 * and *against which dictionary*.
 *
 * The findings themselves come three ways. `findings` is the flat array in
 * `lat-check` order (each `{rule, line?, group, desc, severity?}`). `byRule()`
 * regroups them into the spec-rule map `{ "AGS Format Rule N": [...] }` for
 * rule-oriented reporting. For machine output, `toJson()` and `toNdjson()` return
 * strings byte-identical to `lat-check --json` / `--ndjson` (minted native-side),
 * and `toString()` gives a one-line human summary.
 *
 * @see {@link validate} — the verb that produces a `Report`.
 */
export class Report {
  readonly #r: ValidationReport;
  constructor(r: ValidationReport) {
    this.#r = r;
  }

  /** Did an `index` certificate stand in for the rule engine?
   *
   * NOT "the file was not checked". A certificate can only remove the CONTENT half of a
   * validation — the part that is a pure function of the file's bytes. Anything that
   * reads the world outside them (Rule 20's on-disk `FILE/` tree, via `checkFiles`) is
   * re-run every time, certificate or not, because a directory can change without the
   * file changing.
   *
   * `resolution` used to carry a `"certified"` sentinel instead of this, which made one
   * field answer two questions — *which dictionary judged the file* and *did we skip the
   * engine* — and answer neither properly. */
  get certified(): boolean {
    return this.#r.certified;
  }

  /** Why a proffered `index` certificate did NOT stand in for the engine, as a stable
   * token (`"content_changed"`, `"dictionary_changed"`, `"edition_differs"`,
   * `"encoding_differs"`, `"tier_not_measured_warnings"`, …), or `undefined`.
   *
   * `undefined` in the two ordinary cases: no certificate was offered, or one was offered
   * and vouched (then `certified` is `true`). A defined value means a cert *was* offered
   * but couldn't answer — the token says which guard fired. Mirrors
   * `laterite.Report.revalidate_reason`; the terse twin of `lat`'s human sentence. */
  get revalidateReason(): string | undefined {
    return this.#r.revalidateReason ?? undefined;
  }

  get file(): string {
    return this.#r.file;
  }
  get dictVersion(): string {
    return this.#r.dictVersion;
  }
  get resolution(): string {
    return this.#r.resolution;
  }
  /** Every finding the run made, across the tiers asked for. A total: under
   * `maxFindingsPerRule` it also counts the findings `findings` does not hold, and
   * `truncated` says how many, rule by rule. */
  get count(): number {
    return this.#r.count;
  }
  /** The rules `maxFindingsPerRule` cut: `{ rule: { shown, total } }`, in rule
   * order. `shown` is how many of the rule's findings `findings` holds and `total`
   * how many it made. Empty when nothing was cut, which is always the case without
   * a cap. Mirrors `laterite.Report.truncated`. */
  get truncated(): Record<string, { shown: number; total: number }> {
    const out: Record<string, { shown: number; total: number }> = {};
    for (const t of this.#r.truncated)
      out[t.rule] = { shown: t.shown, total: t.total };
    return out;
  }
  /** The VERDICT (distinct from the native `ok`, which only means
   * "validatable"). Not `count === 0`: since #321 a warning is reported without
   * being fatal, so a valid file can carry findings. The engine decides this
   * once — reading it here rather than recomputing keeps `isValid` and
   * `exitCode` from ever disagreeing. */
  get isValid(): boolean {
    return this.#r.valid;
  }
  get exitCode(): number {
    return this.#r.exitCode;
  }
  /** Findings in the error tier — the only tier that decides the verdict. */
  get errors(): number {
    return this.#r.errors;
  }
  /** Findings in the warning tier. Fatal only under `warningsAsErrors`. */
  get warnings(): number {
    return this.#r.warnings;
  }
  /** Findings in the FYI tier. Never fatal. */
  get fyi(): number {
    return this.#r.fyi;
  }
  /** All findings, in `lat-check` order: `{rule, line?, group, desc, severity?}`. */
  get findings(): Finding[] {
    return this.#r.findings;
  }

  /** `{ "AGS Format Rule N": [{line?, group, desc, …}] }` — the spec-rule
   * grouping (mirrors `Report.by_rule`). */
  byRule(): Record<string, RuleFinding[]> {
    const out: Record<string, RuleFinding[]> = {};
    for (const f of this.#r.findings) {
      const { rule, ...rest } = f;
      (out[rule] ??= []).push(rest);
    }
    return out;
  }

  /** `{file, findings:{…}}` pretty-JSON — byte-identical to `lat-check --json`.
   * A capped run that cut a rule adds `truncated: {rule: {shown, total}}`. */
  toJson(): string {
    return this.#r.json;
  }
  /** One flat `{rule, …}` per line — byte-identical to `lat-check --ndjson`. A
   * capped run that cut a rule ends with one `{"truncated": …}` record, which has
   * no `rule` key. */
  toNdjson(): string {
    return this.#r.ndjson;
  }

  toString(): string {
    const v = this.count === 0 ? "valid" : `${this.count} finding(s)`;
    return `<Report ${JSON.stringify(this.file)} ${v} dict=${this.dictVersion}>`;
  }
}
