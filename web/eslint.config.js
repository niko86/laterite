// ESLint 9 flat config — type-aware linting for the web (SolidJS + Vite) app.
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
import js from "@eslint/js";
import tseslint from "typescript-eslint";
import solid from "eslint-plugin-solid/configs/recommended";
import vitest from "@vitest/eslint-plugin";
import prettier from "eslint-config-prettier";
import globals from "globals";

export default tseslint.config(
  {
    ignores: [
      "dist/**",
      "node_modules/**",
      "src/wasm/**",
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
);
