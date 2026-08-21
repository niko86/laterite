import { describe, expect, it } from "vitest";
import { DICT_VERSIONS, EDITIONS } from "./editions";

// editions.ts is the generated SSOT for the AGS editions the UI offers (laterite-dev#529).
// Byte-level drift vs the dictionary is gated in Python
// (tests/test_web_editions_match_generator.py); this pins the runtime SHAPE the
// four consumers rely on — a non-empty edition list and the "auto"-first vocabulary.
describe("editions SSOT (laterite-dev#529)", () => {
  it("EDITIONS is the non-empty AGS edition list, without the auto sentinel", () => {
    expect(EDITIONS.length).toBeGreaterThan(0);
    expect(EDITIONS).toContain("4.2");
    expect(EDITIONS).not.toContain("auto");
  });

  it("DICT_VERSIONS is 'auto' followed by exactly the editions", () => {
    expect(DICT_VERSIONS[0]).toBe("auto");
    expect([...DICT_VERSIONS].slice(1)).toEqual([...EDITIONS]);
  });
});
