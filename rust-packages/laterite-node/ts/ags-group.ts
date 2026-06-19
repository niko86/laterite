// The marker base for the generated typed-graph classes. In its OWN module (no
// imports) so the generated file can extend it without a circular dependency —
// `typed-graph.ts` re-exports both. `buildAgs4` uses `instanceof AgsGroup` to
// recognise a typed tree root (minification-safe).
export abstract class AgsGroup {}
