import { defineConfig } from "vitest/config";

export default defineConfig({
  test: {
    include: ["test/**/*.test.ts"],
    // The napi loader (`index.js`) and the `.node` binary are native — keep Vite
    // from transforming them; require() them as-is in the node runtime.
    server: { deps: { external: [/index\.js$/, /\.node$/] } },
  },
});
