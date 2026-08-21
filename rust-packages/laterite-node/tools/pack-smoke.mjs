// Packaging smoke — the guard that would have caught the missing-napi-loader
// publish break (and any "the tarball doesn't actually run" regression).
//
// The `node` CI job builds the addon itself (`napi build`), so the napi loader
// (index.js) + types (index.d.ts) are always present there — which is exactly
// why a publish-only gap (the publish job consumes prebuilt .nodes and never
// runs `napi build`) was invisible to CI. This test mirrors what `npm install
// laterite` actually ships and runs:
//
//   1. build the dual ESM/CJS dist from the *committed* loader (no `napi build`),
//   2. `npm pack` and assert the tarball carries the runtime loader + dist,
//   3. extract it, drop in the locally-built platform `.node` (the bit a real
//      install pulls from `@laterite/native-*`),
//   4. require the packed CJS *and* import the packed ESM, exercising
//      read / table (born-typed) / validate / buildAgs4 end to end, and
//   5. spawn the packed `lat` console script (bin.mjs → dist/cli.mjs) so the
//      [bin] entry point — a real `npm install` puts on PATH — is proven to run.
//
// apache-arrow (a real runtime dep) is resolved via NODE_PATH from the dev
// node_modules — the stand-in for it being installed alongside the package.
import { execFileSync } from "node:child_process";
import {
  mkdtempSync,
  rmSync,
  copyFileSync,
  writeFileSync,
  readdirSync,
  symlinkSync,
  existsSync,
} from "node:fs";
import { tmpdir } from "node:os";
import { join, resolve } from "node:path";

const pkgDir = resolve(import.meta.dirname, "..");
const run = (cmd, args, opts = {}) =>
  execFileSync(cmd, args, { cwd: pkgDir, encoding: "utf8", ...opts });

// 1 — dist from the committed loader (this is the publish job's build step).
run("npm", ["run", "build:ts"], { stdio: "inherit" });

// 2 — pack + assert the tarball contents.
const packed = JSON.parse(run("npm", ["pack", "--json"]));
const tgz = packed[0].filename;
const entries = packed[0].files.map((f) => f.path);
const required = [
  "index.js",
  "index.d.ts",
  "dist/index.cjs",
  "dist/index.mjs",
  "dist/index.d.ts",
  // The `lat` console script (`bin: { lat: "./bin.mjs" }`) and the CLI module it
  // imports `main()` from. `npm install laterite` puts `lat` on PATH via bin.mjs,
  // so a tarball that ships the library but drops these publishes a broken `lat`
  // — a gap the library-only smoke below never saw (laterite-dev#554).
  "bin.mjs",
  "dist/cli.mjs",
];
for (const f of required) {
  if (!entries.includes(f)) {
    throw new Error(`tarball is missing ${f} (would ship a broken package)`);
  }
}
console.log(`tarball ${tgz}: ${entries.length} files, loader + dist present ✓`);

// 3 — extract + drop in the locally-built platform `.node`.
const work = mkdtempSync(join(tmpdir(), "laterite-pack-"));
try {
  run("tar", ["-xzf", tgz, "-C", work]);
  const root = join(work, "package");
  const localNode = readdirSync(pkgDir).find(
    (f) => f.startsWith("laterite-node.") && f.endsWith(".node"),
  );
  if (!localNode)
    throw new Error("no locally-built laterite-node.*.node found");
  copyFileSync(join(pkgDir, localNode), join(root, localNode));
  // The package's runtime deps (apache-arrow) sit in node_modules next to it in
  // a real install — mirror that so BOTH require (CJS) and import (ESM, which
  // ignores NODE_PATH) resolve them the normal node_modules-walking way.
  symlinkSync(join(pkgDir, "node_modules"), join(root, "node_modules"), "dir");

  // The fixture (same born-typed columns the P1 flagship asserts).
  const ags =
    '"GROUP","LOCA"\r\n' +
    '"HEADING","LOCA_ID","LOCA_GL"\r\n' +
    '"UNIT","","m"\r\n' +
    '"TYPE","ID","2DP"\r\n' +
    '"DATA","BH01","12.30"\r\n';

  // 4 — exercise the packed CJS + ESM entries against the real native binary.
  const assertions = (entry, importer) => `
    const assert = require("node:assert");
    ${importer}.then((m) => {
      const file = m.read(undefined, { text: ${JSON.stringify(ags)} });
      assert.ok(file.groups.includes("LOCA"), "groups");
      const loca = file.table("LOCA");
      assert.strictEqual(loca.getChild("LOCA_GL").get(0), 12.3, "born-typed 2DP");

      const rep = m.validate(undefined, { text: ${JSON.stringify(ags)} });
      assert.strictEqual(typeof rep.isValid, "boolean", "validate");
      assert.ok(rep.findings.length > 0, "findings");

      const res = m.buildAgs4(
        new Map([["LOCA", [{ LOCA_ID: "BH01", LOCA_GL: 12.3 }]]]),
        { dictVersion: "4.1.1", mode: "autofix" },
      );
      assert.ok(res.text.includes("LOCA"), "emit");
      console.log("  ${entry}: read+table+validate+emit OK ✓");
    }).catch((e) => { console.error(e); process.exit(1); });
  `;
  // CJS entry (require) and ESM entry (dynamic import) — both must load + run.
  writeFileSync(
    join(root, "_smoke.cjs"),
    assertions("CJS", 'Promise.resolve(require("./dist/index.cjs"))'),
  );
  run("node", ["_smoke.cjs"], { cwd: root, stdio: "inherit" });
  writeFileSync(
    join(root, "_smoke-esm.cjs"),
    assertions("ESM", 'import("./dist/index.mjs")'),
  );
  run("node", ["_smoke-esm.cjs"], { cwd: root, stdio: "inherit" });

  // 5 — spawn the packed `lat` console script. `bin.mjs` imports `main()` from
  // `dist/cli.mjs`; running it end-to-end (through the dropped-in native `.node`)
  // proves the [bin] entry point resolves and runs from the tarball — a DIFFERENT
  // resolution path than the library `import`/`require` above (laterite-dev#554: the console
  // script rots while the library gate stays green).
  writeFileSync(join(root, "_cli-fixture.ags"), ags);
  const cliOut = run(
    "node",
    ["bin.mjs", "read", "_cli-fixture.ags", "--json"],
    {
      cwd: root,
    },
  );
  const groups = JSON.parse(cliOut);
  if (!Array.isArray(groups) || !groups.includes("LOCA")) {
    throw new Error(
      `packed lat CLI: unexpected \`read --json\` output: ${cliOut}`,
    );
  }
  console.log("  CLI: packed `lat read --json` OK ✓");

  console.log(
    "packaging smoke: the packed library AND `lat` CLI load and run ✓",
  );
} finally {
  rmSync(work, { recursive: true, force: true });
  if (existsSync(join(pkgDir, tgz))) rmSync(join(pkgDir, tgz));
}
