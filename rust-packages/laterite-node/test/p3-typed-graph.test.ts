// P3 — the typed-graph builder + the generator drift guard.
import { readFileSync } from "node:fs";
import { describe, expect, it } from "vitest";
import {
  generateRegistry,
  generateTypedGraph,
  loadDictionary,
} from "../tools/generate-typed-graph.mjs";
import { LOCA, PROJ, emitAgs4, read } from "../ts/index";

describe("typed-graph builder → emitAgs4", () => {
  it("walks a typed tree into valid AGS4 that round-trips", () => {
    const proj = new PROJ({
      PROJ_ID: "P1",
      PROJ_NAME: "Demo project",
      locas: [
        new LOCA({ LOCA_ID: "BH01", LOCA_GL: 12.3 }),
        new LOCA({ LOCA_ID: "BH02", LOCA_GL: 13.0 }),
      ],
    });
    const res = emitAgs4(proj);
    expect(res.text).toMatch(/"GROUP","PROJ"/);
    expect(res.text).toMatch(/"GROUP","LOCA"/);
    // Every declared heading is emitted (LOCA_GL is column 7, not adjacent to
    // LOCA_ID), so assert the row + the canonicalised 2DP value separately.
    expect(res.text).toMatch(/"DATA","BH01",/);
    expect(res.text).toMatch(/"12\.30"/); // 2DP canonicalised from 12.3
    expect(res.text).toMatch(/"DATA","P1","Demo project"/);

    // The emitted bytes re-parse to the built groups.
    const back = read(undefined, { text: res.text });
    expect(back.groups).toEqual(["PROJ", "LOCA"]);
    expect(back.table("LOCA").numRows).toBe(2);
    expect(back.table("LOCA").getChild("LOCA_GL")!.get(0)).toBe(12.3);
  });

  it("a class carries its static code and constructs from a partial", () => {
    expect(PROJ.code).toBe("PROJ");
    expect(LOCA.code).toBe("LOCA");
    const loca = new LOCA({ LOCA_ID: "BH01" });
    expect(loca.LOCA_ID).toBe("BH01");
    expect(loca.LOCA_GL).toBeNull(); // unset heading
    expect(loca.samps).toEqual([]); // child array default
  });
});

describe("generator drift guard", () => {
  // Mirrors test_pyi_stubs_match_generator.py: re-run the generator and assert
  // the committed files are byte-identical — fails loud on a dictionary edit.
  const groups = loadDictionary();

  it("registry.generated.ts is up to date", () => {
    const onDisk = readFileSync(new URL("../ts/registry.generated.ts", import.meta.url), "utf8");
    expect(generateRegistry(groups)).toBe(onDisk);
  });

  it("typed-graph.generated.ts is up to date", () => {
    const onDisk = readFileSync(new URL("../ts/typed-graph.generated.ts", import.meta.url), "utf8");
    expect(generateTypedGraph(groups)).toBe(onDisk);
  });
});
