import { describe, expect, it } from "vitest";
import {
  appOnlyPackages,
  findForbiddenModules,
  SHARED_PACKAGES,
} from "./heavyDeps";

describe("appOnlyPackages", () => {
  it("forbids everything declared that is not explicitly shared", () => {
    expect(
      appOnlyPackages(
        { echarts: "^6.1.0", leaflet: "^1.9.4", "solid-js": "^1.9.14" },
        ["solid-js"],
      ),
    ).toEqual(["echarts", "leaflet"]);
  });

  it("sorts, so a violation message is stable between builds", () => {
    expect(
      appOnlyPackages({ leaflet: "*", "apache-arrow": "*", echarts: "*" }, []),
    ).toEqual(["apache-arrow", "echarts", "leaflet"]);
  });

  it("allows the shared set by default", () => {
    const declared = Object.fromEntries(
      SHARED_PACKAGES.map((pkg) => [pkg, "*"]),
    );
    expect(appOnlyPackages(declared)).toEqual([]);
  });

  // The guard's whole point: the app growing a heavy dependency must fence the
  // apex off from it on the same commit, with nobody editing a list.
  it("picks up a newly declared package with no edit here", () => {
    expect(appOnlyPackages({ "some-new-3d-engine": "^1.0.0" }, [])).toEqual([
      "some-new-3d-engine",
    ]);
  });
});

describe("findForbiddenModules", () => {
  const FORBIDDEN = ["@duckdb/duckdb-wasm", "apache-arrow", "leaflet"];

  it("passes a graph of first-party modules", () => {
    expect(
      findForbiddenModules(
        ["/repo/web/landing/index.html", "/repo/web/src/shared/styles/foo.css"],
        FORBIDDEN,
      ),
    ).toEqual([]);
  });

  it("catches an absolute module path and reports what pulled it in", () => {
    expect(
      findForbiddenModules(
        ["/repo/web/node_modules/leaflet/dist/leaflet-src.js"],
        FORBIDDEN,
      ),
    ).toEqual([
      {
        pkg: "leaflet",
        moduleId: "/repo/web/node_modules/leaflet/dist/leaflet-src.js",
      },
    ]);
  });

  it("catches a scoped package", () => {
    expect(
      findForbiddenModules(
        ["/repo/web/node_modules/@duckdb/duckdb-wasm/dist/duckdb-browser.mjs"],
        FORBIDDEN,
      ).map((v) => v.pkg),
    ).toEqual(["@duckdb/duckdb-wasm"]);
  });

  it("catches a nested (unhoisted) install", () => {
    expect(
      findForbiddenModules(
        ["/repo/web/node_modules/some-wrapper/node_modules/leaflet/dist/x.js"],
        FORBIDDEN,
      ).map((v) => v.pkg),
    ).toEqual(["leaflet"]);
  });

  it("catches a repo-relative id, where the marker starts the string", () => {
    expect(
      findForbiddenModules(["node_modules/leaflet/dist/x.js"], FORBIDDEN).map(
        (v) => v.pkg,
      ),
    ).toEqual(["leaflet"]);
  });

  // The reason this matches a path SEGMENT and not the bare name: a substring
  // test on "leaflet" condemns leaflet-draw, and one on "apache-arrow" would
  // have to be right about every package that merely mentions it.
  it("does not condemn a package whose name only starts with a forbidden one", () => {
    expect(
      findForbiddenModules(
        ["/repo/web/node_modules/leaflet-draw/dist/leaflet.draw.js"],
        FORBIDDEN,
      ),
    ).toEqual([]);
  });

  it("does not condemn a package whose name only ends with a forbidden one", () => {
    expect(
      findForbiddenModules(
        ["/repo/web/node_modules/not-leaflet/index.js"],
        FORBIDDEN,
      ),
    ).toEqual([]);
  });

  it("ignores Vite's virtual modules, which are not paths", () => {
    expect(
      findForbiddenModules(["\0vite/modulepreload-polyfill.js"], FORBIDDEN),
    ).toEqual([]);
  });

  it("normalises Windows separators", () => {
    expect(
      findForbiddenModules(
        ["C:\\repo\\web\\node_modules\\leaflet\\dist\\x.js"],
        FORBIDDEN,
      ).map((v) => v.pkg),
    ).toEqual(["leaflet"]);
  });

  it("reports each package once, however many of its modules are pulled in", () => {
    expect(
      findForbiddenModules(
        [
          "/w/node_modules/leaflet/dist/a.js",
          "/w/node_modules/leaflet/dist/b.js",
          "/w/node_modules/leaflet/dist/c.js",
        ],
        FORBIDDEN,
      ),
    ).toEqual([
      { pkg: "leaflet", moduleId: "/w/node_modules/leaflet/dist/a.js" },
    ]);
  });

  it("orders by package, so the failure reads the same on every runner", () => {
    expect(
      findForbiddenModules(
        [
          "/w/node_modules/leaflet/dist/a.js",
          "/w/node_modules/@duckdb/duckdb-wasm/dist/b.js",
          "/w/node_modules/apache-arrow/Arrow.dom.mjs",
        ],
        FORBIDDEN,
      ).map((v) => v.pkg),
    ).toEqual(["@duckdb/duckdb-wasm", "apache-arrow", "leaflet"]);
  });
});
