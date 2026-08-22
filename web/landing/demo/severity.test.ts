/* The severity grammar (#526). Pure: the engine names a severity, these map it
 * to one look — the UI never decides how bad something is.
 */

import { describe, expect, it } from "vitest";
import { severityCellTint, severityTint, worstSeverity } from "./severity";
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
