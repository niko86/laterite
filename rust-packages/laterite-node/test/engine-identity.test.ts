// The addon can say which ENGINE it carries, not just which package it is.
//
// Since the tiers split (laterite#202) the npm package version and the engine
// version are independent numbers, so `version()` no longer answers "which rules
// ran". `engineFingerprint()` does — a build-time digest over every rule source,
// the dictionary and the rules catalogue, so it moves even when someone edits a
// rule and forgets to bump anything.
//
// This module already learned the lesson once at a different level: the wasm
// surface got a `version()` export because a compliance report HARD-CODED
// "0.5.1" and kept printing it while the workspace moved to 0.7.0. The build was
// current; only the report lied. A hardcoded fingerprint would fail the same way
// and be harder to spot, so the shape is asserted rather than assumed.
import { describe, expect, it } from "vitest";
import { engineFingerprint, engineVersion, version } from "../ts/index";
import { census } from "../ts/cli";

// `build.rs` truncates the SHA-256 to 16 hex chars.
const FINGERPRINT = /^[0-9a-f]{16}$/;

describe("engine identity", () => {
  it("reports a well-formed digest", () => {
    const fp = engineFingerprint();
    expect(fp, `fingerprint ${fp} is not 16 hex chars`).toMatch(FINGERPRINT);
  });

  it("is stable within a build", () => {
    // A value that changed per call could never be compared to anything.
    expect(engineFingerprint()).toBe(engineFingerprint());
  });

  it("distinguishes the engine version from the fingerprint", () => {
    expect(engineVersion()).toMatch(/^\d+\.\d+\.\d+/);
    expect(
      engineVersion(),
      "engineVersion and engineFingerprint returned the same string — one is wired to the wrong constant",
    ).not.toBe(engineFingerprint());
  });

  it("offers the package version and the engine version as separate doors", () => {
    // They are EQUAL today and diverge at the first bump of either tier, so this
    // cannot assert they differ. The failure it guards is a surface that only
    // ever had one door and silently answered the wrong question with it.
    expect(version()).toMatch(/^\d+\.\d+\.\d+/);
    expect(typeof engineVersion()).toBe("string");
    expect(typeof engineFingerprint()).toBe("string");
  });

  it("the npx launcher's census reports the engine it is running", () => {
    // `lat census` is the door `laterite-ags4-xcheck` uses to identity-check the
    // three launcher legs. Before it existed the cross-surface gate compared their
    // bytes without knowing whether they were the same build — and a launcher
    // driving a stale dist agrees with a current one on almost every case, so the
    // gate would have reported an identity it never checked.
    //
    // Comparing against engineFingerprint() is the load-bearing half: a hard-coded
    // digest here would satisfy a shape check forever while naming an engine this
    // launcher is not carrying.
    const c = census() as { engine: string; census_version: number };
    expect(c.engine).toBe(engineFingerprint());
    expect(
      c.census_version,
      "the engine field arrived in census schema 6; an older schema has no engine to report",
    ).toBeGreaterThanOrEqual(6);
  });
});
