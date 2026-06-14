// The outcome of `validate()` — the Node port of laterite-py's `Report`.
// `toJson` / `toNdjson` are byte-faithful to the `ags4-check` binary (produced
// native-side; see `lib.rs::findings_json`). `findings` is a plain array (Node
// has no polars analog; the data is small + structured, so an array is more
// ergonomic than an arrow-js Table here).
import type { Finding, ValidationReport } from "./native";

/** One finding without its `rule` key — the value shape `byRule()` groups. */
export type RuleFinding = Omit<Finding, "rule">;

export class Report {
  readonly #r: ValidationReport;
  constructor(r: ValidationReport) {
    this.#r = r;
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
  /** All findings, in `ags4-check` order: `{rule, line?, group, desc, severity?}`. */
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

  /** `{file, findings:{…}}` pretty-JSON — byte-identical to `ags4-check --json`. */
  toJson(): string {
    return this.#r.json;
  }
  /** One flat `{rule, …}` per line — byte-identical to `ags4-check --ndjson`. */
  toNdjson(): string {
    return this.#r.ndjson;
  }

  toString(): string {
    const v = this.isValid ? "valid" : `${this.count} finding(s)`;
    return `<Report ${JSON.stringify(this.file)} ${v} dict=${this.dictVersion}>`;
  }
}
