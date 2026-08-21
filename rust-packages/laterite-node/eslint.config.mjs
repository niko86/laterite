// ESLint 9 flat config — type-aware linting for the laterite-node TS surface.
//
// `projectService` wires every file to laterite-node/tsconfig.json so the
// type-aware typescript-eslint rules (no-floating-promises, no-misused-promises,
// no-unnecessary-condition, …) can see real types. eslint-config-prettier is
// last so Prettier owns formatting.
//
// Named .mjs (not .js) because this package is `"type": "commonjs"` — a
// `eslint.config.js` here would be parsed as CommonJS and reject this ESM syntax.
//
// Scope: hand-written TypeScript only. The napi-generated loader/types
// (index.js, index.d.ts), the drift-guarded generated tables
// (ts/registry.generated.ts, ts/typed-graph.generated.ts), the tsup build
// output (dist/) and the JS tooling (bin.mjs, tools/*.mjs) are all out of scope.
import js from "@eslint/js";
import tseslint from "typescript-eslint";
import vitest from "@vitest/eslint-plugin";
import prettier from "eslint-config-prettier";
import globals from "globals";

export default tseslint.config(
  {
    ignores: [
      "dist/**",
      "node_modules/**",
      "**/*.d.ts",
      "ts/*.generated.ts",
      "**/*.js",
      "**/*.mjs",
      "**/*.cjs",
    ],
  },
  js.configs.recommended,
  ...tseslint.configs.strictTypeChecked,
  {
    files: ["**/*.ts"],
    languageOptions: {
      globals: { ...globals.node },
      parserOptions: {
        // tsup.config.ts / vitest.config.ts sit outside tsconfig.json's
        // `include`; allowDefaultProject type-checks them via an inferred
        // program so the project service doesn't error on them.
        projectService: {
          allowDefaultProject: ["*.config.ts"],
        },
        tsconfigRootDir: import.meta.dirname,
      },
    },
  },
  {
    files: ["test/**/*.test.ts"],
    ...vitest.configs.recommended,
  },
  prettier,
  {
    // laterite-dev#615 burn-down: the many `${count}`-style numeric interpolations are safe
    // (numbers stringify losslessly), so allowNumber lets them through; the real
    // offenders (a JSON.parse `any` in cli.ts, a `string[]` in a test message)
    // were fixed at source. Back at `error`.
    name: "ratchet/restrict-template-expressions",
    rules: {
      "@typescript-eslint/restrict-template-expressions": [
        "error",
        { allowNumber: true },
      ],
    },
  },
  {
    // laterite-dev#615: the type-boundary family is burned down to zero and enforced at
    // `error` in shipped source, but test files legitimately assert-then-access
    // (`x!` after an `expect`) and poke untyped boundaries (JSON.parse, native
    // returns) — grinding those into narrows adds bulk with no safety gain, since
    // a wrong assumption just fails the test loudly. The rules' payoff is catching
    // a hidden null/`any` in code that SHIPS; a test is the check, not the risk.
    // So the family is off in tests only; source stays strict.
    name: "ratchet/tests-allow-type-boundary",
    files: ["test/**/*.ts"],
    rules: {
      "@typescript-eslint/no-non-null-assertion": "off",
      "@typescript-eslint/no-unsafe-member-access": "off",
      "@typescript-eslint/no-unsafe-call": "off",
      "@typescript-eslint/no-unsafe-assignment": "off",
      "@typescript-eslint/no-unsafe-return": "off",
      "@typescript-eslint/no-unsafe-argument": "off",
    },
  },
);
