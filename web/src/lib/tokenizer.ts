// The browser's AGS4 line tokenizer + field quoter — sourced from the shared
// Rust leaves through a tiny dedicated wasm (#533, part of the #527 arc).
//
// This retires the hand-written TS state machine that used to live in
// agsline.ts: `splitAgsFields` now wraps `laterite_ags4_parse::tokenize_spans`
// and `quoteAgsField` wraps `laterite_types::quote_field`, so the browser
// tokenizes/quotes through the SAME authority as every other surface (the
// engine wasm, the wheel, the node binding, the CLI) instead of a second copy.
//
// The module is ~30 KB (not the 6.9 MB engine), instantiated ONCE on the main
// thread at boot — the app gates first render on `tokenizerReady()`, so the
// synchronous `splitAgsFields`/`quoteAgsField` below never run before the wasm
// is live. The offset model these return is browser-only by design (excluded
// from the #555 cross-surface value gate); its lossless-reassembly + code-point
// offset invariants are pinned in Rust (`tokenize_spans` proptest).

import init, {
  tokenize_spans as wasmTokenize,
  quote_field as wasmQuote,
} from "../wasm-tokenizer/ags4_tokenizer.js";

export interface AgsField {
  /** The raw slice for this field, including quotes + trailing comma. */
  text: string;
  /** Char offset (code points) where this token starts in `raw`. */
  start: number;
  /** Char offset (code points) one past this token's end. */
  end: number;
  /**
   * Char offset (code points) of the field's INNER value — the content
   * between the surrounding quotes (an unquoted field: its trimmed
   * content), excluding the quotes AND the trailing comma. This is the
   * range a field-level highlight should paint, not the whole token.
   * For an empty quoted field `valueStart === valueEnd`.
   */
  valueStart: number;
  /** Char offset (code points) one past the inner value's end. */
  valueEnd: number;
}

/**
 * Split a raw AGS4 line into offset-preserving field tokens whose `.text`
 * slices, concatenated in order, reproduce `raw` exactly. Backed by the shared
 * Rust `tokenize_spans` via wasm — requires {@link tokenizerReady} to have
 * resolved (the app gates first render on it, so render-path callers are safe).
 */
export function splitAgsFields(raw: string): AgsField[] {
  return wasmTokenize(raw) as AgsField[];
}

/** Quote a raw value as an AGS4 field: wrap in double quotes, doubling any
 *  internal quote. Backed by the shared Rust `quote_field` via wasm. */
export function quoteAgsField(value: string): string {
  return wasmQuote(value);
}

/** Build one AGS4 line from raw field values: each quoted, comma-joined.
 *  `agsLine(["GROUP", "LOCA"])` → `"GROUP","LOCA"`. */
export function agsLine(values: string[]): string {
  return values.map(quoteAgsField).join(",");
}

// --- boot: instantiate the tiny wasm once; first render awaits this ---------
//
// wasm-pack `--target web` exports an async `init(module_or_path)`; after it
// resolves the tokenizer functions above run synchronously. We pass the Vite
// `?url` asset (imported lazily so this module stays importable in a plain
// node/vitest context, where the wasm is instead init'd from disk in a setup
// file) explicitly, matching the worker's init — the glue's `import.meta.url`
// fetch fallback breaks under a non-root deploy base.
let readyPromise: Promise<void> | null = null;

export function tokenizerReady(): Promise<void> {
  if (!readyPromise) {
    readyPromise = import("../wasm-tokenizer/ags4_tokenizer_bg.wasm?url")
      .then(({ default: url }) => init({ module_or_path: url }))
      .then(() => undefined);
  }
  return readyPromise;
}
