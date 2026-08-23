/* The wasm boundary of engine.ts, exercised against a MOCK module — the
 * loading latch, the byte round-trips, and both branches of validateText.
 * The real wasm's behaviour on the seeded delivery is the landing e2e's
 * job; what this file owns is the plumbing between the page and whichever
 * module arrives: init exactly once, bytes in, decoded text out, and the
 * engine's error envelope passed through rather than swallowed. */

import { describe, expect, it, vi } from "vitest";

const init = vi.fn(() => Promise.resolve());
const validate = vi.fn();
const compute_fixes = vi.fn();
const apply_fixes = vi.fn();

vi.mock("../../src/wasm/ags4_wasm", () => ({
  default: init,
  validate,
  compute_fixes,
  apply_fixes,
}));

import {
  applyFixesText,
  computeFixesText,
  engine,
  isLoaded,
  validateText,
} from "./engine";

describe("the loading latch", () => {
  it("arms once: not loaded before, loaded after, init called exactly once", async () => {
    expect(isLoaded()).toBe(false);
    const [a, b] = await Promise.all([engine(), engine()]);
    expect(a).toBe(b);
    expect(init).toHaveBeenCalledTimes(1);
    expect(isLoaded()).toBe(true);
    // The resident fast path: a third call resolves without re-initing.
    await engine();
    expect(init).toHaveBeenCalledTimes(1);
  });
});

describe("validateText", () => {
  it("hands the engine BYTES and flattens what comes back", async () => {
    validate.mockReturnValueOnce({
      ok: false,
      findings: [
        {
          rule: "AGS Format Rule 8",
          items: [
            {
              line: 17,
              group: "LOCA",
              heading: "LOCA_GL",
              data_row: 1,
              desc: "bad",
            },
          ],
        },
      ],
    });
    const report = await validateText("hello");
    expect(validate).toHaveBeenCalledWith(new TextEncoder().encode("hello"));
    expect(report.ok).toBe(false);
    expect(report.findings).toEqual([
      {
        rule: "AGS Format Rule 8",
        line: 17,
        group: "LOCA",
        heading: "LOCA_GL",
        dataRow: 1,
        severity: "error",
        desc: "bad",
      },
    ]);
  });

  it("passes the engine's error envelope through instead of swallowing it", async () => {
    validate.mockReturnValueOnce({
      error: { kind: "encoding", message: "not AGS" },
    });
    const report = await validateText("junk");
    expect(report).toEqual({
      ok: false,
      findings: [],
      error: { kind: "encoding", message: "not AGS" },
    });
  });
});

describe("the fix round-trip", () => {
  it("computeFixesText returns the engine's own fix records", async () => {
    const fixes = [{ id: 1 }];
    compute_fixes.mockReturnValueOnce(fixes);
    expect(await computeFixesText("text")).toBe(fixes);
    expect(compute_fixes).toHaveBeenCalledWith(
      new TextEncoder().encode("text"),
    );
  });

  it("applyFixesText decodes the engine's bytes back to text", async () => {
    apply_fixes.mockReturnValueOnce(new TextEncoder().encode("fixed"));
    expect(await applyFixesText("text", [])).toBe("fixed");
    expect(apply_fixes).toHaveBeenCalledWith(
      new TextEncoder().encode("text"),
      null,
      [],
    );
  });
});
