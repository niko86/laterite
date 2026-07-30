import { describe, expect, it } from "vitest";
import {
  reportIsOnlyFyi,
  reportSeverity,
  severityCounts,
  type FindingDto,
  type ValidationReport,
} from "./validator";

// reportIsOnlyFyi decides amber (informational) vs red (failure) in the
// SummaryBanner. The boundaries matter: one real error among many FYI must
// stay red.
//
// These fixtures used to pass `severity: "error"`, which the engine NEVER
// emits — it omits the field for errors and writes it only for warning/fyi.
// So the tests were exercising a shape that cannot arrive, while the real
// error shape (absent) was described as "un-tagged (legacy)" and asserted to
// count as a WARNING. That assertion was the bug, written down: it is why the
// banner's split and the severity filter both mis-classified every error.
//
// `anError()` now spells the wire truth, and the union no longer admits
// "error" at all, so the old fixtures cannot be written back by accident.

const find = (severity?: FindingDto["severity"]): FindingDto => ({
  line: 1,
  group: "LOCA",
  desc: "x",
  severity,
});

/** An error as the engine actually serialises one: no `severity` key. */
const anError = (): FindingDto => find(undefined);

const report = (findings: FindingDto[]): ValidationReport => ({
  ok: findings.length === 0,
  dict_version: "4.1.1",
  resolution: "fallback",
  finding_count: findings.length,
  shown_count: findings.length,
  findings: findings.length
    ? [{ rule: "Rule 1", total: findings.length, items: findings }]
    : [],
  error: null,
  revalidate_reason: null,
});

describe("reportIsOnlyFyi", () => {
  it("is false for a clean report (0 findings ⇒ green, not amber)", () => {
    expect(reportIsOnlyFyi(report([]))).toBe(false);
  });

  it("is true when every finding is FYI", () => {
    expect(reportIsOnlyFyi(report([find("fyi"), find("fyi")]))).toBe(true);
  });

  it("is false when one error hides among many FYI (stays red)", () => {
    const many = [...Array(20)].map(() => find("fyi"));
    expect(reportIsOnlyFyi(report([...many, anError()]))).toBe(false);
  });

  it("treats a finding with no severity key as non-FYI (it is an error)", () => {
    expect(reportIsOnlyFyi(report([anError()]))).toBe(false);
  });

  it("is false when a warning is present", () => {
    expect(reportIsOnlyFyi(report([find("fyi"), find("warning")]))).toBe(false);
  });

  it("scans across rule groups, not just the first", () => {
    const r = report([find("fyi")]);
    r.findings.push({ rule: "Rule 9", total: 1, items: [anError()] });
    r.finding_count = 2;
    expect(reportIsOnlyFyi(r)).toBe(false);
  });
});

describe("severityCounts / reportSeverity", () => {
  it("splits a mixed report by severity (the user's 36 + 14 case)", () => {
    const findings = [
      ...Array(36)
        .fill(0)
        .map(() => anError()),
      ...Array(14)
        .fill(0)
        .map(() => find("fyi")),
    ];
    expect(severityCounts(report(findings))).toEqual({
      error: 36,
      warning: 0,
      fyi: 14,
    });
  });

  // The regression test for the defect this file used to assert as correct:
  // an absent `severity` is an ERROR, and counting it as a warning is exactly
  // what made the banner under-report errors.
  it("counts a finding with no severity key as an error, not a warning", () => {
    expect(severityCounts(report([anError(), find("warning")]))).toEqual({
      error: 1,
      warning: 1,
      fyi: 0,
    });
  });

  it("reportSeverity is exact for an uncapped report", () => {
    const r = report([anError(), find("fyi")]);
    expect(reportSeverity(r)).toEqual({
      counts: { error: 1, warning: 0, fyi: 1 },
      exact: true,
    });
  });

  it("reportSeverity is NOT exact once the per-rule cap clips items", () => {
    // 3 serialized items but a true total of 9000 ⇒ the split would undercount.
    const r = report([anError(), anError(), find("fyi")]);
    r.findings[0]!.total = 9000;
    r.finding_count = 9000;
    expect(reportSeverity(r).exact).toBe(false);
  });
});
