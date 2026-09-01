import { writeFileSync } from "node:fs";
import type { AppliedFix } from "./native";

/** A flattened build finding — `rule` plus whatever rich keys the validator set. */
export interface BuildFinding {
  rule: string;
  line?: number;
  group?: string;
  desc?: string;
  [key: string]: unknown;
}

/**
 * The product of {@link buildAgs4} — the data→AGS4 door's return value, and the
 * Node port of laterite-py's `BuildResult`. Where `read` hands you an *existing*
 * file, `buildAgs4` *constructs* one from your data and then runs the output back
 * through the validator; this object is what falls out of that round trip. It is
 * a plain, inert carrier — no DuckDB, no native handle to hold open — so you can
 * keep it, pass it around, or persist it at leisure.
 *
 * It carries three things. {@link BuildResult.bytes | `bytes`} is the AGS4
 * document as the validator emitted it (UTF-8); reach for {@link BuildResult.text
 * | `text`} when you want it decoded as a string, or {@link BuildResult.save |
 * `save(path)`} to write the bytes straight to disk (it returns the path).
 * {@link BuildResult.findings | `findings`} is the *residual* set of validator
 * findings — what the build could **not** clear given the `mode` it ran under
 * (e.g. `"autofix"` applies the safe fixes and leaves only what it can't touch,
 * `"report"` records everything); each is a flat {@link BuildFinding} of `rule`
 * plus whatever rich keys the validator set. {@link BuildResult.fixesApplied |
 * `fixesApplied`} counts how many safe fixes were applied along the way, and
 * {@link BuildResult.applied | `applied`} is the ledger of those fixes (each a
 * `{kind, label, rule, line?, risk}` record, the same shape `fix()`'s
 * `FixResult.applied` carries). A clean build is an empty `findings` array; a
 * non-empty one tells you exactly what the emitted document still trips on.
 */
export class BuildResult {
  constructor(
    readonly bytes: Buffer,
    readonly findings: BuildFinding[],
    /** The safe fixes AutoFix applied (`{kind, label, rule, line?, risk}`); empty outside `"autofix"`. */
    readonly applied: AppliedFix[],
    readonly fixesApplied: number,
  ) {}

  /** The AGS4 document decoded as text. */
  get text(): string {
    return this.bytes.toString("utf8");
  }

  /** Save the bytes to `path`; returns `path`. */
  save(path: string): string {
    writeFileSync(path, this.bytes);
    return path;
  }

  toString(): string {
    return `<BuildResult ${this.bytes.length} bytes, ${this.findings.length} finding(s), fixesApplied=${this.fixesApplied}>`;
  }
}

/**
 * What `buildAgs4(..., { out })` hands back: the verdict on a file already on
 * disk — the to-disk twin of {@link BuildResult}, mirroring laterite-py's
 * `BuildSaved`. Same `findings` / `applied` / `fixesApplied` verdict, but the
 * document lives at {@link BuildSaved.path | `path`} and there is deliberately
 * no `bytes`: the point of `out` is a long-lived caller that does not want the
 * whole file resident after the call, and a result quietly carrying it anyway
 * would defeat that.
 *
 * Build-and-judge survives the trip to disk: the bytes are staged to a
 * temporary file beside the destination and renamed into place only after the
 * verdict allows, so `path` never holds unjudged output — a `"strict"` failure
 * throws with nothing written.
 */
export class BuildSaved {
  constructor(
    /** Where the judged AGS4 document was written. */
    readonly path: string,
    readonly findings: BuildFinding[],
    /** The safe fixes AutoFix applied (`{kind, label, rule, line?, risk}`); empty outside `"autofix"`. */
    readonly applied: AppliedFix[],
    readonly fixesApplied: number,
  ) {}

  toString(): string {
    return `<BuildSaved ${JSON.stringify(this.path)}, ${this.findings.length} finding(s), fixesApplied=${this.fixesApplied}>`;
  }
}
