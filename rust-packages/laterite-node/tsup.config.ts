import { defineConfig } from "tsup";

// Builds the shippable JS — dual ESM (`dist/index.mjs`) + CJS (`dist/index.cjs`)
// + types (`dist/index.d.ts`) from `ts/index.ts`. The napi loader (`#native`,
// resolved at runtime via package.json `imports`) and the heavy deps stay
// external: the loader requires the platform `.node` package, apache-arrow is a
// runtime dep, and @duckdb/node-api is the optional peer (lazy-imported).
export default defineConfig({
  entry: ["ts/index.ts", "ts/cli.ts"],
  format: ["esm", "cjs"],
  dts: true,
  clean: true,
  outDir: "dist",
  target: "node18",
  external: ["#native", "apache-arrow", "@duckdb/node-api"],
  outExtension({ format }) {
    return { js: format === "esm" ? ".mjs" : ".cjs" };
  },
});
