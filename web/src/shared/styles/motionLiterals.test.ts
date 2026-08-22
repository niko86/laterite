/* No raw motion literals on the landing (#534).
 *
 * The motion layer's whole contract is that call sites state WHICH properties
 * transition and the tokens decide how long and on what curve — that is what
 * makes reduced-motion support inherited rather than re-implemented. A raw
 * millisecond or bezier in a class string opts that element out of the
 * collapse silently. This scans the landing's components; the app's own
 * surfaces are #411's other tickets.
 *
 * Scope, stated per the gates-say-what-they-drop rule: only .tsx/.ts sources
 * under web/landing are read; CSS files, tests, and the other surfaces are
 * not this gate's problem.
 */

import { readFileSync, readdirSync, statSync } from "node:fs";
import { join } from "node:path";
import { fileURLToPath } from "node:url";
import { describe, expect, it } from "vitest";

const LANDING = fileURLToPath(new URL("../../../landing", import.meta.url));

function sources(dir: string): string[] {
  const out: string[] = [];
  for (const name of readdirSync(dir)) {
    if (name === "dist" || name === "node_modules") continue;
    const path = join(dir, name);
    if (statSync(path).isDirectory()) out.push(...sources(path));
    else if (/\.(tsx|ts)$/.test(name) && !/\.test\.tsx?$/.test(name))
      out.push(path);
  }
  return out;
}

/** Arbitrary-value motion utilities, Tailwind's bracket-free numeric ones
 *  (`duration-150` emits raw ms exactly like `duration-[150ms]`), and raw
 *  curves. `duration-(--var)` (the paren token form) stays legal — it names
 *  a token. */
const RAW =
  /duration-\[|duration-\d|ease-\[|delay-\[|delay-\d|cubic-bezier\(|transition:\s/;

describe("the landing states no raw motion values", () => {
  it("every duration and easing resolves through a token", () => {
    const files = sources(LANDING);
    // A filter nobody can see is a blind spot with a green tick on it: say
    // what was scanned, and refuse to pass over an empty set.
    expect(files.length).toBeGreaterThan(0);
    console.info(`motionLiterals: scanned ${files.length} landing source(s)`);
    const offenders = files.filter((path) =>
      RAW.test(readFileSync(path, "utf8")),
    );
    expect(offenders).toEqual([]);
  });
});
