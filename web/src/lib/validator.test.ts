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
// stay red, and an un-tagged (legacy) finding must NOT be mistaken for FYI.

const find = (severity?: FindingDto["severity"]): FindingDto => ({
  line: 1,
  group: "LOCA",
  desc: "x",
  severity,
});

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
    expect(reportIsOnlyFyi(report([...many, find("error")]))).toBe(false);
  });

  it("treats an un-tagged finding as non-FYI (severity defaults to warning)", () => {
    expect(reportIsOnlyFyi(report([find(undefined)]))).toBe(false);
  });

  it("is false when a warning is present", () => {
    expect(reportIsOnlyFyi(report([find("fyi"), find("warning")]))).toBe(false);
  });

  it("scans across rule groups, not just the first", () => {
    const r = report([find("fyi")]);
    r.findings.push({ rule: "Rule 9", total: 1, items: [find("error")] });
    r.finding_count = 2;
    expect(reportIsOnlyFyi(r)).toBe(false);
  });
});

describe("severityCounts / reportSeverity", () => {
  it("splits a mixed report by severity (the user's 36 + 14 case)", () => {
    const findings = [
      ...Array(36).fill(0).map(() => find("error")),
      ...Array(14).fill(0).map(() => find("fyi")),
    ];
    expect(severityCounts(report(findings))).toEqual({
      error: 36,
      warning: 0,
      fyi: 14,
    });
  });

  it("counts an un-tagged finding as a warning", () => {
    expect(severityCounts(report([find(undefined), find("warning")]))).toEqual({
      error: 0,
      warning: 2,
      fyi: 0,
    });
  });

  it("reportSeverity is exact for an uncapped report", () => {
    const r = report([find("error"), find("fyi")]);
    expect(reportSeverity(r)).toEqual({
      counts: { error: 1, warning: 0, fyi: 1 },
      exact: true,
    });
  });

  it("reportSeverity is NOT exact once the per-rule cap clips items", () => {
    // 3 serialized items but a true total of 9000 ⇒ the split would undercount.
    const r = report([find("error"), find("error"), find("fyi")]);
    r.findings[0].total = 9000;
    r.finding_count = 9000;
    expect(reportSeverity(r).exact).toBe(false);
  });
});
