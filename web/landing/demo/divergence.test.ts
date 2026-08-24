/* #660: what the demo says about the other engine, and where it gets it.
 *
 * These pin the CONTRACT between the generated notes and the page, not the
 * wording. A note's prose is editorial and will be rewritten; which finding it
 * attaches to, and which state makes it appear, are the parts that go wrong
 * silently — a note keyed to a rule nothing raises never shows, and neither
 * does a panel keyed to a cell value nothing produces.
 */

import { describe, expect, it } from "vitest";
import { readFileSync } from "node:fs";
import { fileURLToPath } from "node:url";
import {
  PYTHON_AGS4_VERSION,
  divergenceForRule,
  divergencesTheyRaise,
} from "./divergence";
import { SEEDED, parse, emit } from "./delivery";
import notes from "./divergence-notes.json";

const map = JSON.parse(
  readFileSync(fileURLToPath(new URL("./state-map.json", import.meta.url)), {
    encoding: "utf-8",
  }),
) as {
  python_ags4_version: string;
  difference_shapes: {
    rust_only: string[];
    python_only: string[];
    triage: string;
  }[];
};

describe("the notes cover the map they are drawn from", () => {
  it("has a note for every difference shape the sweep found", () => {
    // By IDENTITY, not by count: a decremented number passes while pointing at
    // the wrong records. The shapes with no difference at all are the ordinary
    // case and have nothing to explain.
    const shapes = map.difference_shapes
      .filter((s) => s.rust_only.length || s.python_only.length)
      .map((s) => s.triage)
      .sort();
    expect(notes.notes.map((n) => n.observation).sort()).toEqual(shapes);
  });

  it("is a claim about the same python-ags4 the map was swept against", () => {
    expect(PYTHON_AGS4_VERSION).toBe(map.python_ags4_version);
  });

  it("gives every note a side the page knows how to render", () => {
    for (const note of notes.notes)
      expect(["ours", "theirs", "tier"]).toContain(note.side);
  });
});

describe("divergenceForRule", () => {
  it("explains the declined parentage check on the finding that carries it", () => {
    const note = divergenceForRule("Warning (Related to Rule 10c)");
    expect(note?.observation).toBe("O-52");
    expect(note?.side).toBe("ours");
  });

  it("says the unrecognised-edition warning is a TIER difference, not a silence", () => {
    // The map recorded this as one python-ags4 "does not report at all" until
    // both engines were compared unfiltered (#671). If this ever reads "ours"
    // again, the filter is back.
    const note = divergenceForRule("Warning (Related to Rule 14)");
    expect(note?.observation).toBe("O-45");
    expect(note?.side).toBe("tier");
  });

  it("says nothing about a rule the two engines agree on", () => {
    expect(divergenceForRule("AGS Format Rule 8")).toBeUndefined();
    expect(divergenceForRule("AGS Format Rule 16")).toBeUndefined();
  });
});

describe("divergencesTheyRaise", () => {
  it("is silent on the delivery as seeded", () => {
    expect(divergencesTheyRaise(SEEDED)).toEqual([]);
  });

  it("appears once TRAN_AGS is cleared — the one state where they report and we do not", () => {
    const cleared = parse(
      emit(
        SEEDED.map((g) =>
          g.code === "TRAN"
            ? {
                ...g,
                rows: g.rows.map((row, i) =>
                  i === 0
                    ? row.map((cell, c) =>
                        g.headings[c] === "TRAN_AGS" ? "" : cell,
                      )
                    : row,
                ),
              }
            : g,
        ),
      ),
    );
    const raised = divergencesTheyRaise(cleared);
    expect(raised.map((n) => n.observation)).toEqual(["O-53"]);
  });

  it("does not fire on a DIFFERENT cell being cleared", () => {
    const cleared = SEEDED.map((g) =>
      g.code === "TRAN"
        ? {
            ...g,
            rows: g.rows.map((row, i) =>
              i === 0
                ? row.map((cell, c) =>
                    g.headings[c] === "TRAN_PROD" ? "" : cell,
                  )
                : row,
            ),
          }
        : g,
    );
    expect(divergencesTheyRaise(cleared)).toEqual([]);
  });

  it("does not fire when the TRAN group is gone entirely", () => {
    // A deleted group is a different state with a different answer, and
    // indexing into a group that is not there must not throw.
    expect(
      divergencesTheyRaise(SEEDED.filter((g) => g.code !== "TRAN")),
    ).toEqual([]);
  });
});
