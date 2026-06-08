// One definition for the app's bordered form controls, so each tab stops
// drifting its own padding/size/focus treatment (the audit found selects at
// py-1 vs py-1.5, text-sm vs inherited, and focus:border-accent on some but not
// others). Compose per-instance modifiers (w-*, mono, flex-1, disabled:*)
// AROUND these — they only set the shared look.
//
//   controlClass  — the standard <select>/<input> (Source/Output, ChartBuilder,
//                   Controls, Dictionary edition, the mobile group picker, …).
//   controlCompact — dense inline controls in a text-xs row (the SqlBuilder
//                   WHERE filters, the snippet-name box, the sidebar filter).
//
// Prominent search inputs (Rule/Dictionary/Template) are a deliberately larger,
// already-consistent role (rounded-lg px-3 py-2) and stay as they are.
export const controlClass =
  "rounded border border-line-strong bg-surface-raised px-2 py-1.5 text-sm text-fg outline-none focus:border-accent";

export const controlCompact =
  "rounded border border-line-strong bg-surface-raised px-2 py-1 text-xs text-fg outline-none focus:border-accent";
