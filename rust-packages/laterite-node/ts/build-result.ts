// The product of `buildAgs4()` — the Node port of laterite-py's `BuildResult`:
// the AGS4 `bytes`, the validator `findings` on those bytes (post-fix in AutoFix
// mode), and the count of safe fixes applied.
import { writeFileSync } from "node:fs";

/** A flattened build finding — `rule` plus whatever rich keys the validator set. */
export interface BuildFinding {
  rule: string;
  line?: number;
  group?: string;
  desc?: string;
  [key: string]: unknown;
}

export class BuildResult {
  constructor(
    readonly bytes: Buffer,
    readonly findings: BuildFinding[],
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
