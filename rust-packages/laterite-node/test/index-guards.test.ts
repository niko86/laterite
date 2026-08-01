// The guards on `index.ts`'s public entry points — the arms that turn a mistake
// into a typed, readable failure instead of letting a raw runtime error escape.
//
// These are all error paths, which is exactly why they were uncovered: the happy
// path is what every other suite exercises. But an error path that has never run is
// where a `throw` referencing an undefined variable, or a mapping that returns the
// wrong class, lives undisturbed — and the caller only finds out in production, at
// the moment they were already having a bad day.
import { mkdtempSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { fileURLToPath } from "node:url";
import { readFileSync } from "node:fs";
import { describe, expect, it } from "vitest";

/** Run `fn` and hand back whatever it threw. Keeps assertions OUT of a catch
 *  block — an `expect` in there is skipped entirely when the call unexpectedly
 *  succeeds, so the test passes by not running (vitest/no-conditional-expect). */
function thrownBy(fn: () => unknown): unknown {
  try {
    fn();
  } catch (e) {
    return e;
  }
  return undefined;
}

import {
  Ags4Error,
  FileNotFoundError,
  buildAgs4,
  diff,
  fix,
  read,
} from "../ts/index";
import { AgsGroup } from "../ts/typed-graph";

const CLEAN_PATH = fileURLToPath(
  new URL(
    "../../laterite-ags4-validator/tests/fixtures/clean_minimal.ags",
    import.meta.url,
  ),
);
const CLEAN = readFileSync(CLEAN_PATH);
const tmp = () => mkdtempSync(join(tmpdir(), "laterite-guards-"));

describe("a path that is not there", () => {
  it("becomes FileNotFoundError, not a raw ENOENT", () => {
    // `diffBytes` reads paths itself precisely so a missing one arrives as the
    // mapped class. Left unmapped, callers would have to sniff `err.code` — the
    // brittle message/field matching this whole error module exists to remove.
    const missing = join(tmp(), "not-here.ags");
    const e = thrownBy(() => diff(missing, CLEAN));
    expect(e).toBeInstanceOf(FileNotFoundError);
    expect((e as FileNotFoundError).exitCode).toBe(3);
    // The path is IN the message — a bare "No such file or directory" from a
    // two-input operation cannot tell you which side was missing.
    expect((e as FileNotFoundError).message).toContain(missing);
  });

  it("names the missing side when it is the second input", () => {
    const missing = join(tmp(), "also-not-here.ags");
    expect(() => diff(CLEAN, missing)).toThrow(
      new RegExp(missing.replace(/[.*+?^${}()|[\]\\]/g, "\\$&")),
    );
  });

  it("re-throws a read failure that is NOT a missing file", () => {
    // The `if ENOENT` arm is a narrowing, not a catch-all: a directory, a
    // permission error or a bad handle must propagate as itself. Mapping every
    // read failure to FileNotFoundError would tell a user to check the path when
    // the path was fine.
    const dir = tmp(); // a directory, not a file → EISDIR/EPERM, never ENOENT
    const caught = thrownBy(() => diff(dir, CLEAN));
    expect(caught).toBeDefined();
    expect(caught).not.toBeInstanceOf(FileNotFoundError);
  });

  it("accepts an already-read handle without touching the filesystem", () => {
    // The `instanceof Ags4File` arm short-circuits before any read, so a handle
    // built from bytes — which has no path at all — is a legal diff input.
    const handle = read(CLEAN);
    expect(() => diff(handle, handle)).not.toThrow();
  });
});

describe("buildAgs4 refuses a typed-graph node it cannot place", () => {
  // The guard lives on the AgsGroup walk (`walkTree`), which is only entered when
  // the argument really is an AgsGroup — the Map/array forms take a different path.
  // So these subclass the real base rather than passing a look-alike object.

  it("refuses a node whose class carries no group code", () => {
    // Every generated group class has a static `code`. A hand-rolled subclass has
    // none, and without the guard the walk would look up `registry.get(undefined)`
    // and fail much later with no mention of the offending node.
    class Anonymous extends AgsGroup {}
    expect(() => buildAgs4(new Anonymous())).toThrow(Ags4Error);
    expect(() => buildAgs4(new Anonymous())).toThrow(
      /not a known typed AGS group/,
    );
  });

  it("refuses a code the registry has never heard of", () => {
    // The guard checks the code AND its descriptor, so a plausible-looking
    // four-letter code that no edition defines cannot slip through on shape alone.
    class Impostor extends AgsGroup {
      static code = "ZZZZ";
    }
    expect(() => buildAgs4(new Impostor())).toThrow(
      /not a known typed AGS group/,
    );
  });
});

describe("fix maps an engine failure to the typed error", () => {
  it("throws the mapped class for a file that is not there", () => {
    const missing = join(tmp(), "nope.ags");
    const e = thrownBy(() => fix(missing));
    // Whatever kind the engine reported, it must arrive as an Ags4Error subclass
    // carrying the engine's own exit code — not a generic Error with exitCode
    // undefined, which would become a 0 exit and report failure as success.
    expect(e).toBeInstanceOf(Ags4Error);
    expect(typeof (e as Ags4Error).exitCode).toBe("number");
    expect((e as Ags4Error).exitCode).toBeGreaterThan(0);
  });

  it("throws for input that is not AGS4 at all", () => {
    const p = join(tmp(), "prose.ags");
    writeFileSync(p, "this file has no GROUP rows whatsoever\r\n");
    expect(() => fix(p)).toThrow(Ags4Error);
  });
});
