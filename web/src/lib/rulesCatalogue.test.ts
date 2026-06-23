import { describe, expect, it } from "vitest";
import { readFileSync } from "node:fs";
import { fileURLToPath } from "node:url";

// Drift gate: web/public/rules-catalogue.json — the static asset the
// RuleExplainer tool fetches — must stay a faithful copy of the single source
// of truth, rust-packages/laterite-ags4-validator/data/rules_meta.json (the
// editorial rule metadata the validator embeds and exposes via --list-rules).
//
// scripts/sync-rules.mjs copies the canonical file verbatim on every
// build/dev; this test re-checks the committed copy in CI (the web `unit` job)
// so a stale commit — or a hand-edit back to phantom rules / stale severities —
// fails loudly. Mirrors tests/test_sensitive_headings_faithful.py (the same
// generated+gated SSOT pattern this repo uses for sensitive_headings.json).

const COMMITTED = readFileSync(
  fileURLToPath(new URL("../../public/rules-catalogue.json", import.meta.url)),
  "utf8",
);
const CANONICAL = readFileSync(
  fileURLToPath(
    new URL(
      "../../../rust-packages/laterite-ags4-validator/data/rules_meta.json",
      import.meta.url,
    ),
  ),
  "utf8",
);

interface Obs {
  id: string;
  note: string;
}
interface Rule {
  rule: string;
  title: string;
  checks: string;
  severity: string;
  fixable: boolean;
  observations: Obs[];
}
interface Catalogue {
  schema_version: number;
  rules: Rule[];
}

const doc = JSON.parse(COMMITTED) as Catalogue;

// The exact set of numbered rule labels the engine emits — the 27 laterite
// implements. Rule 12 is a deliberate no-op (subsumed by 10b) and 16a folds
// into 16, so neither appears. Kept in lock-step with catalogue.rs::RULE_LABELS
// by the Rust faithfulness gate; this asserts the web copy carries exactly it.
const RULE_LABELS = [
  "1", "2", "2a", "2b", "3", "4", "5", "6", "7", "8", "9", "10a", "10b", "10c",
  "11a", "11b", "11c", "13", "14", "15", "16", "17", "18", "19", "19a", "19b",
  "20",
];

describe("rules-catalogue.json drift gate", () => {
  it("is byte-identical to the canonical rules_meta.json", () => {
    // sync-rules.mjs is a verbatim copy, so a byte mismatch means the asset is
    // stale — run `npm run sync-rules` (or any `npm run build`/`dev`).
    expect(COMMITTED).toBe(CANONICAL);
  });

  it("covers exactly the engine's 27 rule labels (no phantom 12 / 16a)", () => {
    const labels = doc.rules.map((r) => r.rule);
    expect([...labels].sort()).toEqual([...RULE_LABELS].sort());
    expect(labels).not.toContain("12");
    expect(labels).not.toContain("16a");
  });

  it("is well-formed: every rule has the fields the RuleExplainer renders", () => {
    for (const r of doc.rules) {
      expect(typeof r.rule).toBe("string");
      expect(typeof r.title).toBe("string");
      expect(typeof r.checks).toBe("string");
      // The numbered catalogue is error/mixed only — `mixed` marks a rule that
      // can also emit an FYI/Warning bucket (see catalogue.rs). `fyi` is a
      // finding severity, never a catalogue-rule severity.
      expect(["error", "mixed"]).toContain(r.severity);
      expect(typeof r.fixable).toBe("boolean");
      expect(Array.isArray(r.observations)).toBe(true);
    }
  });
});
