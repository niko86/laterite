import { describe, expect, it } from "vitest";

import {
  localImports,
  packageImports,
  rewriteFontUrls,
  toMaterialScheme,
} from "./docs-tokens.mjs";

describe("localImports", () => {
  it("lists the relative stylesheets in source order", () => {
    expect(
      localImports(`@import "./fonts.css";\n@import "./colors.css";\n`),
    ).toEqual(["fonts.css", "colors.css"]);
  });

  it("ignores bare package specifiers", () => {
    expect(localImports(`@import "@fontsource/x/latin-400.css";`)).toEqual([]);
  });
});

describe("packageImports", () => {
  it("lists the bare specifiers in source order", () => {
    expect(
      packageImports(
        `@import "@fontsource-variable/figtree/wght.css";\n` +
          `@import "@fontsource/ibm-plex-mono/latin-400.css";\n`,
      ),
    ).toEqual([
      "@fontsource-variable/figtree/wght.css",
      "@fontsource/ibm-plex-mono/latin-400.css",
    ]);
  });

  it("ignores relative imports", () => {
    expect(packageImports(`@import "./colors.css";`)).toEqual([]);
  });
});

describe("toMaterialScheme", () => {
  it("rewrites the dark rule onto Material's slate attribute", () => {
    expect(toMaterialScheme(`:root { --a: 1 }\n.dark {\n  --a: 2;\n}\n`)).toBe(
      `:root { --a: 1 }\n[data-md-color-scheme="slate"] {\n  --a: 2;\n}\n`,
    );
  });

  // The shared file's own prose says ".dark" repeatedly while explaining the
  // selector; only the rule may move.
  it("leaves the class name alone where it is prose, not a selector", () => {
    const css = `/* a retune must say :root:not(.dark) rather than .dark */\n.dark {\n}\n`;
    expect(toMaterialScheme(css)).toBe(
      `/* a retune must say :root:not(.dark) rather than .dark */\n` +
        `[data-md-color-scheme="slate"] {\n}\n`,
    );
  });

  // A restructure that splits or drops the dark block must fail the sync rather
  // than quietly ship a docs site with no dark theme.
  it("throws when the dark rule is missing", () => {
    expect(() => toMaterialScheme(`:root { --a: 1 }`)).toThrow(
      /exactly one `\.dark` rule/,
    );
  });

  it("throws when there is more than one dark rule", () => {
    expect(() => toMaterialScheme(`.dark {\n}\n.dark {\n}\n`)).toThrow(
      /exactly one `\.dark` rule/,
    );
  });
});

describe("rewriteFontUrls", () => {
  it("repoints each file at the docs font directory and collects it", () => {
    const { css, files } = rewriteFontUrls(
      `src: url(./files/a.woff2) format('woff2'), url(./files/a.woff) format('woff');`,
      "../fonts/",
    );
    expect(css).toBe(
      `src: url(../fonts/a.woff2) format('woff2'), url(../fonts/a.woff) format('woff');`,
    );
    expect(files).toEqual(["a.woff2", "a.woff"]);
  });

  it("reports each file once even when several faces share it", () => {
    const { files } = rewriteFontUrls(
      `url(./files/a.woff2) url(./files/a.woff2)`,
      "../fonts/",
    );
    expect(files).toEqual(["a.woff2"]);
  });

  // Fontsource writes every url as ./files/… — anything else means the package
  // layout moved under us, and copying nothing is worse than stopping.
  it("throws on a url it cannot repoint", () => {
    expect(() =>
      rewriteFontUrls(
        `src: url(https://fonts.gstatic.com/a.woff2);`,
        "../fonts/",
      ),
    ).toThrow(/not a Fontsource file url/);
  });

  it("throws when a stylesheet declares no faces at all", () => {
    expect(() => rewriteFontUrls(`/* nothing here */`, "../fonts/")).toThrow(
      /no font files/,
    );
  });
});
