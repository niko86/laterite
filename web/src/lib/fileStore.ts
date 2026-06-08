// The single loaded AGS file, shared across the Validate / Explore / Fix /
// Tools tabs. Lifted out of ValidatePane's local signals so a file dropped
// (or edited, or fixed) on one tab is the same file every tab sees — and so
// state DERIVED from the file (the Explore DuckDB tables, the computed fixes)
// recomputes the moment the bytes change.
//
// Module-level Solid signals are app-lifetime and owner-less by design here:
// there is exactly one loaded file at a time, for the life of the page.

import { createMemo, createSignal } from "solid-js";

const [bytes, setBytesSignal] = createSignal<Uint8Array | null>(null);
const [name, setName] = createSignal<string>("");
// True when the current bytes came from a hand-edit (typing/paste). A
// <textarea> strips every \r, so edited bytes are LF-only and must have CRLF
// re-inserted before the engine sees them (else AGS4 Rule 2a fires on every
// line, permanently). Shared so the Fix tab — not just Validate — derives
// canonicalBytes the same way.
const [edited, setEdited] = createSignal(false);
// The file as originally loaded — the baseline for "revert to original" and
// the Fix tab's diff. Set only by loadFile; NOT touched by edits or fixes.
const [originalBytes, setOriginalBytes] = createSignal<Uint8Array | null>(null);

// Modules that cache state derived from the file (e.g. DuckDB ingested
// tables) register a hook here; it fires on every bytes change so they can
// drop the stale derivation.
const resetHooks = new Set<() => void>();
const fireReset = () => {
  for (const hook of resetHooks) hook();
};

/** Load a fresh file (upload / sample). Establishes the baseline + clears the
 *  edited flag. */
function loadFile(b: Uint8Array, n: string): void {
  setBytesSignal(b);
  setName(n);
  setEdited(false);
  setOriginalBytes(b);
  fireReset();
}

/** Set new bytes for an EDIT or applied FIX (originalBytes left intact).
 *  Invalidates file-derived caches. */
function setBytes(b: Uint8Array | null, n?: string): void {
  setBytesSignal(b);
  if (n !== undefined) setName(n);
  fireReset();
}

// What the engine (validate / fixes / download / export) sees: CRLF
// re-inserted for hand-edited content only — an uploaded LF file must still
// (correctly) flag Rule 2a. The editor's bytes()/text() stay LF, so the
// textarea round-trip (and cursor) is untouched. Hand-edits are UTF-8.
const canonicalBytes = createMemo(() => {
  const b = bytes();
  if (!b || !edited()) return b;
  const s = new TextDecoder("utf-8", { fatal: false }).decode(b);
  return new TextEncoder().encode(s.replace(/\r?\n/g, "\r\n"));
});

export const fileStore = {
  /** Reactive accessor for the current file bytes (null until loaded). */
  bytes,
  /** Reactive accessor for the display name. */
  name,
  /** Reactive accessor: did the current bytes come from a hand-edit? */
  edited,
  /** Reactive accessor for the originally-loaded baseline bytes. */
  originalBytes,
  /** Derived: the bytes the engine should validate/fix (CRLF-correct). */
  canonicalBytes,
  loadFile,
  setBytes,
  /** Raw name setter (supports the functional updater form). */
  setName,
  setEdited,
  /** Register an invalidate-on-file-change hook; returns an unsubscribe. */
  onReset(fn: () => void): () => void {
    resetHooks.add(fn);
    return () => {
      resetHooks.delete(fn);
    };
  },
};
