// The tier-2 engine's wasm URL, and nothing else. It lives in its own module
// because TWO graphs need it and they have to agree on it exactly:
// `tier2.worker.ts` instantiates from it, and the idle warm in `prefetch.ts`
// (#356) primes it into the CacheFirst bucket before either of its tabs is
// opened. Import the `?url` asset separately in each and a path edit on one side
// leaves the warm priming something nothing compiles — the 5.2 MB is then
// downloaded twice, on exactly the devices the warm exists to help, and nothing
// errors. Same failure mode as the precache glob in `vite.config.ts`, one file
// along.
//
// Vite resolves one `?url` id to one emitted asset, so the main-thread and
// worker bundles inline the same fingerprinted name. That is the property the
// whole warm rests on, and it is not self-evident from two separate bundles —
// the e2e that opens Excel after a completed warm and asserts no NETWORK fetch
// is what actually holds it.
import wasmUrl from "../wasm-full/ags4_wasm_full_bg.wasm?url";

export const TIER2_WASM_URL: string = wasmUrl;
