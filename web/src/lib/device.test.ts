import { afterEach, describe, expect, it, vi } from "vitest";
import { isLowEndDevice } from "./device";

// The predicate the whole speculation policy turns on: it decides whether a
// visitor gets the tier-2 engine and DuckDB primed on idle (prefetch.ts), and
// whether Explore asks before downloading 36 MB (EngineGate). Both callers use
// it as a speculate-at-all gate, so a wrong answer either wastes tens of MB on a
// phone or makes every first click slow on a workstation.
//
// The asymmetry below is the design: unknown means CAPABLE. `deviceMemory` and
// the Network Information API are Chromium-only, so treating "didn't say" as
// low-end would down-tier every Firefox and Safari user on the strength of a
// missing API rather than a slow machine.

interface Nav {
  saveData?: boolean;
  effectiveType?: string;
  deviceMemory?: number;
  cores?: number;
}

/** Present this device to the predicate and return its verdict. */
function lowEndWith({
  saveData,
  effectiveType,
  deviceMemory,
  cores,
}: Nav): boolean {
  vi.stubGlobal("navigator", {
    hardwareConcurrency: cores,
    deviceMemory,
    connection:
      (saveData ?? effectiveType) ? { saveData, effectiveType } : undefined,
  });
  return isLowEndDevice();
}

afterEach(() => {
  vi.unstubAllGlobals();
});

describe("isLowEndDevice", () => {
  it("reads a capable machine as capable", () => {
    expect(lowEndWith({ cores: 8, deviceMemory: 8, effectiveType: "4g" })).toBe(
      false,
    );
  });

  it("treats a browser that reports nothing as capable", () => {
    // Firefox and Safari expose neither `deviceMemory` nor `connection`. Only a
    // POSITIVE low-end reading down-tiers, so silence must not.
    expect(lowEndWith({})).toBe(false);
  });

  it("treats Data Saver as low-end whatever the hardware says", () => {
    expect(lowEndWith({ saveData: true, cores: 16, deviceMemory: 8 })).toBe(
      true,
    );
  });

  it.each(["2g", "slow-2g", "3g"])(
    "treats a %s link as low-end",
    (effectiveType) => {
      expect(lowEndWith({ effectiveType, cores: 16, deviceMemory: 8 })).toBe(
        true,
      );
    },
  );

  it("does not read 4g as low-end", () => {
    expect(
      lowEndWith({ effectiveType: "4g", cores: 16, deviceMemory: 8 }),
    ).toBe(false);
  });

  it("treats ≤ 2 GB of RAM as low-end", () => {
    // deviceMemory reports 0.25 | 0.5 | 1 | 2 | 4 | 8 — so `< 4` is the ≤ 2 GB
    // boundary, and 4 itself must stay on the capable side of it.
    expect(lowEndWith({ deviceMemory: 2, cores: 16 })).toBe(true);
    expect(lowEndWith({ deviceMemory: 4, cores: 16 })).toBe(false);
  });

  it("treats ≤ 2 logical cores as low-end", () => {
    expect(lowEndWith({ cores: 2, deviceMemory: 8 })).toBe(true);
    expect(lowEndWith({ cores: 4, deviceMemory: 8 })).toBe(false);
  });
});
