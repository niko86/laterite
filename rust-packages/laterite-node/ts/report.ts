import type { Finding, ValidationReport } from "./native";

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
 * it is `true` iff there are zero findings, which is deliberately *distinct* from
 * the native `ok` flag (`ok` only means the source was parseable enough to
 * validate). `count` is the number of findings, and `exitCode` mirrors what the
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
    return this.#r.certified ?? false;
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
  get count(): number {
    return this.#r.count;
  }
  /** `true` iff there are zero findings (distinct from the native `ok`, which
   * only means "validatable"). */
  get isValid(): boolean {
    return this.#r.count === 0;
  }
  get exitCode(): number {
    return this.#r.exitCode;
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

  /** `{file, findings:{…}}` pretty-JSON — byte-identical to `lat-check --json`. */
  toJson(): string {
    return this.#r.json;
  }
  /** One flat `{rule, …}` per line — byte-identical to `lat-check --ndjson`. */
  toNdjson(): string {
    return this.#r.ndjson;
  }

  toString(): string {
    const v = this.isValid ? "valid" : `${this.count} finding(s)`;
    return `<Report ${JSON.stringify(this.file)} ${v} dict=${this.dictVersion}>`;
  }
}
