// What an `Ags4File` remembers about where it came from, and the guards around it.
//
// `read()` accepts a path, text or raw bytes, and several operations later need the
// ORIGINAL bytes again — `certify()` mints over them, `validate()`/`fix()` re-drive
// the engine from them. `#sourceBytes()` is the one place that decides which of the
// three to hand back, with a fourth case (a synthesised handle that never had a
// source) falling back to the re-emit.
//
// That choice is invisible until it is wrong. A handle that fell back to the re-emit
// when it should have re-read the path would mint a certificate over bytes that are
// not the file on disk — and the certificate would verify, because it is
// self-consistent. So each shape is pinned to the bytes it must produce.
import { mkdtempSync, readFileSync, statSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { fileURLToPath } from "node:url";
import { describe, expect, it } from "vitest";

import { Ags4Error, read } from "../ts/index";

/** `certify()` returns the PATH it wrote; the recorded source size lives in the
 *  certificate's `file.size`. Reading it back is the only way to see which bytes
 *  the mint actually measured. */
const certifiedSize = (idxPath: string): number =>
  JSON.parse(readFileSync(idxPath, "utf8")).file.size;

const CLEAN_PATH = fileURLToPath(
  new URL(
    "../../laterite-ags4-validator/tests/fixtures/clean_minimal.ags",
    import.meta.url,
  ),
);
const CLEAN = readFileSync(CLEAN_PATH);
const tmp = () => mkdtempSync(join(tmpdir(), "laterite-src-"));

describe("the source a handle re-reads", () => {
  it("re-reads the PATH from disk, not a snapshot taken at read() time", () => {
    // The distinction that matters: `#sourceBytes()` calls readFileSync(s.path)
    // rather than returning bytes captured earlier. A handle opened from a path and
    // certified after the file changed must mint over the CURRENT file — otherwise
    // it produces a certificate for content that is not there any more, and the
    // staleness check has nothing to catch it with.
    const dir = tmp();
    const p = join(dir, "moving.ags");
    writeFileSync(p, CLEAN);
    const f = read(p);

    // A mutation that keeps the file VALID — certify() re-validates and refuses an
    // error-dirty file, so a broken mutation would fail for the wrong reason and
    // prove nothing about which bytes were read.
    const mutated = Buffer.from(
      CLEAN.toString("utf8").replace(
        "Clean minimal AGS4 fixture",
        "Clean minimal AGS4 fixture, edited on disk after the handle was opened",
      ),
      "utf8",
    );
    expect(mutated.length).toBeGreaterThan(CLEAN.length);
    writeFileSync(p, mutated);

    const size = certifiedSize(f.certify());
    expect(size).toBe(mutated.length);
    expect(size).not.toBe(CLEAN.length);
  });

  it("hands back raw bytes exactly as given", () => {
    // The certificate's recorded size is the SOURCE length, proving the raw
    // Uint8Array was used verbatim rather than round-tripped through the emitter —
    // which can differ from the source by a trailing blank line.
    const f = read(CLEAN);
    expect(certifiedSize(f.certify(join(tmp(), "raw.ags.idx")))).toBe(
      CLEAN.length,
    );
  });

  it("UTF-8-encodes a text source rather than counting characters", () => {
    // The trap is a multi-byte cell: a JS string's `.length` is UTF-16 code units,
    // but the certificate records BYTES. Measure before encoding and every
    // non-ASCII file gets a size that can never match the disk.
    const withUnicode = CLEAN.toString("utf8").replace(
      /"DATA","([^"]*)"/,
      '"DATA","café"',
    );
    expect(withUnicode).toContain("café");
    const byteLen = Buffer.byteLength(withUnicode, "utf8");
    expect(byteLen).toBeGreaterThan(withUnicode.length); // the é is 2 bytes

    const f = read(Buffer.from(withUnicode, "utf8"));
    expect(certifiedSize(f.certify(join(tmp(), "utf8.ags.idx")))).toBe(byteLen);
  });

  it("refuses to guess a certificate path for a handle with no source path", () => {
    // Deriving `<source>.idx` is only possible when there IS a source path. For a
    // text/bytes handle the caller must say where it goes — inventing a path in the
    // cwd would scatter certificates next to whatever process happened to run.
    expect(() => read(CLEAN).certify()).toThrow(Ags4Error);
    expect(() => read(CLEAN).certify()).toThrow(/no source path/);
  });
});

describe("save()", () => {
  it("writes the handle's bytes and returns the path for chaining", () => {
    const dir = tmp();
    const out = join(dir, "saved.ags");
    const f = read(CLEAN);
    const returned = f.save(out);
    expect(returned).toBe(out);
    expect(readFileSync(out)).toEqual(Buffer.from(f.bytes));
  });

  it("writes the RE-EMIT, which is what makes a round-trip stable", () => {
    // save() writes `this.bytes` (the canonical re-emit), not the source. Reading
    // what it wrote and saving again must produce identical bytes — if it wrote the
    // source instead, the first save would differ from every later one.
    const dir = tmp();
    const once = join(dir, "a.ags");
    const twice = join(dir, "b.ags");
    read(CLEAN).save(once);
    read(readFileSync(once)).save(twice);
    expect(readFileSync(twice)).toEqual(readFileSync(once));
  });
});

describe("asking about a group that is not in the file", () => {
  // Every accessor routes through the same private `#meta(code)`, which throws
  // rather than returning null/undefined. A silent undefined here would surface far
  // away as "cannot read property of undefined" with no mention of the group.
  it.each(["sqlTypes", "headings", "units", "types"] as const)(
    "%s names the missing group",
    (accessor) => {
      const f = read(CLEAN) as unknown as Record<
        string,
        (c: string) => unknown
      >;
      // Asserted, not skipped: a `typeof fn === "function"` guard with an early
      // return would silently drop any accessor that got renamed, leaving this
      // parameterised test green while covering nothing.
      expect(typeof f[accessor]).toBe("function");
      expect(() => f[accessor]?.call(f, "ZZZZ")).toThrow(/ZZZZ/);
    },
  );

  it("says the group is not in the file, not merely that something failed", () => {
    const f = read(CLEAN);
    expect(() => f.sqlTypes("ZZZZ")).toThrow(/not in file/);
  });
});

describe("certify refuses to clobber a file it did not write", () => {
  it("throws rather than overwriting a non-certificate at the target path", () => {
    // `certify(out)` replaces an existing `.ags.idx` freely — re-certifying is
    // normal. But the path is caller-supplied, and a typo pointing at a source file
    // would destroy it silently. The guard sniffs the first bytes for a JSON object.
    const dir = tmp();
    const victim = join(dir, "important.ags");
    writeFileSync(victim, CLEAN);
    const before = readFileSync(victim);

    const f = read(CLEAN);
    expect(() => f.certify(victim)).toThrow(Ags4Error);
    expect(() => f.certify(victim)).toThrow(/refusing to overwrite/);
    // The point of the guard: the file is still there, byte-for-byte.
    expect(readFileSync(victim)).toEqual(before);
  });

  it("replaces an existing certificate without complaint", () => {
    const dir = tmp();
    const idx = join(dir, "c.ags.idx");
    const f = read(CLEAN);
    f.certify(idx);
    const first = statSync(idx).mtimeMs;
    expect(() => f.certify(idx)).not.toThrow();
    expect(statSync(idx).mtimeMs).toBeGreaterThanOrEqual(first);
    expect(readFileSync(idx, "utf8").trimStart().startsWith("{")).toBe(true);
  });

  it("writes into an empty file at the target path", () => {
    // A zero-length file fails the `size > 0` half of the guard, so it is not a
    // "not a certificate" case — an empty placeholder is fine to write over.
    const dir = tmp();
    const empty = join(dir, "empty.ags.idx");
    writeFileSync(empty, "");
    expect(() => read(CLEAN).certify(empty)).not.toThrow();
    expect(statSync(empty).size).toBeGreaterThan(0);
  });
});

describe("toString", () => {
  it("summarises the handle for a log line", () => {
    const f = read(CLEAN);
    const s = f.toString();
    expect(s).toMatch(/^<Ags4File groups=\d+ tranAgs=/);
    expect(s).toContain(`groups=${f.groups.length}`);
  });
});
