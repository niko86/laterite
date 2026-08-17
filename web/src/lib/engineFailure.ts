import { EngineUnavailableError } from "./workerChannel";

/** One voice for a dead engine, across the three panes that render one
 *  (#391): FixPane, ValidatePane, ExplorePane.
 *
 *  Before this they spoke three ways — ExplorePane with typed copy,
 *  ValidatePane with a bare stringified error, FixPane not at all, its empty
 *  fix list posing as a clean file. The mapping lives here so the same
 *  failure reads the same everywhere; what stays with each pane is only its
 *  own noun for `engine` ("The validator", "The fix engine", …), capitalised
 *  to open the sentence. The Excel tool keeps its own hand-rolled copy on
 *  purpose (#391 scoped it out): its failure line also derives a retry flag,
 *  which this mapping deliberately doesn't model.
 *
 *  The load/crash split is `EngineUnavailableError`'s reason for existing:
 *  the two are equally retryable — the channel has retired the worker either
 *  way, so the pane's next request starts a fresh one — but not equally
 *  explicable. "Check your connection" is the useful thing to say about an
 *  engine that never downloaded and a false lead about one that died holding
 *  a file.
 *
 *  `untypedFallback` replaces only the last branch — an error that is not an
 *  engine-availability failure. It exists for ExplorePane's offline case: its
 *  DuckDB wasm is the one engine NOT precached, so a first Explore while
 *  offline fails with a raw fetch error worth explaining. That copy describes
 *  tier-3 caching and would be false in the panes whose wasm is precached,
 *  which is why it is an override passed by one caller and not a branch here.
 */
export function engineFailureMessage(
  e: unknown,
  engine: string,
  untypedFallback?: string,
): string {
  if (e instanceof EngineUnavailableError)
    return e.reason === "load"
      ? `${engine} couldn't be downloaded — the rest of the app is unaffected. Check your connection and try again.`
      : `${engine} stopped — the rest of the app is unaffected.`;
  return untypedFallback ?? `${engine} failed: ${String(e)}`;
}
