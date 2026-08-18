// ESLint 10 flat config — type-aware linting for the web (SolidJS + Vite) app.
//
// eslint-plugin-solid stopped at eslint 9 in its peer range (last release
// 2024-12), so package.json carries an `overrides` entry relaxing that peer to
// whatever eslint the root resolves. That is a deliberate, verified override,
// not a shrug: the plugin's rules were confirmed to still FIRE under eslint 10
// (a solid/reactivity violation is still caught), which is the failure mode an
// override could otherwise hide. Drop the override the day upstream ships a
// release that declares ^10.
//
// The point of this config is the *type-aware* typescript-eslint ruleset
// (no-floating-promises, no-misused-promises, no-unnecessary-condition, …):
// `projectService` wires every file to web/tsconfig.json so those rules can see
// real types. Prettier owns formatting — eslint-config-prettier is last, so it
// switches off every stylistic rule ESLint would otherwise fight it over.
//
// Scope: hand-written TypeScript only. Generated wasm-bindgen glue (src/wasm*),
// build output (dist), vendored deps and *.d.ts are ignored — never hand-edited.
// Plain-JS build scripts (scripts/*.mjs) are out of scope for this TS-focused
// pass; Prettier still formats them (see .prettierignore).
import { readFileSync } from "node:fs";

import js from "@eslint/js";
import tseslint from "typescript-eslint";
import solid from "eslint-plugin-solid/configs/recommended";
import vitest from "@vitest/eslint-plugin";
import prettier from "eslint-config-prettier";
import globals from "globals";

// The raw-palette gate (#404). Application colour resolves through the shared
// tokens (ok/warn/err/info/accent/cta + their -quiet washes, the laterite
// ramp, the neutral roles) — a raw Tailwind palette class (`bg-emerald-600`,
// `text-sky-50`) or a pasted hex (`#0ea5e9` was sky-500 wearing a disguise)
// bypasses the vocabulary and breaks in one theme or the other. The banned
// family names are READ OUT OF tailwindcss's own shipped theme rather than
// restated here, so the list tracks the vendor instead of rotting against it.
// The `(?<!--)` guard keeps token *references* (`var(--stone-100)`,
// `text-[--steel-500]`) legal: our own vars share family names with Tailwind's.
const tailwindFamilies = [
  ...new Set(
    [
      ...readFileSync(
        new URL("./node_modules/tailwindcss/theme.css", import.meta.url),
        "utf8",
      ).matchAll(/--color-([a-z]+)-\d+/g),
    ].map((m) => m[1]),
  ),
];
const RAW_PALETTE_RE = new RegExp(
  `(?<!--)\\b(?:${tailwindFamilies.join("|")})-\\d\\d\\d?\\b`,
);
// 6/8-digit forms only: issue references ("#448", "#1024") share the 3- and
// 4-digit shape, and the app writes no shorthand hex.
const RAW_HEX_RE = /#(?:[0-9a-fA-F]{6}|[0-9a-fA-F]{8})\b/;
// A status/accent/cta BACKGROUND with an alpha is a hand-mixed tint — the
// -quiet token is the sanctioned wash (#404, AC "tinted fills use the -quiet
// tokens"). Backgrounds only: border/text alpha (border-ok\/45, border-err\/60)
// is the system's own idiom for hairlines and stays legal.
const MIXED_TINT_RE =
  /\bbg-(?:ok|warn|err|info|accent|cta)(?:-hover|-quiet)?\/\d/;
const noRawPalette = {
  meta: {
    type: "problem",
    schema: [],
    messages: {
      palette:
        "Raw Tailwind palette class '{{match}}' — status, accent and tinted " +
        "fills resolve through the shared tokens (ok/warn/err/info/accent/cta " +
        "+ -quiet washes; see web/src/shared/styles/colors.css, #404).",
      hex:
        "Raw colour literal '{{match}}' — resolve it from a token instead " +
        "(getComputedStyle(...).getPropertyValue('--…') where a class cannot " +
        "carry it; see web/src/shared/styles/colors.css, #404).",
      mixedTint:
        "Hand-mixed tint '{{match}}' — tinted fills take the status's -quiet " +
        "token (bg-ok-quiet, bg-err-quiet, …), not an alpha on the solid " +
        "token (#404).",
    },
  },
  create(context) {
    const check = (node, text) => {
      const palette = RAW_PALETTE_RE.exec(text);
      if (palette) {
        context.report({
          node,
          messageId: "palette",
          data: { match: palette[0] },
        });
        return;
      }
      const hex = RAW_HEX_RE.exec(text);
      if (hex) {
        context.report({ node, messageId: "hex", data: { match: hex[0] } });
        return;
      }
      const tint = MIXED_TINT_RE.exec(text);
      if (tint) {
        context.report({
          node,
          messageId: "mixedTint",
          data: { match: tint[0] },
        });
      }
    };
    return {
      Literal(node) {
        if (typeof node.value === "string") check(node, node.value);
      },
      TemplateElement(node) {
        check(node, node.value.raw);
      },
    };
  },
};

export default tseslint.config(
  {
    ignores: [
      "dist/**",
      "node_modules/**",
      "src/wasm/**",
      "src/wasm-full/**",
      "src/wasm-tokenizer/**",
      // Playwright e2e specs live outside web/tsconfig.json's `include` (they
      // aren't in `tsc --noEmit` scope either) and use their own runner — out
      // of scope for this pass. Give them a dedicated tsconfig + lint block if
      // they're brought in later.
      "e2e/**",
      "**/*.d.ts",
      "**/*.js",
      "**/*.mjs",
      "**/*.cjs",
    ],
  },
  js.configs.recommended,
  ...tseslint.configs.strictTypeChecked,
  {
    files: ["**/*.{ts,tsx}"],
    languageOptions: {
      globals: { ...globals.browser },
      parserOptions: {
        // These root config files live outside web/tsconfig.json's `include`;
        // allowDefaultProject type-checks them via an inferred program so the
        // project service doesn't error on them. vite.config.ts is deliberately
        // NOT listed — it IS in `include`, and listing an in-project file here
        // is itself an error.
        projectService: {
          allowDefaultProject: [
            "vitest.config.ts",
            "vitest.wasm.config.ts",
            "vitest.wasm-bench.config.ts",
            "vitest.wasm.setup.ts",
            "playwright.config.ts",
          ],
        },
        tsconfigRootDir: import.meta.dirname,
      },
    },
  },
  {
    // Web workers see the worker global scope, not the DOM window.
    files: ["**/*.worker.ts"],
    languageOptions: { globals: { ...globals.worker } },
  },
  {
    files: ["**/*.{ts,tsx}"],
    ...solid,
  },
  {
    files: ["**/*.test.{ts,tsx}"],
    ...vitest.configs.recommended,
  },
  prettier,
  {
    // The gate itself (#404) — application source only. The landing surface
    // still carries stone-* classes (its own tickets' territory) and the
    // config/scripts layer legitimately names colours (this file, the tokens'
    // build plumbing), so the scope is exactly the app the ticket covers.
    name: "design/no-raw-palette",
    files: ["src/**/*.{ts,tsx}"],
    plugins: {
      design: { rules: { "no-raw-palette": noRawPalette } },
    },
    rules: {
      "design/no-raw-palette": "error",
    },
  },
  {
    // #615 burn-down: the many `${count}`-style numeric interpolations are safe
    // (numbers stringify losslessly), so allowNumber lets them through; the only
    // real offenders were three `string | undefined` regex capture groups in
    // agsTypeInfo.ts, fixed at source. Back at `error` — a genuinely unstringify-
    // able value (object, nullish) in a template is now a hard failure.
    name: "ratchet/restrict-template-expressions",
    rules: {
      "@typescript-eslint/restrict-template-expressions": [
        "error",
        { allowNumber: true },
      ],
    },
  },
  {
    // #615: the type-boundary family is burned down to zero and enforced at
    // `error` in shipped source, but test files legitimately assert-then-access
    // (`x!` after an `expect`) and poke untyped boundaries (JSON.parse, native
    // returns) — grinding those into narrows adds bulk with no safety gain, since
    // a wrong assumption just fails the test loudly. The rules' payoff is catching
    // a hidden null/`any` in code that SHIPS; a test is the check, not the risk.
    // So the family is off in tests only; source stays strict.
    name: "ratchet/tests-allow-type-boundary",
    files: ["**/*.test.{ts,tsx}"],
    rules: {
      "@typescript-eslint/no-non-null-assertion": "off",
      "@typescript-eslint/no-unsafe-member-access": "off",
      "@typescript-eslint/no-unsafe-call": "off",
      "@typescript-eslint/no-unsafe-assignment": "off",
      "@typescript-eslint/no-unsafe-return": "off",
      "@typescript-eslint/no-unsafe-argument": "off",
    },
  },
  {
    // #615: the two ratcheted eslint-plugin-solid rules are burned to zero. Every
    // current flag was verified a false positive (stable <For>-keyed / remounted-
    // per-result props, function-prop aliases, one-shot init callbacks, imperative
    // handlers) and scoped-disabled with a reason at its site, so flip both from
    // the plugin's recommended `warn` to `error` — a NEW reactivity smell is now a
    // hard failure that must be fixed or justified with a scoped disable.
    name: "ratchet/solid-reactivity",
    files: ["**/*.{ts,tsx}"],
    rules: {
      "solid/reactivity": "error",
      "solid/components-return-once": "error",
    },
  },
  {
    // eslint 10 added `no-unassigned-vars`, which flags a `let` that is declared
    // and never written. All four hits here are Solid **ref bindings**:
    //
    //     let el!: HTMLDivElement;
    //     <div ref={el}>            // the JSX compiler does the assignment
    //
    // The rule cannot see that write, and the definite-assignment `!` does not
    // satisfy it either — three of the four already carry one and were still
    // flagged. So it is off rather than papered over with four copies of this
    // explanation (and a fifth the next time someone adds a ref).
    //
    // Scoped to `.tsx` deliberately: a ref binding is a JSX construct, so in a
    // plain `.ts` module a never-assigned `let` really is the bug the rule is
    // for, and stays a hard failure there.
    name: "solid/ref-bindings-are-not-unassigned",
    files: ["**/*.tsx"],
    rules: {
      "no-unassigned-vars": "off",
    },
  },
);
