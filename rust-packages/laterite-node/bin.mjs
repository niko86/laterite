#!/usr/bin/env node
// The `lat` executable for the npm `laterite` package — the third launcher of the
// one AGS4 tool (`npx laterite …`, or `lat …` when installed). A thin shim over
// the tsup-built CLI so the testable `main()` stays import-clean.
import { main } from "./dist/cli.mjs";

// Set exitCode and let the event loop drain the output streams rather than
// `process.exit(main())`. process.exit() terminates the process before Node
// flushes an ASYNC stdout — which is what a pipe is — so large output (e.g.
// `rules --json`, ~24 KB) gets truncated at the pipe buffer boundary (~8 KB)
// whenever the reader is even slightly slow (seen intermittently in CI's
// cross-surface capture). The CLI's napi calls are synchronous and register no
// async handles, so once stdout/stderr have drained the process exits on its
// own with this code.
process.exitCode = main();
