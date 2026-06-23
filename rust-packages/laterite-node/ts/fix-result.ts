// The product of `fix()` — the Node port of laterite-py's `FixResult`: the
// repaired `bytes`, the residual `findings` (what could NOT be mechanically
// fixed, after re-validation), and the `applied` fixes.
import { writeFileSync } from "node:fs";
import type { AppliedFix, Finding } from "./native";

export type { AppliedFix };

export class FixResult {
  constructor(
    readonly bytes: Buffer,
    /** Findings that remain after the fixes — what could NOT be mechanically fixed. */
    readonly findings: Finding[],
    /** The fixes that were applied (`{kind, label, rule, line?, risk}`). */
    readonly applied: AppliedFix[],
    readonly dictVersion: string,
  ) {}

  /** How many fixes were applied. */
  get fixesApplied(): number {
    return this.applied.length;
  }

  /** The repaired AGS4 document decoded as text. */
  get text(): string {
    return this.bytes.toString("utf8");
  }

  /** Save the repaired bytes to `path`; returns `path`. */
  save(path: string): string {
    writeFileSync(path, this.bytes);
    return path;
  }

  toString(): string {
    return `<FixResult ${this.bytes.length} bytes, ${this.applied.length} fix(es) applied, ${this.findings.length} residual finding(s)>`;
  }
}
