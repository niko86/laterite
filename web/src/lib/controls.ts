// The shared focus contract for the few controls that stay native: the
// prominent search inputs (Rule/Dictionary/Template — a deliberately larger,
// already-consistent px-3 py-2 role) and the SQL console's textarea. The
// bordered <select>/<input> roles this file used to define (controlClass /
// controlCompact) migrated onto the @shared/components primitives, whose
// CONTROL_CLASS carries the same look (#410); per-instance modifiers (w-*,
// mono, flex-1) still compose AROUND the shared treatment.
//
// `outline-hidden`, not `outline-none`: forced-colors mode discards the
// box-shadow ring, and outline-hidden leaves a transparent outline behind for
// it to repaint — so high contrast keeps a focus indicator (#408).
export const controlFocus =
  "outline-hidden focus-visible:[box-shadow:var(--focus-ring)] focus-visible:border-accent";
