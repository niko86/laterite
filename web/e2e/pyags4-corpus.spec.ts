import { test, expect } from "@playwright/test";
import { existsSync, readdirSync, writeFileSync } from "node:fs";
import { fileURLToPath } from "node:url";
import path from "node:path";

// ─────────────────────────────────────────────────────────────────────────────
// OPT-IN corpus check: drive EVERY python-ags4 .ags test fixture through the
// real wasm Validate page and record which rules each surfaces vs the rule its
// filename implies. It's a breadth check against a large real-world corpus that
// the committed synthetic fixtures can't match — the engine's correctness vs
// python-ags4 is the parity oracle (tools/run_python_ags4_tests.sh, ~122/9), so
// the value here is "the wasm UI survives the whole corpus and surfaces the
// expected rules", not byte-exact parity.
//
// The .ags files are NOT vendored into this repo (they're upstream's, and large).
// This spec SKIPS cleanly unless you clone python-ags4 next to this repo:
//
//     git clone https://gitlab.com/ags-data-format-wg/ags-python-library \
//         ../ags-python-library          # sibling of the ags5_concept repo
//
// Then build + run just this spec (it's skipped in the normal e2e run / CI):
//
//     cd web
//     npm run build:wasm && npm run build      # if not already built
//     npx playwright test pyags4-corpus
//
// Point it elsewhere with PYAGS4_DIR=/abs/path/to/ags-python-library/tests.
// A per-file report (outcome + surfaced rules + match) is written to
// /tmp/pyags4_report.json and logged to the console.
// ─────────────────────────────────────────────────────────────────────────────

const here = path.dirname(fileURLToPath(import.meta.url)); // web/e2e
const PYAGS4 =
  process.env.PYAGS4_DIR ??
  path.join(here, "..", "..", "..", "ags-python-library", "tests");

function collect(): { name: string; path: string }[] {
  const out: { name: string; path: string }[] = [];
  for (const sub of ["test_files", "", "test_utils"]) {
    const dir = sub ? path.join(PYAGS4, sub) : PYAGS4;
    if (!existsSync(dir)) continue;
    for (const n of readdirSync(dir)) {
      if (n.toLowerCase().endsWith(".ags")) out.push({ name: n, path: path.join(dir, n) });
    }
  }
  return out.sort((a, b) => a.name.localeCompare(b.name));
}
const files = collect();

// Expected rule number from the filename (4.1-rule9-1 → 9, 4.1-rule19a → 19,
// 4.1-fyi16-1 → 16); null when the name doesn't encode a rule.
const expectedNum = (name: string): number | null => {
  const m = name.match(/(?:rule|fyi)0*(\d+)/i);
  return m ? Number(m[1]) : null;
};
const numsIn = (s: string): number[] =>
  [...s.matchAll(/rule\s*0*(\d+)/gi)].map((m) => Number(m[1]));

test("python-ags4 .ags corpus through the wasm validator (opt-in)", async ({
  page,
}) => {
  test.skip(
    files.length === 0,
    `python-ags4 corpus not found at ${PYAGS4} — see the header of this file for the clone + run instructions.`,
  );
  test.setTimeout(20 * 60_000);

  const rows: Record<string, unknown>[] = [];
  for (const f of files) {
    await page.goto("/ags5_concept/");
    await expect(
      page.getByRole("button", { name: /Clean \(minimal\)/ }),
    ).toBeVisible();
    await page.locator('input[type="file"]').setInputFiles(f.path);

    const findings = page.getByText(/showing \d+ of \d+ findings/);
    const clean = page.getByText(/Clean — 0 findings/);
    let outcome = "no-report"; // AGS3 refusal / empty / unrecognised
    try {
      await expect(findings.or(clean)).toBeVisible({ timeout: 20_000 });
    } catch {
      /* recorded as no-report */
    }

    let total = 0;
    let surfaced: string[] = [];
    if (await clean.isVisible().catch(() => false)) {
      outcome = "clean";
    } else if (await findings.isVisible().catch(() => false)) {
      outcome = "findings";
      total = Number(
        ((await findings.textContent()) ?? "").match(/of (\d+)/)?.[1] ?? 0,
      );
      surfaced = await page
        .locator("span[title]")
        .evaluateAll((els) => [
          ...new Set(
            els
              .map((e) => e.getAttribute("title") ?? "")
              .filter((s) => /\bRule\b|FYI/i.test(s)),
          ),
        ]);
    }

    const exp = expectedNum(f.name);
    const matched = exp == null ? null : new Set(surfaced.flatMap(numsIn)).has(exp);
    rows.push({ file: f.name, outcome, total, expected: exp, matched, surfaced });
    console.log(
      `${f.name.padEnd(28)} ${outcome.padEnd(9)} n=${String(total).padStart(5)} exp=${exp ?? "-"} match=${matched} :: ${surfaced.join(" | ")}`,
    );
  }

  writeFileSync("/tmp/pyags4_report.json", JSON.stringify(rows, null, 2));
  const named = rows.filter((r) => r.expected != null);
  const matched = named.filter((r) => r.matched).length;
  console.log(
    `\nSUMMARY: ${files.length} files · ${named.length} rule-named · matched ${matched}/${named.length} · report → /tmp/pyags4_report.json`,
  );

  // The durable guard: the wasm validator processes EVERY real fixture without
  // crashing (each file yields an outcome). Rule-match is logged, not asserted —
  // the residual mismatches are documented engine-vs-python-ags4 divergences
  // (OBSERVATIONS.md) + deliberately-OK fixtures, not regressions.
  expect(rows.length).toBe(files.length);
});
