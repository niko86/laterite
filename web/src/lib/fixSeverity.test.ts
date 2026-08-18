import { describe, expect, it } from "vitest";
import { buildSevIndex, fixSeverity } from "./fixSeverity";
import type { Fix, FindingDto, RuleGroup, ValidationReport } from "./validator";

// Fixtures carry only the fields the join reads; the rest exist to satisfy the
// wire shapes. `severity` is omitted for errors on purpose — that IS how the
// engine encodes an error (see `severityOf`).
const finding = (
  line: number | null,
  severity?: "warning" | "fyi",
): FindingDto => ({
  line,
  group: "LOCA",
  desc: "",
  ...(severity ? { severity } : {}),
});

const report = (findings: RuleGroup[]): ValidationReport => ({
  ok: false,
  dict_version: "4.1.1",
  resolution: "exact",
  finding_count: findings.reduce((n, g) => n + g.items.length, 0),
  shown_count: findings.reduce((n, g) => n + g.items.length, 0),
  findings,
  error: null,
  revalidate_reason: null,
});

const fix = (rule: string, line: number | null): Fix =>
  ({
    kind: "strip_bom",
    label: "",
    rule,
    line,
    risk: "safe",
    edits: [],
  }) as unknown as Fix;

describe("fixSeverity", () => {
  // #412. The window: computeFixes settles, the worker dies, the labelling
  // validate never answers. `retire()` rejects only the PENDING op, so the pane
  // holds real fixes and no report. This used to answer "warning" — a confident
  // label on a fix whose finding nobody looked at, indistinguishable from a
  // genuine warning, and the same mistake `severityOf` records being fixed at
  // five other sites.
  it("has no severity to report when there is no report at all", () => {
    const idx = buildSevIndex(undefined);
    expect(fixSeverity(idx, fix("AGS Format Rule 1", 3))).toBeUndefined();
    expect(fixSeverity(idx, fix("AGS Format Rule 1", null))).toBeUndefined();
  });

  // The other half of #412: a report that simply doesn't mention this fix's
  // rule is a DIFFERENT absence, and it keeps the long-standing default. If
  // this goes undefined too, the change has repainted the benign case.
  it("still defaults to warning when a report exists but omits the rule", () => {
    const idx = buildSevIndex(
      report([
        { rule: "AGS Format Rule 8", total: 1, items: [finding(2, "fyi")] },
      ]),
    );
    expect(fixSeverity(idx, fix("AGS Format Rule 1", 3))).toBe("warning");
  });

  it("prefers the rule+line hit over the rule-wide one", () => {
    const idx = buildSevIndex(
      report([
        {
          rule: "AGS Format Rule 1",
          total: 2,
          items: [finding(3, "fyi"), finding(9)],
        },
      ]),
    );
    // line 3 is FYI; the rule as a whole is error (line 9 omits `severity`).
    expect(fixSeverity(idx, fix("AGS Format Rule 1", 3))).toBe("fyi");
    expect(fixSeverity(idx, fix("AGS Format Rule 1", 42))).toBe("error");
  });

  it("takes the most severe when one rule+line carries several findings", () => {
    const idx = buildSevIndex(
      report([
        {
          rule: "AGS Format Rule 1",
          total: 2,
          items: [finding(3, "fyi"), finding(3)],
        },
      ]),
    );
    expect(fixSeverity(idx, fix("AGS Format Rule 1", 3))).toBe("error");
  });
});
