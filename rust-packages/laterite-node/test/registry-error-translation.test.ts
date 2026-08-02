// `ancestorChain` / `inheritedKeyNames` — the two registry facades that catch a
// native throw and re-type it to `Ags4Error`.
//
// Both do it with `e instanceof Error ? e.message : String(e)`, and only the
// first arm was ever taken. The second is not decoration: napi surfaces a
// rejection from the Rust side, and a non-`Error` value reaching JS turns
// `e.message` into `undefined` — so an unguarded version would report an
// `Ags4Error` whose message is the string "undefined", losing the only
// diagnostic the caller had.
//
// The module registry is reset per case so each gets its own mocked `./native`;
// the real behaviour is asserted first, unmocked, so the translation is pinned
// against something true before it is pinned against something staged.
import { afterEach, describe, expect, it, vi } from "vitest";

afterEach(() => {
  vi.resetModules();
  vi.doUnmock("../ts/native");
});

describe("the real registry facades", () => {
  it("walks a real ancestor chain and names an unknown code", async () => {
    const { ancestorChain, inheritedKeyNames } = await import("../ts/registry");
    const { Ags4Error } = await import("../ts/errors");

    // SAMP hangs off LOCA hangs off PROJ — the spine of every AGS file.
    expect(ancestorChain("SAMP")).toEqual(["SAMP", "LOCA", "PROJ"]);
    // A root group is its own single-element chain, which is how a caller
    // distinguishes "root" from "unknown".
    expect(ancestorChain("PROJ")).toEqual(["PROJ"]);
    expect(() => ancestorChain("ZZZZ")).toThrow(Ags4Error);
    expect(() => ancestorChain("ZZZZ")).toThrow(/ZZZZ/);

    // SAMP re-declares LOCA_ID as a KEY, so it inherits it from its parent.
    expect(inheritedKeyNames("SAMP").has("LOCA_ID")).toBe(true);
    // PROJ is a root: nothing above it to inherit from.
    expect(inheritedKeyNames("PROJ").size).toBe(0);
    expect(() => inheritedKeyNames("ZZZZ")).toThrow(Ags4Error);
  });
});

describe("when the native side throws something that is not an Error", () => {
  it("keeps the thrown value's text in ancestorChain's Ags4Error", async () => {
    vi.doMock("../ts/native", async () => {
      const real =
        await vi.importActual<typeof import("../ts/native")>("../ts/native");
      return {
        ...real,
        registryAncestorChain: () => {
          // Throwing a non-Error is the entire point of this case — it is the
          // shape the `String(e)` arm exists for, so the rule is off here only.
          // eslint-disable-next-line @typescript-eslint/only-throw-error
          throw "the group tree is on fire";
        },
      };
    });
    const { ancestorChain } = await import("../ts/registry");
    const { Ags4Error } = await import("../ts/errors");

    expect(() => ancestorChain("LOCA")).toThrow(Ags4Error);
    // The text survives. Without the String(e) arm this would read "undefined".
    expect(() => ancestorChain("LOCA")).toThrow("the group tree is on fire");
  });

  it("keeps the thrown value's text in inheritedKeyNames' Ags4Error", async () => {
    vi.doMock("../ts/native", async () => {
      const real =
        await vi.importActual<typeof import("../ts/native")>("../ts/native");
      return {
        ...real,
        registryInheritedKeyNames: () => {
          // As above: a non-Error throw is the condition under test.
          // eslint-disable-next-line @typescript-eslint/only-throw-error
          throw 42;
        },
      };
    });
    const { inheritedKeyNames } = await import("../ts/registry");
    const { Ags4Error } = await import("../ts/errors");

    expect(() => inheritedKeyNames("SAMP")).toThrow(Ags4Error);
    expect(() => inheritedKeyNames("SAMP")).toThrow("42");
  });

  it("still re-types a genuine Error rather than letting it through raw", async () => {
    // The other arm, asserted through the same mock harness so the two are
    // compared on equal terms: an `Error` keeps its message AND becomes an
    // Ags4Error, which is the facade's actual contract.
    vi.doMock("../ts/native", async () => {
      const real =
        await vi.importActual<typeof import("../ts/native")>("../ts/native");
      return {
        ...real,
        registryAncestorChain: () => {
          throw new RangeError("no such group in this edition");
        },
      };
    });
    const { ancestorChain } = await import("../ts/registry");
    const { Ags4Error } = await import("../ts/errors");

    expect(() => ancestorChain("LOCA")).toThrow(Ags4Error);
    expect(() => ancestorChain("LOCA")).toThrow(
      "no such group in this edition",
    );
    // Re-typed, not merely re-thrown — the facade promises one error type.
    expect(() => ancestorChain("LOCA")).not.toThrow(RangeError);
  });
});
