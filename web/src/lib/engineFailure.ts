import { EngineUnavailableError } from "./workerChannel";

/** One voice for a dead engine, across the four surfaces that render one:
 *  FixPane, ValidatePane, ExplorePane (#391), and the Excel converter (#414).
 *
 *  Before this they spoke three ways — ExplorePane with typed copy,
 *  ValidatePane with a bare stringified error, FixPane not at all, its empty
 *  fix list posing as a clean file. The mapping lives here so the same
 *  failure reads the same everywhere; what stays with each surface is only
 *  its own noun for `engine` ("The validator", "The fix engine", …),
 *  capitalised to open the sentence. The Excel converter (scoped out of #391,
 *  joined in #414) also derives a retry flag beside its call and suffixes its
 *  crash line with a retry sentence next to its Try again button — both stay
 *  pane-side because retryability is something this mapping deliberately
 *  doesn't model.
 *
 *  The load/crash split is `EngineUnavailableError`'s reason for existing:
 *  the two are equally retryable — the channel has retired the worker either
 *  way, so the pane's next request starts a fresh one — but not equally
 *  explicable. "Check your connection" is the useful thing to say about an
 *  engine that never downloaded and a false lead about one that died holding
 *  a file.
 *
 *  `untypedFallback` replaces only the last branch — an error that is not an
 *  engine-availability failure. Each caller's copy is true only where it
 *  stands, which is why both are overrides passed in and not branches here.
 *  ExplorePane's offline case: its DuckDB wasm is the one engine NOT
 *  precached, so a first Explore while offline fails with a raw fetch error
 *  worth explaining — copy describing tier-3 caching that would be false in
 *  the panes whose wasm is precached. The Excel converter's op-level line: an
 *  untyped error there is a conversion that failed, about the workbook and
 *  not the engine, so it keeps "Conversion failed" over this branch's
 *  engine-noun framing.
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
  // Op-level worker failures arrive as `new Error(msg.error)`, so `String(e)`
  // would double the phrasing: "… failed: Error: <message>" (#415). Strip only
  // the redundant plain-`Error` prefix — a named type (TypeError) keeps its
  // name, which is half the information, and an empty message keeps `String(e)`
  // so the line never ends at a bare colon.
  const detail =
    e instanceof Error && e.name === "Error" && e.message !== ""
      ? e.message
      : String(e);
  return untypedFallback ?? `${engine} failed: ${detail}`;
}
