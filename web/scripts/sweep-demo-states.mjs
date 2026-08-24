/* Enumerate the states the landing demo can reach one lever from its seed, and
 * emit each one through the demo's OWN emitter (#659).
 *
 * The point of the sweep is a claim about what two engines say, so the files it
 * validates have to be files the demo can actually produce. That is why this
 * loads `landing/demo/delivery.ts` through Vite's module runner rather than
 * reimplementing the model: `emit()` here is the same function the page's
 * textarea is filled from, resolved the same way, `?raw` seed and all. A copy
 * would make the sweep a claim about the copy. `assertSeedRoundTrips` holds
 * that to something: it compares `emit(SEEDED)` against the seed file on disk
 * and stops the run if they differ.
 *
 * ## What "reachable" means, and why it is narrower than "in the file"
 *
 * A lever is enumerated only where a reader can actually pull it. The demo
 * renders an editable table for the groups in its own `DEMO_GROUPS` schema and
 * for no others, so the file's remaining groups are visible but not mutable:
 * no cell edit, no row add, no group delete. Enumerating them would put states
 * in the map that nothing can produce, which is the same failure as leaving
 * reachable ones out — a map is only worth having if membership means
 * something. `EXCLUDED_GROUPS` records them and why, and the manifest carries
 * it, because a silent exclusion is indistinguishable from an oversight.
 *
 * ## What "every state" can honestly mean
 *
 * Not the product of the levers. A group is present or not and a row is
 * present or not, so the reachable structures alone are 2^groups x 2^rows over
 * the seeded delivery — a number with ten digits in it, each state costing a
 * python-ags4 subprocess. Nothing exhausts that.
 *
 * What IS enumerable is every state ONE lever from the seed, which is what
 * this produces, plus a small NAMED set of multi-lever sequences: the ones the
 * demo's own teach loops walk a reader through. The bound and the levers left
 * out are reported on every run and carried in the manifest. That is not a
 * footnote — a map that quietly stopped at depth 1 would read as "the demo has
 * been swept" when it means something much narrower.
 *
 * ## Why cell VALUES are classes rather than values
 *
 * `setCell` takes free text, so the values are unbounded — dropping the
 * editable whole-file pane bounded the FILE, not the CELL. What is bounded is
 * what a value can MEAN to the validator, and the findings key off that: two
 * values in the same class produce the same rule set. Each class is named for
 * the rule it exists to reach, and applicability is derived from the demo's own
 * schema (KEY, REQUIRED, the declared TYPE) rather than hardcoded per cell.
 * Class values are generated to be VALID for the heading's declared type
 * wherever the class is not about the type, so a class reaches the rule it
 * claims instead of tripping Rule 8 on the way.
 */

import { createServer } from "vite";
import { mkdirSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";

const HERE = dirname(fileURLToPath(import.meta.url));
const WEB = resolve(HERE, "..");
const SEED_FILE = join(WEB, "landing", "demo", "seeded-delivery.ags");

/** A value that is well-formed for `type` and unlike anything in the seed, so
 *  a class about parentage or abbreviations reaches THAT rule rather than
 *  tripping Rule 8 on its way there. */
function typedValue(type) {
  if (type === "DT") return "1999-01-01";
  const dp = /^(\d)DP$/.exec(type);
  if (dp) return (999).toFixed(Number(dp[1]));
  if (type === "SCI") return "9.99E+02";
  return "NO-SUCH-VALUE";
}

/** The value classes, each named for the rule it exists to reach. The control
 *  is the seed itself, which is emitted and validated like any other state. */
const CLASSES = [
  {
    id: "blank",
    rule: "10b (REQUIRED must be present) / 10a (the KEY tuple identifies the row)",
    applies: () => true,
    value: () => "",
  },
  {
    id: "non-ascii",
    rule: "1 (the file must be ASCII)",
    applies: (h) => h.type === "X" || h.type === "XN",
    value: () => "Ceci n’est pas ASCII",
  },
  {
    id: "wrong-for-type",
    rule: "8 (the value must match its declared TYPE)",
    // Only where the TYPE actually constrains the text. ID and X accept
    // anything, so there is no "wrong" value to write into them.
    applies: (h) => !["ID", "X", "XN", "PA"].includes(h.type),
    value: () => "not-a-number",
  },
  {
    id: "unmatched-parent-key",
    rule: "10c (a child row must match a parent row)",
    applies: (h, ctx) => h.key && ctx.parentCarries,
    value: (h) => typedValue(h.type),
  },
  {
    id: "unlisted-abbreviation",
    rule: "16 (a PA value must be listed in ABBR)",
    applies: (h) => h.type === "PA",
    value: () => "ZZZ",
  },
];

/** The demo's mutation surface that this does NOT turn into states, each with
 *  the reason. A reader checking this against `store.ts` will find them
 *  missing; better they find them named than conclude the sweep overlooked
 *  them. `applyGroupFixes` is the one that matters — a lever a reader really
 *  pulls, not a combinatorial corner. */
const LEVERS_NOT_ENUMERATED = [
  {
    lever: "applyGroupFixes",
    why:
      "it applies the ENGINE's own fixes to one group, so reproducing it " +
      "faithfully means running the wasm engine inside the enumerator rather " +
      "than the pure model. Reaching for `lat fix` instead would validate a " +
      "DIFFERENT state, because the demo applies one group's share of the " +
      "fixes and not the whole file's. Named rather than approximated: a " +
      "state produced by a near-miss of the demo's own lever is exactly the " +
      "plausible-but-unreachable entry this sweep exists to keep out",
  },
  {
    lever: "undo / redo / reset",
    why:
      "navigation, not mutation — each returns to a state some other lever " +
      "already produced, so they add nothing to enumerate",
  },
  {
    lever: "restoreGroup",
    why:
      "identity on the seed, since nothing is deleted yet, so it has no " +
      "depth-1 state. It is exercised by `assertDeleteRestoreRoundTrips` " +
      "instead, which is the stronger claim: not that it reaches a new " +
      "state, but that it returns EXACTLY to the one it started from",
  },
];

/** The multi-lever states the demo's own teach loops walk a reader through.
 *  Named, not generated: depth 2 across the whole space is combinatorial, and
 *  most pairs are two unrelated findings sitting side by side. These are the
 *  ones the page actually leads someone into. */
const SEQUENCES = [
  {
    id: "seq-delete-parent-row-then-child-row",
    why: "the orphan teach loop: remove a LOCA, then a SAMP that hung off it",
    steps: [
      { op: "deleteRow", group: "LOCA", row: 1 },
      { op: "deleteRow", group: "SAMP", row: 0 },
    ],
  },
  {
    id: "seq-add-row-then-blank-its-key",
    why: "an appended row inherits its parent KEY; blanking it strands the row",
    steps: [
      { op: "addRow", group: "SAMP" },
      { op: "setCellKey", group: "SAMP", row: -1, value: "" },
    ],
  },
  {
    id: "seq-duplicate-a-whole-key-tuple",
    why:
      "Rule 10a is about the TUPLE, so copying one cell cannot reach it in a " +
      "group whose key is several headings; this copies the whole tuple of " +
      "row 1 onto row 2",
    steps: [{ op: "copyKeyTuple", group: "SAMP", from: 0, to: 1 }],
  },
  {
    id: "seq-delete-a-group-and-use-what-it-defined",
    why: "Rule 16 with nothing left to check against, which is a different answer",
    steps: [
      { op: "deleteGroup", group: "LLPL" },
      { op: "setCellType", group: "SAMP", type: "PA", value: "ZZZ" },
    ],
  },
];

async function main() {
  const outDir = process.argv[2];
  if (!outDir) {
    console.error("usage: sweep-demo-states.mjs <out-dir>");
    process.exit(2);
  }

  const server = await createServer({
    root: join(WEB, "landing"),
    configFile: join(WEB, "landing", "vite.config.ts"),
    server: { middlewareMode: true },
    logLevel: "error",
  });
  let delivery, schema;
  try {
    delivery = await server.ssrLoadModule("/demo/delivery.ts");
    schema = await server.ssrLoadModule("/demo/schema.ts");
  } finally {
    await server.close();
  }

  const {
    SEEDED,
    emit,
    setCell,
    addRow,
    deleteRow,
    deleteGroup,
    restoreGroup,
  } = delivery;
  const { DEMO_GROUPS, keyHeadings } = schema;
  const groupOf = (code) => DEMO_GROUPS.find((g) => g.code === code);

  // The demo renders an editable table only for the groups in its own schema,
  // so that list IS the answer to "what can a reader touch".
  const editable = new Set(DEMO_GROUPS.map((g) => g.code));
  const EXCLUDED_GROUPS = SEEDED.filter((g) => !editable.has(g.code)).map(
    (g) => ({
      group: g.code,
      why:
        "present in the delivery but not in the demo's own group schema, so " +
        "the page renders no editable table for it: no cell edit, no row " +
        "add, no group delete. Enumerating it would put states in the map " +
        "that nothing can produce",
    }),
  );

  // The claim the whole sweep rests on: emit() is the demo's real serializer,
  // not a lookalike. If it stopped agreeing with the checked-in seed, every
  // state below would be a claim about something the page never shows.
  const seedText = emit(SEEDED);
  const onDisk = readFileSync(SEED_FILE, "utf8");
  if (seedText !== onDisk) {
    throw new Error(
      "the demo's emit() no longer reproduces landing/demo/seeded-delivery.ags " +
        "byte for byte, so no state below is evidence about what the page shows",
    );
  }

  // restoreGroup's real claim is not that it reaches a new state but that it
  // returns to EXACTLY the one it started from. Asserted here rather than
  // filed as a sequence that produced nothing, which is indistinguishable
  // from a sequence that silently broke.
  for (const g of SEEDED) {
    if (!editable.has(g.code)) continue;
    const round = emit(restoreGroup(deleteGroup(SEEDED, g.code), g.code));
    if (round !== seedText) {
      throw new Error(
        `deleting ${g.code} and restoring it does not return to the seed, so ` +
          "the demo's undo story is broken independently of anything here",
      );
    }
  }

  rmSync(outDir, { recursive: true, force: true });
  mkdirSync(outDir, { recursive: true });

  const states = [];
  const skipped = [];
  const write = (id, lever, text, detail) => {
    // A lever that changed nothing reached no new state, and recording it
    // would fill the map with copies of the seed.
    if (text === seedText && lever !== "seed") {
      skipped.push({ id, why: "the lever left the delivery unchanged" });
      return;
    }
    writeFileSync(join(outDir, `${id}.ags`), text, "utf8");
    states.push({ id, lever, ...detail });
  };

  write("seed", "seed", seedText, { why: "the delivery as the page loads" });

  for (const g of SEEDED) {
    if (!editable.has(g.code)) continue;
    write(
      `delete-group-${g.code}`,
      "deleteGroup",
      emit(deleteGroup(SEEDED, g.code)),
      { group: g.code },
    );
    const parent = groupOf(g.code)?.parent ?? null;
    write(
      `add-row-${g.code}`,
      "addRow",
      emit(addRow(SEEDED, g.code, parent, keyHeadings(g.code))),
      { group: g.code, parent },
    );
    for (let r = 0; r < g.rows.length; r += 1) {
      write(
        `delete-row-${g.code}-${r}`,
        "deleteRow",
        emit(deleteRow(SEEDED, g.code, r)),
        { group: g.code, row: r },
      );
    }
  }

  for (const g of SEEDED) {
    const meta = groupOf(g.code);
    if (!meta) continue;
    const parentMeta = meta.parent ? groupOf(meta.parent) : null;
    for (let r = 0; r < g.rows.length; r += 1) {
      for (let c = 0; c < g.headings.length; c += 1) {
        const h = meta.headings.find((x) => x.name === g.headings[c]);
        if (!h) continue;
        const ctx = {
          parentCarries: Boolean(
            parentMeta?.headings.some((x) => x.name === h.name),
          ),
        };
        for (const cls of CLASSES) {
          if (!cls.applies(h, ctx)) continue;
          const value = cls.value(h, ctx);
          if (value === g.rows[r][c]) {
            skipped.push({
              id: `set-${g.code}-${r}-${h.name}-${cls.id}`,
              why: "the class value equals what the cell already holds",
            });
            continue;
          }
          write(
            `set-${g.code}-${r}-${h.name}-${cls.id}`,
            "setCell",
            emit(setCell(SEEDED, g.code, r, c, value)),
            {
              group: g.code,
              row: r,
              heading: h.name,
              class: cls.id,
              rule: cls.rule,
            },
          );
        }
      }
    }
  }

  // One entry per step verb. A cascade of `else if` on the same string has to
  // be extended in step with SEQUENCES, and a mistyped verb falls through it
  // silently, leaving a sequence that quietly did less than it says. A lookup
  // makes an unknown verb an error.
  const STEP = {
    deleteRow: (d, step) => deleteRow(d, step.group, step.row),
    deleteGroup: (d, step) => deleteGroup(d, step.group),
    restoreGroup: (d, step) => restoreGroup(d, step.group),
    addRow: (d, step) =>
      addRow(
        d,
        step.group,
        groupOf(step.group)?.parent ?? null,
        keyHeadings(step.group),
      ),
    // A negative row counts from the end, which is how a step names the row
    // the step before it appended.
    setCellKey: (d, step) => {
      const g = d.find((x) => x.code === step.group);
      const col = g ? g.headings.indexOf(keyHeadings(step.group)[0]) : -1;
      const row = step.row < 0 ? (g?.rows.length ?? 0) + step.row : step.row;
      return g && col >= 0 ? setCell(d, step.group, row, col, step.value) : d;
    },
    setCellType: (d, step) => {
      const g = d.find((x) => x.code === step.group);
      const named = groupOf(step.group)?.headings.find(
        (x) => x.type === step.type,
      );
      const col = g && named ? g.headings.indexOf(named.name) : -1;
      return g && col >= 0 ? setCell(d, step.group, 0, col, step.value) : d;
    },
    // Rule 10a is about the whole KEY TUPLE, so one cell cannot reach it in a
    // group keyed on several headings. This is a reader doing what a reader
    // does: copying a row's identity onto another row, one cell at a time.
    copyKeyTuple: (d, step) => {
      const g = d.find((x) => x.code === step.group);
      if (!g) return d;
      let next = d;
      for (const key of keyHeadings(step.group)) {
        const col = g.headings.indexOf(key);
        if (col < 0) continue;
        next = setCell(next, step.group, step.to, col, g.rows[step.from][col]);
      }
      return next;
    },
  };

  for (const seq of SEQUENCES) {
    let d = SEEDED;
    for (const step of seq.steps) {
      const apply = STEP[step.op];
      if (!apply) throw new Error(`${seq.id}: unknown step verb ${step.op}`);
      d = apply(d, step);
    }
    write(seq.id, "sequence", emit(d), {
      why: seq.why,
      steps: seq.steps.length,
    });
  }

  const manifest = {
    schema: 1,
    seed: "landing/demo/seeded-delivery.ags",
    emitter:
      "landing/demo/delivery.ts::emit (loaded through Vite, not reimplemented; " +
      "asserted byte-identical against the seed file before anything is emitted)",
    depth: {
      exhaustive_to: 1,
      meaning:
        "every state one lever from the seed, over the groups the demo renders as editable: each group deleted, each row deleted, a row added to each group, and each cell set to each value class that can apply to it",
      beyond:
        "multi-lever states are covered only by the named sequences below; the full product of the levers is 2^groups x 2^rows over the seeded delivery and is NOT swept",
      sequences: SEQUENCES.map((s) => ({ id: s.id, why: s.why })),
      levers_not_enumerated: LEVERS_NOT_ENUMERATED,
      groups_not_enumerated: EXCLUDED_GROUPS,
    },
    classes: CLASSES.map((c) => ({ id: c.id, rule: c.rule })),
    counts: {
      states: states.length,
      by_lever: states.reduce((acc, s) => {
        acc[s.lever] = (acc[s.lever] ?? 0) + 1;
        return acc;
      }, {}),
      skipped: skipped.length,
    },
    skipped,
    states,
  };
  writeFileSync(
    join(outDir, "manifest.json"),
    `${JSON.stringify(manifest, null, 2)}\n`,
    "utf8",
  );
  console.error(
    `sweep-demo-states: ${states.length} state(s) emitted to ${outDir}; ` +
      `${skipped.length} lever(s) skipped as no-ops; ` +
      `${EXCLUDED_GROUPS.length} group(s) excluded as not reader-editable; ` +
      "exhaustive to depth 1 only",
  );
}

await main();
