#!/usr/bin/env node
// The `lat` executable for the npm `laterite` package — the third launcher of the
// one AGS4 tool (`npx laterite …`, or `lat …` when installed). A thin shim over
// the tsup-built CLI so the testable `main()` stays import-clean.
import { main } from "./dist/cli.mjs";

process.exit(main());
