// `fetchUnion()` and the two loaders that sit on it — in their own file because
// each case needs a fresh module registry to reset the shared `unionCache`.
//
// The union dictionary is the single source every Explore builder reads: the
// group tree, the KEY sets the join suggestions are derived from, the per-
// edition projection the validator UI shows. Its failure doors matter more than
// its happy path:
//
//   * a non-OK response must THROW. `res.json()` on a 404 will either reject
//     with a syntax error somewhere unrelated, or — against a proxy that serves
//     a JSON error page — SUCCEED, yielding a union with no groups. That is an
//     Explore tab where nothing is a known AGS group, no joins are suggested,
//     and no error is shown;
//   * the cache is a promise, so a failed first load is remembered. That is
//     deliberate (one dead asset should not become one request per component),
//     but it means the rejection has to be reproducible, not swallowed;
//   * `loadStandardDict(null)` must resolve to "auto" rather than passing the
//     nullish through as an edition name — an unrecognised edition projects to
//     a different (smaller) dictionary, not to an error.
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";

/** Two groups and a parent link — enough for the projection and the DictMap. */
const UNION = {
  groups: {
    PROJ: {
      parent: null,
      description: "Project Information",
      headings: [
        { name: "PROJ_ID", status: "KEY", type: "ID", description: "id" },
      ],
    },
    LOCA: {
      parent: "PROJ",
      description: "Location Details",
      headings: [
        { name: "LOCA_ID", status: "KEY", type: "ID", description: "id" },
        { name: "LOCA_GL", status: "OTHER", type: "2DP", description: "level" },
      ],
    },
  },
};

const okFetch = () =>
  vi.fn().mockResolvedValue({
    ok: true,
    status: 200,
    json: () => Promise.resolve(structuredClone(UNION)),
  });

beforeEach(() => {
  vi.resetModules();
  vi.unstubAllGlobals();
});

afterEach(() => {
  vi.unstubAllGlobals();
  vi.restoreAllMocks();
});

describe("fetchUnion", () => {
  it("fetches the union once however many callers ask", async () => {
    const fetchMock = okFetch();
    vi.stubGlobal("fetch", fetchMock);

    const { fetchUnion } = await import("./dict");
    const [a, b] = await Promise.all([fetchUnion(), fetchUnion()]);

    expect(fetchMock).toHaveBeenCalledTimes(1);
    // The same settled promise, so both callers see the same object.
    expect(a).toBe(b);
    // BASE_URL-relative, so the app still finds it when served from a subpath
    // (this site deploys under /laterite/).
    expect(String(fetchMock.mock.calls[0]?.[0])).toContain(
      "ags_dictionary.json",
    );
  });

  it("throws on a non-OK response instead of parsing the error body", async () => {
    const json = vi.fn();
    vi.stubGlobal(
      "fetch",
      vi.fn().mockResolvedValue({ ok: false, status: 404, json }),
    );

    const { fetchUnion } = await import("./dict");
    await expect(fetchUnion()).rejects.toThrow("HTTP 404");
    // And it never reached the body — the whole point of checking `ok` first.
    expect(json).not.toHaveBeenCalled();
  });

  it("remembers a failure rather than re-requesting a dead asset", async () => {
    const fetchMock = vi
      .fn()
      .mockResolvedValue({ ok: false, status: 500, json: () => ({}) });
    vi.stubGlobal("fetch", fetchMock);

    const { fetchUnion } = await import("./dict");
    await expect(fetchUnion()).rejects.toThrow("HTTP 500");
    await expect(fetchUnion()).rejects.toThrow("HTTP 500");
    expect(fetchMock).toHaveBeenCalledTimes(1);
  });
});

describe("the loaders on top of it", () => {
  it("loadStandardDict projects an edition, and defaults a nullish one to auto", async () => {
    vi.stubGlobal("fetch", okFetch());
    const { loadStandardDict } = await import("./dict");

    const auto = await loadStandardDict(null);
    // "auto" is a resolution, not a literal edition — it must yield real groups.
    expect(auto.groups.map((g) => g.code).sort()).toEqual(["LOCA", "PROJ"]);
    // undefined takes the same door, since the picker starts empty.
    const undef = await loadStandardDict(undefined);
    expect(undef.groups).toHaveLength(2);
    // The parent link survives the projection — the Explore tree is built on it.
    expect(auto.groups.find((g) => g.code === "LOCA")?.parent).toBe("PROJ");
  });

  it("loadDict turns the fetched union into the shared DictMap", async () => {
    vi.stubGlobal("fetch", okFetch());
    const { loadDict } = await import("./relationships");

    const map = await loadDict();
    expect(map.get("LOCA")?.parent).toBe("PROJ");
    // The KEY set is what every join suggestion is derived from, so it is the
    // part of the map that has to survive the conversion.
    expect(map.get("LOCA")?.keys).toEqual(["LOCA_ID"]);
    expect(map.get("PROJ")?.parent).toBeNull();
  });

  it("propagates a dead dictionary through loadDict rather than yielding an empty map", async () => {
    // An empty DictMap is the dangerous outcome: every group becomes "unknown",
    // the Explore tab suggests nothing, and nothing anywhere says why.
    vi.stubGlobal(
      "fetch",
      vi.fn().mockResolvedValue({ ok: false, status: 503, json: () => ({}) }),
    );
    const { loadDict } = await import("./relationships");
    await expect(loadDict()).rejects.toThrow("HTTP 503");
  });
});
