// P3 — the typed-graph builder + the generator drift guard.
import { readFileSync } from "node:fs";
import { describe, expect, it } from "vitest";
import {
  generateRegistry,
  generateTypedGraph,
  loadDictionary,
} from "../tools/generate-typed-graph.mjs";
import { LOCA, PROJ, buildAgs4, read } from "../ts/index";

describe("typed-graph builder → buildAgs4", () => {
  it("walks a typed tree into valid AGS4 that round-trips", () => {
    const proj = new PROJ({
      PROJ_ID: "P1",
      PROJ_NAME: "Demo project",
      locas: [
        new LOCA({ LOCA_ID: "BH01", LOCA_GL: 12.3 }),
        new LOCA({ LOCA_ID: "BH02", LOCA_GL: 13.0 }),
      ],
    });
    const res = buildAgs4(proj);
    expect(res.text).toMatch(/"GROUP","PROJ"/);
    expect(res.text).toMatch(/"GROUP","LOCA"/);
    // Only the SET headings are emitted (LOCA_ID, LOCA_GL) — not the full union
    // schema — so LOCA_GL is now adjacent to LOCA_ID and the build is clean.
    expect(res.text).toMatch(/"HEADING","LOCA_ID","LOCA_GL"/);
    expect(res.text).toMatch(/"DATA","BH01","12\.30"/); // 2DP canonicalised from 12.3
    expect(res.text).toMatch(/"DATA","P1","Demo project"/);
    expect(res.findings).toHaveLength(0); // sparse graph builds valid (prune + synth)

    // The emitted bytes re-parse to the built groups + the autofix-synthesized
    // metadata catalogs (UNIT/TYPE/TRAN).
    const back = read(undefined, { text: res.text });
    expect(back.groups).toEqual(["PROJ", "LOCA", "TRAN", "UNIT", "TYPE"]);
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
    const onDisk = readFileSync(
      new URL("../ts/registry.generated.ts", import.meta.url),
      "utf8",
    );
    expect(generateRegistry(groups)).toBe(onDisk);
  });

  it("typed-graph.generated.ts is up to date", () => {
    const onDisk = readFileSync(
      new URL("../ts/typed-graph.generated.ts", import.meta.url),
      "utf8",
    );
    expect(generateTypedGraph(groups)).toBe(onDisk);
  });
});
