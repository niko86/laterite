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
// already-consistent role (px-3 py-2) and keep their size; radius and focus
// treatment are the same input contract as here (#408).
//
// `outline-hidden`, not `outline-none`: forced-colors mode discards the
// box-shadow ring, and outline-hidden leaves a transparent outline behind for
// it to repaint — so high contrast keeps a focus indicator (#408).
export const controlFocus =
  "outline-hidden focus-visible:[box-shadow:var(--focus-ring)] focus-visible:border-accent";

export const controlClass = `rounded-xs border border-line-strong bg-surface-raised px-2 py-1.5 text-sm text-fg ${controlFocus}`;

export const controlCompact = `rounded-xs border border-line-strong bg-surface-raised px-2 py-1 text-xs text-fg ${controlFocus}`;
