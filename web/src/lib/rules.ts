// Small UI helpers for rendering rule labels. The engine keys findings
// by the full label ("AGS Format Rule 8"); the UI shows a short form and
// needs a stable DOM anchor per rule for the legend → section jump.

export function shortRule(rule: string): string {
  return rule.replace(/^AGS Format Rule /, "Rule ");
}

export function ruleAnchor(rule: string): string {
  return (
    "rule-" +
    rule
      .toLowerCase()
      .replace(/[^a-z0-9]+/g, "-")
      .replace(/^-+|-+$/g, "")
  );
}
