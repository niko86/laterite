// The controls that stay native after the primitives migration (#410), and
// the shared focus contract they ride.
//
//   searchControl — the prominent search inputs (Rule/Dictionary/Template): a
//                   deliberately larger, already-consistent px-3 py-2 role the
//                   compact Input primitive doesn't cover (#408). Width and
//                   flex modifiers (w-full, min-w-0 flex-1) compose AROUND it.
//   controlFocus  — the focus treatment alone, for the SQL console's textarea.
//                   The bordered <select>/<input> roles that used to live here
//                   (controlClass/controlCompact) lost their last consumers to
//                   @shared/components' CONTROL_CLASS and are retired.
//
// `outline-hidden`, not `outline-none`: forced-colors mode discards the
// box-shadow ring, and outline-hidden leaves a transparent outline behind for
// it to repaint — so high contrast keeps a focus indicator (#408).
export const controlFocus =
  "outline-hidden focus-visible:[box-shadow:var(--focus-ring)] focus-visible:border-accent";

export const searchControl = `rounded-xs border border-line-strong bg-surface-raised px-3 py-2 text-sm text-fg ${controlFocus} placeholder:text-fg-dim`;
