// `loadSensitive()` — the fetch-once-and-share loader for the sensitive-headings
// SSOT, in its own file because it needs a fresh module registry per case.
//
// The classification it fetches decides which columns the Anonymiser pre-ticks for
// redaction. That makes both of its untested behaviours consequential:
//
//   * the module-level cache means the promise is shared, so a failed first load
//     is remembered too — every later caller gets the same rejection rather than
//     silently retrying and appearing to work;
//   * a non-OK response must THROW. `res.json()` on a 404 body parses HTML as
//     JSON and rejects with a syntax error, or worse, succeeds against a proxy's
//     JSON error page and yields a doc with NO headings — which pre-ticks
//     nothing, and hands the user an anonymiser that redacts nothing while
//     looking like it worked.
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";

const DOC = {
  categories: { coordinate: "..." },
  scrub_policy: { coordinate: "blank" },
  headings: { LOCA_NATE: { category: "coordinate" } },
};

/** A fresh copy of the module, so its module-level `cache` starts empty. */
async function freshModule() {
  vi.resetModules();
  return import("./sensitive");
}

beforeEach(() => {
  vi.unstubAllGlobals();
});

afterEach(() => {
  vi.unstubAllGlobals();
  vi.restoreAllMocks();
});

describe("loadSensitive", () => {
  it("fetches the SSOT and returns the parsed document", async () => {
    const fetchMock = vi.fn().mockResolvedValue({
      ok: true,
      status: 200,
      json: () => Promise.resolve(DOC),
    });
    vi.stubGlobal("fetch", fetchMock);

    const { loadSensitive } = await freshModule();
    await expect(loadSensitive()).resolves.toEqual(DOC);

    expect(fetchMock).toHaveBeenCalledTimes(1);
    // The URL is BASE_URL-relative, so the app still finds it when served from a
    // subpath rather than only at the root the site deploys to today.
    expect(String(fetchMock.mock.calls[0]?.[0])).toContain(
      "sensitive_headings.json",
    );
  });

  it("fetches ONCE however many callers ask", async () => {
    // The point of the cache. Several components load this during the same
    // render pass; without sharing, the anonymiser would issue a request per
    // consumer for a static asset.
    const fetchMock = vi.fn().mockResolvedValue({
      ok: true,
      status: 200,
      json: () => Promise.resolve(DOC),
    });
    vi.stubGlobal("fetch", fetchMock);

    const { loadSensitive } = await freshModule();
    const [a, b, c] = await Promise.all([
      loadSensitive(),
      loadSensitive(),
      loadSensitive(),
    ]);

    expect(fetchMock).toHaveBeenCalledTimes(1);
    // The same object, not merely equal ones — proving one shared promise.
    expect(a).toBe(b);
    expect(b).toBe(c);
  });

  it("rejects on a non-OK response instead of parsing the error body", async () => {
    // A 404 from a mis-synced deploy returns an HTML page. Handing that to
    // res.json() either throws something unrelated to the real problem, or (via a
    // proxy's JSON error page) succeeds and produces a doc with no headings —
    // an anonymiser that pre-ticks nothing and looks like it worked.
    const json = vi.fn();
    vi.stubGlobal(
      "fetch",
      vi.fn().mockResolvedValue({ ok: false, status: 404, json }),
    );

    const { loadSensitive } = await freshModule();
    await expect(loadSensitive()).rejects.toThrow("HTTP 404");
    // The status is IN the message — "failed to load" alone cannot distinguish a
    // missing asset from a broken one.
    expect(json).not.toHaveBeenCalled();
  });

  it("reports the actual status code, not a generic failure", async () => {
    vi.stubGlobal(
      "fetch",
      vi.fn().mockResolvedValue({ ok: false, status: 503, json: vi.fn() }),
    );
    const { loadSensitive } = await freshModule();
    await expect(loadSensitive()).rejects.toThrow("HTTP 503");
  });

  it("remembers a failure rather than silently retrying", async () => {
    // The cache is assigned before the promise settles, so a rejection is cached
    // too. That is worth pinning either way: a caller that retries on its own
    // would otherwise see a mysterious sticky failure, and someone "fixing" it by
    // clearing the cache on error would turn one bad deploy into a request storm.
    const fetchMock = vi
      .fn()
      .mockResolvedValue({ ok: false, status: 500, json: vi.fn() });
    vi.stubGlobal("fetch", fetchMock);

    const { loadSensitive } = await freshModule();
    await expect(loadSensitive()).rejects.toThrow("HTTP 500");
    await expect(loadSensitive()).rejects.toThrow("HTTP 500");
    expect(fetchMock).toHaveBeenCalledTimes(1);
  });

  it("propagates a network failure", async () => {
    vi.stubGlobal("fetch", vi.fn().mockRejectedValue(new TypeError("offline")));
    const { loadSensitive } = await freshModule();
    await expect(loadSensitive()).rejects.toThrow("offline");
  });
});
