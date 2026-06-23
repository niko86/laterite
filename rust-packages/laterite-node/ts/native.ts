// The native addon — the Node analog of laterite-py's `_laterite_native`.
//
// `#native` is a package subpath-import (see package.json `imports`) resolving to
// the napi-rs-generated CJS loader (`./index.js`, which picks the right
// per-platform `.node`) + its types (`./index.d.ts`). Using the subpath import
// rather than a relative `../index.js` keeps this seam stable through bundling —
// the compiled dist resolves `#native` the same way the source does.
export {
  version,
  parseArrow,
  runCheck,
  fixFile,
  listRules,
  emitAgs4FromIpc,
  Reading,
  canonicalType,
  displayHint,
  parseValue,
  transportPack,
  transportUnpack,
  transportLock,
  transportUnlock,
} from "#native";
export type {
  GroupMeta,
  Finding,
  ValidationReport,
  AppliedFix,
  FixReport,
  GroupIpc,
  EmitResult as NativeEmitResult,
  PackStats,
  UnpackStats,
} from "#native";
