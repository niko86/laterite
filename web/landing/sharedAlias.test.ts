import { describe, expect, it } from "vitest";
import { resolve } from "node:path";
import appConfig from "../vite.config";
import landingConfig from "./vite.config";

// #406 asks that BOTH builds resolve the shared primitives through the alias,
// and the app half is self-enforcing: web/src/components/validate/SummaryBanner.tsx
// imports `@shared/components`, so a broken alias fails `tsc` and the app build.
//
// The landing half has no such consumer yet. Its page is deliberately static —
// it ships zero JavaScript, and adopting a Solid primitive purely to keep the
// path warm would put the whole Solid runtime on a placeholder to render one
// label. The page gets its real components in #395.
//
// So this pins the arrangement instead of a usage: that the two configs agree
// on where `@shared` points, and that the landing build still carries the
// plugin needed to compile a component when #395 brings one. Those are the two
// ways this can regress while every other gate stays green — a divergent alias,
// or a plugin quietly dropped from a config nothing currently exercises.

/** Vite's alias config is either a record or an array of {find, replacement}. */
function aliasFor(
  config: { resolve?: { alias?: unknown } },
  key: string,
): string | undefined {
  const alias = config.resolve?.alias;
  if (!alias) return undefined;
  if (Array.isArray(alias)) {
    const hit = (alias as { find: unknown; replacement: string }[]).find(
      (a) => a.find === key,
    );
    return hit?.replacement;
  }
  return (alias as Record<string, string>)[key];
}

const pluginNames = (config: { plugins?: unknown }): string[] =>
  ((config.plugins ?? []) as { name?: string }[])
    .flat(Infinity)
    .map((p) => p.name)
    .filter((n): n is string => typeof n === "string");

describe("the @shared alias", () => {
  const expected = resolve(import.meta.dirname, "../src/shared");

  it("points at the shared directory from the app build", () => {
    expect(aliasFor(appConfig, "@shared")).toBe(expected);
  });

  it("points at the SAME directory from the landing build", () => {
    expect(aliasFor(landingConfig, "@shared")).toBe(expected);
  });

  // The point of the whole arrangement: a button exists once. Two aliases that
  // drift are two copies of it, and nothing else would notice.
  it("resolves identically in both, so a primitive cannot fork", () => {
    expect(aliasFor(landingConfig, "@shared")).toBe(
      aliasFor(appConfig, "@shared"),
    );
  });
});

describe("the landing build", () => {
  it("can compile a Solid component, ready for #395's primitives", () => {
    expect(pluginNames(landingConfig).join(" ")).toMatch(/solid/);
  });

  it("keeps its dependency firewall armed", () => {
    expect(pluginNames(landingConfig)).toContain("no-app-only-dependencies");
  });

  // The separation #394 exists for: the app's machinery must not be here.
  it("carries none of the app-only build machinery", () => {
    const names = pluginNames(landingConfig).join(" ");
    expect(names).not.toMatch(/pwa/i);
    expect(names).not.toContain("spa-404-fallback");
    expect(names).not.toContain("offload-duckdb-wasm-to-r2");
  });
});
