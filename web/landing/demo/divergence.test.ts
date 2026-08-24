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
  pythonFindingCount,
} from "./divergence";
import { SEEDED, parse, emit } from "./delivery";
import type { Finding } from "./engine";
import notes from "./divergence-notes.json";
import counts from "./python-counts.json";

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
  states: {
    id: string;
    rust_rule_counts: Record<string, number>;
    python_rule_counts: Record<string, number>;
  }[];
};

/** Findings shaped as the engine hands them over, from a rule-to-count map.
 *  Only `rule` and `severity` are read by the signature, and building them
 *  here rather than running the wasm engine keeps this a test of the LOOKUP. */
const findingsFrom = (tally: Record<string, number>): Finding[] =>
  Object.entries(tally).flatMap(([rule, n]) =>
    Array.from(
      { length: n },
      () => ({ rule, severity: "error" }) as unknown as Finding,
    ),
  );

/** Clear one cell of the seeded delivery, through the demo's own emitter so
 *  the result is a delivery the page could actually be in. */
const clearCell = (group: string, heading: string) =>
  parse(
    emit(
      SEEDED.map((g) =>
        g.code === group
          ? {
              ...g,
              rows: g.rows.map((row, i) =>
                i === 0
                  ? row.map((cell, c) =>
                      g.headings[c] === heading ? "" : cell,
                    )
                  : row,
              ),
            }
          : g,
      ),
    ),
  );

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

describe("pythonFindingCount", () => {
  it("is a claim about the same python-ags4 the map was swept against", () => {
    expect(counts.python_ags4_version).toBe(map.python_ags4_version);
  });

  it("reads the seeded delivery's answer off the sweep", () => {
    // Pinned by IDENTITY against the map's own `seed` state rather than by a
    // literal: a hard-coded 4 would keep passing while the sweep moved under
    // it, which is exactly the drift this table exists to make visible.
    const seed = map.states.find((s) => s.id === "seed")!;
    const expected = Object.values(seed.python_rule_counts).reduce(
      (a, b) => a + b,
      0,
    );
    expect(
      pythonFindingCount(findingsFrom(seed.rust_rule_counts), SEEDED),
    ).toBe(expected);
  });

  it("agrees with the map on every state the sweep measured", () => {
    // The lookup IS the map, restated in a form a browser can key on. If any
    // state disagrees, the signature is not the function the sweep proved it
    // to be — and only the collision below is allowed to need a cell.
    const collided = new Set(
      counts.signatures
        .filter((e) => "when_cell_is" in e)
        .map((e) => e.signature),
    );
    // Collected and asserted once, so a failure names every state that
    // disagrees rather than the first one the loop reached.
    const wrong: string[] = [];
    for (const state of map.states) {
      const signature = Object.keys(state.rust_rule_counts)
        .sort()
        .map((rule) => `${rule}=${state.rust_rule_counts[rule]}`)
        .join("|");
      if (collided.has(signature)) continue;
      const expected = Object.values(state.python_rule_counts).reduce(
        (a, b) => a + b,
        0,
      );
      const got = pythonFindingCount(
        findingsFrom(state.rust_rule_counts),
        SEEDED,
      );
      if (got !== expected) wrong.push(`${state.id}: ${got} != ${expected}`);
    }
    expect(wrong).toEqual([]);
  });

  it("resolves the TRAN_AGS collision: the same findings, two answers", () => {
    // The one signature that is not a function of the findings alone. Clearing
    // TRAN_STAT and clearing TRAN_AGS leave laterite saying exactly the same
    // thing, and only the second earns python-ags4's extra FYI (O-53).
    const entry = counts.signatures.find((e) => "when_cell_is" in e)!;
    const tally = Object.fromEntries(
      entry.signature.split("|").map((part) => {
        const at = part.lastIndexOf("=");
        return [part.slice(0, at), Number(part.slice(at + 1))];
      }),
    );
    const findings = findingsFrom(tally);
    const viaAgs = pythonFindingCount(findings, clearCell("TRAN", "TRAN_AGS"));
    const viaStat = pythonFindingCount(
      findings,
      clearCell("TRAN", "TRAN_STAT"),
    );
    expect(viaStat).toBe(entry.python);
    expect(viaAgs).not.toBe(viaStat);
  });

  it("says nothing rather than guessing about a state the sweep never saw", () => {
    // Silence would be indistinguishable from the two engines agreeing, which
    // is the confusion the whole feature exists to remove — so the caller gets
    // null and the page says so.
    expect(
      pythonFindingCount(findingsFrom({ "AGS Format Rule 99": 3 }), SEEDED),
    ).toBeNull();
  });

  it("keys on the tiers the demo shows, not the tiers the sweep measured", () => {
    // The sweep measures laterite with FYI on so the two engines are
    // tier-comparable; the demo's validate call leaves it off. An FYI arriving
    // in the findings must not change the key, or every lookup misses at once.
    const seed = map.states.find((s) => s.id === "seed")!;
    const withFyi = [
      ...findingsFrom(seed.rust_rule_counts),
      {
        rule: "FYI (Related to Rule 16)",
        severity: "fyi",
      } as unknown as Finding,
    ];
    expect(pythonFindingCount(withFyi, SEEDED)).toBe(
      pythonFindingCount(findingsFrom(seed.rust_rule_counts), SEEDED),
    );
  });
});
