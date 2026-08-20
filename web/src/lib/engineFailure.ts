import { EngineUnavailableError } from "./workerChannel";

/** What a pane holding its own Try again button can honestly say. */
const RETRY_CONTROL_RECOVERY = "Check your connection and try again.";

/** What a tier-1 pane can. Validate and Fix share one engine channel and have
 *  no retry control by design (#391) — the channel retires the dead worker, so
 *  the next file, paste or sample starts a fresh one. That is the action worth
 *  naming, and it is the only one either pane can actually be asked to take. */
const TIER1_RECOVERY = "Load your file again to retry.";

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
 *  crash line with a retry sentence next to its Try again button — that stays
 *  pane-side, because it describes a control only that pane has.
 *
 *  The load/crash split is `EngineUnavailableError`'s reason for existing:
 *  the two are equally retryable — the channel has retired the worker either
 *  way, so the pane's next request starts a fresh one — but not equally
 *  explicable. "Check your connection" is the useful thing to say about an
 *  engine that never downloaded and a false lead about one that died holding
 *  a file.
 *
 *  What closes that sentence varies, because its advice does not hold
 *  everywhere (#413). "…and try again" points at a Try again button: Explore
 *  and the Excel converter have one, and the tier-1 panes deliberately do not
 *  — #391 left Validate and Fix relying on the channel retiring the dead
 *  worker, so their recovery is the next input rather than a control. Telling
 *  one of their users to try again names an affordance that is not on the
 *  screen. Both wordings live here and `tier1EngineFailureMessage` is how the
 *  two panes sharing that channel reach the true one: a named door rather
 *  than a fourth argument spelled at each call site, since the choice follows
 *  the channel, not the pane.
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
  loadRecovery = RETRY_CONTROL_RECOVERY,
): string {
  if (e instanceof EngineUnavailableError)
    return e.reason === "load"
      ? `${engine} couldn't be downloaded — the rest of the app is unaffected. ${loadRecovery}`
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

/** The tier-1 door onto that mapping — ValidatePane and FixPane. Same voice,
 *  same branches; only the recovery differs, and it differs because of the
 *  channel they share rather than anything either pane chose. */
export const tier1EngineFailureMessage = (e: unknown, engine: string): string =>
  engineFailureMessage(e, engine, undefined, TIER1_RECOVERY);
