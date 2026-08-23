/* The severity grammar (#526). Pure: the engine names a severity, these map it
 * to one look — the UI never decides how bad something is.
 */

import { describe, expect, it } from "vitest";
import {
  severityCellTint,
  severityLineTint,
  severityRowEdge,
  severityRowTint,
  severityTint,
  worstPerLine,
  worstSeverity,
} from "./severity";
import type { Finding } from "./engine";

const finding = (severity: Finding["severity"]): Finding => ({
  rule: "Rule 8",
  line: 1,
  group: "LOCA",
  heading: "LOCA_GL",
  dataRow: 1,
  severity,
  desc: "",
});

describe("severityTint", () => {
  it("gives each engine severity its own look, and notes a neutral one", () => {
    const looks = new Set(
      ["error", "warning", "fyi", "note"].map(severityTint),
    );
    expect(looks.size).toBe(4);
  });

  it("treats an unknown severity as an error rather than unstyled", () => {
    // A new engine severity must fail LOUD (red), not render as plain text
    // nobody notices — on the callout AND on the cell variant.
    expect(severityTint("brand-new")).toBe(severityTint("error"));
    expect(severityCellTint("brand-new")).toBe(severityCellTint("error"));
  });
});

const at = (line: number | null, severity: string): Finding => ({
  ...finding(severity as Finding["severity"]),
  line,
});

describe("severityLineTint", () => {
  it("gives each tier its own band, and errors stay the loud fallback", () => {
    expect(severityLineTint("error")).toContain("border-l-err");
    expect(severityLineTint("warning")).toContain("border-l-warn");
    expect(severityLineTint("fyi")).toContain("border-l-info");
    expect(severityLineTint("brand-new-tier")).toBe(severityLineTint("error"));
  });
});

describe("worstPerLine", () => {
  it("bands each line with the worst tier among its findings", () => {
    const map = worstPerLine([at(7, "warning"), at(7, "error"), at(9, "fyi")]);
    expect(map.get(7)).toBe("error");
    expect(map.get(9)).toBe("fyi");
  });

  it("skips absence findings — no line, no band", () => {
    expect(worstPerLine([at(null, "error")]).size).toBe(0);
  });

  it("ranks an unknown tier as an error, so it wins its line", () => {
    const map = worstPerLine([at(3, "warning"), at(3, "brand-new-tier")]);
    expect(map.get(3)).toBe("brand-new-tier");
  });
});

describe("worstSeverity", () => {
  it("ranks error over warning over fyi", () => {
    expect(worstSeverity([finding("fyi"), finding("warning")])).toBe("warning");
    expect(
      worstSeverity([finding("fyi"), finding("error"), finding("warning")]),
    ).toBe("error");
    expect(worstSeverity([finding("fyi")])).toBe("fyi");
  });

  it("answers null for no findings — a cell with nothing wrong has no severity", () => {
    expect(worstSeverity([])).toBeNull();
  });

  it("ranks an unknown severity as an error, so it wins the cell and renders loud", () => {
    // The other half of the fail-loud promise: if an unknown tier ranked LOW
    // it would lose to a known fyi and never surface at all.
    const unknown = {
      ...finding("fyi"),
      severity: "brand-new" as Finding["severity"],
    };
    expect(worstSeverity([finding("warning"), unknown])).toBe("brand-new");
    expect(severityCellTint("brand-new")).toBe(severityCellTint("error"));
  });
});

describe("severityRowTint", () => {
  it("is a different dress from the cell grammar — a row fault is a different claim", () => {
    expect(severityRowTint("error")).not.toBe(severityCellTint("error"));
  });

  it("falls to error for an unknown tier, loud like every variant", () => {
    expect(severityRowTint("brand-new-tier")).toBe(severityRowTint("error"));
  });
});

describe("severityRowEdge", () => {
  it("marks with the tier's own colour", () => {
    expect(severityRowEdge("warning")).toContain("--warn");
    expect(severityRowEdge("fyi")).toContain("--info");
  });

  it("falls to error for an unknown tier", () => {
    expect(severityRowEdge("brand-new-tier")).toBe(severityRowEdge("error"));
  });
});
