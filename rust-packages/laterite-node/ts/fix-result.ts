import { writeFileSync } from "node:fs";
import type { AppliedFix, Finding } from "./native";

export type { AppliedFix };

/**
 * The product of {@link fix} — the Node port of laterite-py's `FixResult`.
 *
 * `fix()` mechanically repairs an AGS4 document — always the *safe* set (CRLF /
 * BOM / embedded-CR normalisation, short-row padding, numeric reformatting, the
 * TRAN delimiter+concatenator rows), plus the intent-guessing *risky* set when
 * asked — then re-validates the repaired bytes. This object is what that pass
 * hands back: the fixed document, an account of what was changed, and an honest
 * record of what it could not touch. Construction is internal; you receive one
 * from `fix()`, you don't build it.
 *
 * The repair is non-destructive — nothing is written to disk by the fixer. The
 * repaired document rides home on `.bytes`, decoded on demand via the `.text`
 * getter (UTF-8), and persisted only when you choose to with `.save(path)`
 * (which writes the bytes and returns the path). `applied` enumerates each fix
 * that landed (`{kind, label, rule, line?, risk}` — serde snake_case, identical
 * across Python / CLI / Node), with `.fixesApplied` as its count for a quick
 * "did anything change?". Crucially, `findings` is *not* the input's problems —
 * it is the residual after re-validation: the rule violations that survived the
 * repair and still need a human. `dictVersion` records the AGS4 edition the fix
 * resolved against. `toString()` gives a one-line summary (byte count, fixes
 * applied, residual findings).
 */
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
