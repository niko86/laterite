// The typed-graph surface — `from laterite import PROJ, LOCA, …`. The 92 classes
// are generated from the dictionary (typed-graph.generated.ts, drift-tested);
// `AgsGroup` (its own module, to avoid a cycle) is the marker base that lets
// `emitAgs4` recognise a typed tree root via `instanceof`.
export { AgsGroup } from "./ags-group";
export * from "./typed-graph.generated";
