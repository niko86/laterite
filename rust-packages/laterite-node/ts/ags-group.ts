// The marker base for the generated typed-graph classes. In its OWN module (no
// imports) so the generated file can extend it without a circular dependency —
// `typed-graph.ts` re-exports both. `buildAgs4` uses `instanceof AgsGroup` to
// recognise a typed tree root (minification-safe).
export abstract class AgsGroup {
  // Nominal brand so the marker base is a distinct (non-empty) type; `declare`
  // makes it type-only — no runtime field is emitted, so instances are unchanged
  // and the depth-first `buildAgs4` walk never sees it.
  declare protected readonly __agsGroup: true;
}
