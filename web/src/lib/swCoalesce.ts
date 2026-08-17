// In-flight request coalescing for the service worker's CacheFirst routes
// (#366). CacheFirst alone answers each cache miss with its own network fetch,
// so an idle warm still downloading when its engine's worker starts becomes a
// SECOND full download of the same artifact — several MB for tier 2, ~36 MB
// for DuckDB — splitting the link exactly when it is slowest. The main thread
// cannot close that race: the warm holds no handle a later worker could join,
// and the worker is created from five call paths that would each need one.
// Every one of those fetches passes through here once the SW controls the
// page, which is what makes this the covering fix rather than a guard on one
// path. (The remaining gap is a page the SW does NOT yet control — the cold
// first visit — where no interception exists at all; prefetch.ts names it.)
//
// The joiner is released when the leader's CACHE WRITE settles, not when its
// response arrives: a fetch resolves at headers, with the body still
// streaming, and a joiner released that early re-runs the strategy against a
// cache the bytes have not reached — starting download #2 anyway. Workbox
// exposes exactly that moment as `handleAll`'s second promise, which settles
// once the strategy's cache put (queued via waitUntil) has finished.

/** What a coalesced handler is called with, and what it forwards: the request
 *  plus whatever else the router supplied (`event`, `url` — passed through
 *  untouched at runtime). Only the URL is read here. */
export interface CoalesceOptions {
  request: Request;
}

/** The slice of a workbox `Strategy` this wrapper needs: one call that returns
 *  the response promise and the everything-including-the-cache-write promise.
 *  Structural, so the unit suite can drive it with a fake that separates the
 *  two settlements — the distinction the whole module turns on. */
export interface CoalescableStrategy {
  handleAll(options: CoalesceOptions): [Promise<Response>, Promise<void>];
}

/** Wrap `strategy` so concurrent requests for one URL share one network
 *  flight. The first requester leads; the rest wait for its cache write, then
 *  re-enter the strategy — a cache hit if the leader landed, an honest fresh
 *  fetch if it failed. */
export function coalesce(
  strategy: CoalescableStrategy,
): (options: CoalesceOptions) => Promise<Response> {
  const inflight = new Map<string, Promise<void>>();

  return async (options: CoalesceOptions): Promise<Response> => {
    const key = options.request.url;

    // Wait out every flight already up for this URL. A loop, not an `if`:
    // another waiter woken by the same settlement may have registered a new
    // flight before this one re-checks.
    for (;;) {
      const pending = inflight.get(key);
      if (!pending) break;
      await pending;
    }

    const [response, done] = strategy.handleAll(options);
    // The map entry swallows `done`'s rejection deliberately: a failed flight
    // must release its joiners to fetch for themselves, and the failure itself
    // already reaches the caller through `response`.
    const flight: Promise<void> = done
      .catch(() => {})
      .then(() => {
        // Guarded, not unconditional: a delete that ran late must never evict
        // a successor's live flight and un-coalesce it.
        if (inflight.get(key) === flight) inflight.delete(key);
      });
    inflight.set(key, flight);
    return response;
  };
}
