// The error-mapping table — the contract callers branch on.
//
// `errors.ts` says it plainly: the engine "reports a `kind` + `exit_code` that
// these map to the right error class ... so callers can branch on the type, not a
// brittle message match". Nothing tested that map. It was covered only incidentally,
// by tests that happened to trigger some of its arms.
//
// The drift this guards is specific and silent. `ValidatorError::kind()` calls
// itself "the single PRODUCER of the error-kind value domain — every surface
// delegates here instead of re-mapping the variants by hand, so the tables can't
// drift", and a new variant DOES force that Rust match. But nothing forces
// `makeError` to keep up: an unmapped kind falls through to `default:` and returns a
// base `Ags4Error`. The message is still right, the exit code is still right, and
// every `instanceof` a caller wrote returns false. So the test below reads the
// producers OUT OF THE RUST SOURCE rather than restating them here — a third
// hand-list is exactly what this repo keeps getting caught by.
import { readFileSync } from "node:fs";
import { resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { describe, expect, it } from "vitest";

import {
  Ags4Error,
  BadDictError,
  FileNotFoundError,
  MergeConflictError,
  NotAgs4Error,
  UnsupportedEditionError,
  WorldCheckRequiresSourceError,
  fromNativeError,
  makeError,
} from "../ts/errors";

const repo = resolve(fileURLToPath(new URL("../../..", import.meta.url)));
const read = (p: string) => readFileSync(resolve(repo, p), "utf8");

/** Every kind this surface can actually receive, read from its two producers:
 *  the shared engine enum, and the napi layer's own locally-thrown literals. */
function producedKinds(): string[] {
  const engine = read("rust-packages/laterite-ags4-validator/src/error.rs");
  // The `kind()` match arms: `ValidatorError::X(..) => "not_found",`
  const fromEngine = [
    ...engine.matchAll(/ValidatorError::[^=]+=>\s*"([a-z_]+)"/g),
  ]
    .map((m) => m[1])
    .filter((k): k is string => k !== undefined);
  const napi = read("rust-packages/laterite-node/src/lib.rs");
  // The napi layer throws a few of its own: `"bad_args{SEP}5{SEP}..."`.
  const fromNapi = [...napi.matchAll(/"([a-z_]+)\{SEP\}/g)]
    .map((m) => m[1])
    .filter((k): k is string => k !== undefined);
  return [...new Set([...fromEngine, ...fromNapi])].sort();
}

describe("the error-kind map is not behind its producers", () => {
  it("finds the producers at all", () => {
    // If a refactor moves `kind()` or changes how the napi layer throws, the
    // regexes above quietly match nothing and the whole guard passes vacuously —
    // the failure mode of every source-scraping test.
    const kinds = producedKinds();
    // Zero scraped kinds would make the guard below pass vacuously.
    expect(kinds.length).toBeGreaterThanOrEqual(6);
    expect(kinds).toContain("not_found");
    expect(kinds).toContain("merge_conflict");
  });

  it("maps every produced kind to a SPECIFIC class, never the base", () => {
    const unmapped = producedKinds().filter((kind) => {
      const e = makeError(kind, 1, "x");
      // The `default:` arm returns a base Ags4Error. Any real mapping returns a
      // subclass, so an exact constructor match means the kind fell through.
      return e.constructor === Ags4Error;
    });
    // A failure here means those kinds reach callers as a plain Ags4Error, so every
    // `instanceof` they wrote is false while the message and exit code still look
    // correct — the quietest possible way for this contract to break.
    expect(unmapped).toEqual([]);
  });
});

describe("makeError", () => {
  // The kind -> class contract, stated as the specification it is. Paired with the
  // drift guard above: that one proves the list is complete, this one proves each
  // entry is right.
  it.each([
    ["not_found", FileNotFoundError],
    ["io", FileNotFoundError],
    ["not_ags4", NotAgs4Error],
    ["not_utf8", NotAgs4Error],
    ["unsupported_edition", UnsupportedEditionError],
    ["bad_dict", BadDictError],
    ["bad_args", BadDictError],
    ["world_check_requires_source", WorldCheckRequiresSourceError],
    ["merge_conflict", MergeConflictError],
    ["emit_error", MergeConflictError],
  ])("maps %s", (kind, cls) => {
    const e = makeError(kind, 7, "boom");
    expect(e).toBeInstanceOf(cls);
    expect(e.message).toBe("boom");
    // The exit code comes from the ENGINE, not the class default — the class
    // defaults are only a fallback for hand-construction.
    expect(e.exitCode).toBe(7);
  });

  it("returns a usable error for a kind it has never heard of", () => {
    // Not a hypothetical: this is what a newly-added engine variant looks like
    // here before someone updates the table. It must still carry the message and
    // exit code — degrading to a generic error is survivable, losing the failure
    // is not.
    const e = makeError("a_kind_from_the_future", 9, "something new broke");
    expect(e).toBeInstanceOf(Ags4Error);
    expect(e.message).toBe("something new broke");
    expect(e.exitCode).toBe(9);
  });

  it("gives each class the exit code `lat` uses for it", () => {
    // Hand-constructed (no engine code supplied), so these are the defaults, and
    // they are byte-faithful to the CLI's documented exit codes.
    expect(new Ags4Error("x").exitCode).toBe(1);
    expect(new FileNotFoundError("x").exitCode).toBe(3);
    expect(new NotAgs4Error("x").exitCode).toBe(4);
    expect(new UnsupportedEditionError("x").exitCode).toBe(4);
    expect(new BadDictError("x").exitCode).toBe(5);
    expect(new MergeConflictError("x").exitCode).toBe(6);
  });

  it("names every class after itself, so a caught error prints usefully", () => {
    // `Error.name` is what shows in a stack trace and in `String(e)`. Left as the
    // inherited "Error" it would make every one of these indistinguishable in a log.
    expect(new UnsupportedEditionError("x").name).toBe(
      "UnsupportedEditionError",
    );
    expect(new MergeConflictError("x").name).toBe("MergeConflictError");
    expect(new WorldCheckRequiresSourceError("x").name).toBe(
      "WorldCheckRequiresSourceError",
    );
  });
});

describe("fromNativeError recovers the class from the wire format", () => {
  // `parseArrow` returns a HANDLE, so it cannot return a failure report — it
  // throws `kind␟code␟message` instead. This is the only thing that turns that
  // string back into the typed error the other protocol delivers directly.
  // Written as the escape, not the raw character: it is invisible in an editor,
  // and a stray copy/paste of the literal would silently make every case below
  // take the generic arm while still passing its own weaker assertions.
  const SEP = "\u001f";

  it("round-trips a thrown kind␟code␟message", () => {
    const e = fromNativeError(
      new Error(`unsupported_edition${SEP}4${SEP}AGS3 refused`),
    );
    expect(e).toBeInstanceOf(UnsupportedEditionError);
    expect(e.exitCode).toBe(4);
    expect(e.message).toBe("AGS3 refused");
  });

  it("keeps a message containing the separator's own text intact", () => {
    // split() on 3 parts is exact: a message that itself contains the separator
    // would produce 4 parts and silently fall through to the generic arm, losing
    // the class. Pinned so a future "join the tail back" change is deliberate.
    const e = fromNativeError(new Error(`io${SEP}3${SEP}a${SEP}b`));
    expect(e.constructor).toBe(Ags4Error);
    expect(e.message).toBe(`io${SEP}3${SEP}a${SEP}b`);
  });

  it("passes an ordinary throw through as a generic error", () => {
    const e = fromNativeError(new Error("segfault in something unrelated"));
    expect(e.constructor).toBe(Ags4Error);
    expect(e.message).toBe("segfault in something unrelated");
    expect(e.exitCode).toBe(1);
  });

  it("survives a non-Error throw", () => {
    // napi normally throws Errors, but JS lets anything be thrown, and this runs
    // in the catch path of a native call — the one place a surprise is likely.
    expect(fromNativeError("just a string").message).toBe("just a string");
    expect(fromNativeError(undefined).message).toBe("undefined");
    expect(fromNativeError({ nope: true }).message).toBe("[object Object]");
  });

  it("falls back to exit code 1 when the code is not a number", () => {
    // The `|| 1` in fromNativeError. A NaN exitCode would propagate into
    // `process.exitCode` and become 0 — a failure reported as success.
    const e = fromNativeError(
      new Error(`not_ags4${SEP}not-a-number${SEP}bad input`),
    );
    expect(e).toBeInstanceOf(NotAgs4Error);
    expect(e.exitCode).toBe(1);
    expect(Number.isNaN(e.exitCode)).toBe(false);
  });

  it("does not read exit code 0 as success", () => {
    // `Number.parseInt("0") || 1` is 1, not 0 — deliberate here, since a thrown
    // error carrying exit 0 would be a contradiction.
    expect(fromNativeError(new Error(`io${SEP}0${SEP}x`)).exitCode).toBe(1);
  });
});
