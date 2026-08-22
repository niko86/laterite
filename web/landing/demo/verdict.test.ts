/* #531: the scoreboard's counting and phrasing, pinned. */

import { describe, expect, it } from "vitest";
import { readFileSync } from "node:fs";
import { fileURLToPath } from "node:url";
import { scoreboardLabel, tally } from "./verdict";
import { HERO_LINES, HERO_LINE_COUNT } from "./heroExcerpt";
import type { Finding } from "./engine";

const finding = (severity: Finding["severity"]): Finding => ({
  rule: "AGS Format Rule 8",
  line: 17,
  group: "LOCA",
  heading: null,
  dataRow: null,
  severity,
  desc: "",
});

describe("tally", () => {
  it("counts errors and warnings, and lets FYI pass the gate", () => {
    const t = tally([
      finding("error"),
      finding("error"),
      finding("warning"),
      finding("fyi"),
    ]);
    expect(t).toEqual({ errors: 2, warnings: 1 });
  });
});

describe("scoreboardLabel", () => {
  it("states counts, dropping the zero half", () => {
    expect(scoreboardLabel({ errors: 2, warnings: 1 })).toBe(
      "2 errors · 1 warning",
    );
    expect(scoreboardLabel({ errors: 1, warnings: 0 })).toBe("1 error");
    expect(scoreboardLabel({ errors: 0, warnings: 3 })).toBe("3 warnings");
  });

  it("zero of both is a verdict, not silence", () => {
    expect(scoreboardLabel({ errors: 0, warnings: 0 })).toBe("✓ valid AGS4");
  });
});

describe("the hero excerpt", () => {
  it("is the committed fixture's opening lines, byte for byte — the drift gate", () => {
    // Read from DISK, not through the ?raw import the module itself uses:
    // this is the independent path that catches a re-hardcoded excerpt or a
    // build transform mangling the bytes.
    const onDisk = readFileSync(
      fileURLToPath(new URL("./seeded-delivery.ags", import.meta.url)),
      "utf8",
    )
      .split(/\r?\n/)
      .slice(0, HERO_LINE_COUNT);
    expect([...HERO_LINES]).toEqual(onDisk);
  });
});
