// P3 — laterite.transport: zstd pack/unpack + age lock/unlock round-trips.
import { mkdtempSync, readFileSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { afterAll, describe, expect, it } from "vitest";
import { transport } from "../ts/index";

const dir = mkdtempSync(join(tmpdir(), "laterite-transport-"));
const p = (name: string) => join(dir, name);
const PAYLOAD = '"GROUP","PROJ"\r\n'.repeat(500); // compressible AGS-ish text

afterAll(() => {
  // best-effort temp cleanup
  try {
    require("node:fs").rmSync(dir, { recursive: true, force: true });
  } catch {
    /* ignore */
  }
});

describe("pack / unpack (zstd)", () => {
  it("round-trips a file and reports stats", () => {
    const src = p("in.ags");
    writeFileSync(src, PAYLOAD);
    const packed = transport.pack(src, p("out.zst"));
    expect(packed.bytes).toBeGreaterThan(0n);
    expect(packed.ratio).toBeLessThan(1); // compressible payload shrank
    expect(typeof packed.elapsedS).toBe("number");

    const stats = transport.unpack(p("out.zst"), p("back.ags"));
    expect(readFileSync(p("back.ags"), "utf8")).toBe(PAYLOAD);
    expect(stats.bytes).toBe(BigInt(Buffer.byteLength(PAYLOAD)));
  });
});

// The passphrase path pins scrypt log_N 18 (laterite-transport SCRYPT_LOG_N,
// for age-ecosystem interop). That's a deliberately expensive KDF — ~256 MiB /
// sub-second on a normal box, but many seconds on a memory-starved CI container
// (the 256 MiB scrypt buffer thrashes), so these round-trips need far more than
// vitest's 5 s default. lock + unlock is two scrypt derivations each.
const SCRYPT_TIMEOUT_MS = 90_000;

describe("lock / unlock (zstd + age passphrase)", () => {
  it(
    "round-trips with the right passphrase",
    () => {
      const src = p("secret.ags");
      writeFileSync(src, PAYLOAD);
      transport.lock(src, p("secret.zst.age"), "correct horse");
      transport.unlock(p("secret.zst.age"), p("secret.back.ags"), "correct horse");
      expect(readFileSync(p("secret.back.ags"), "utf8")).toBe(PAYLOAD);
    },
    SCRYPT_TIMEOUT_MS,
  );

  it(
    "fails to unlock with the wrong passphrase",
    () => {
      const src = p("secret2.ags");
      writeFileSync(src, PAYLOAD);
      transport.lock(src, p("secret2.zst.age"), "right");
      expect(() => transport.unlock(p("secret2.zst.age"), p("nope.ags"), "wrong")).toThrow();
    },
    SCRYPT_TIMEOUT_MS,
  );
});

// The in-memory twins (#389) — the Node mirror of laterite-py's `pack_bytes`/….
// The point isn't just a self-round-trip: the `*Bytes` envelope must be the SAME
// as the file form (both call the one shared leaf), so a blob sealed in memory
// opens via the file API and vice versa — that cross-form interop is what lets a
// web backend seal `read().fix().bytes` without ever touching disk.
const BYTES = Buffer.from(PAYLOAD, "utf8");

describe("packBytes / unpackBytes (zstd, in-memory)", () => {
  it("round-trips bytes → bytes with no filesystem", () => {
    const packed = transport.packBytes(BYTES);
    expect(packed.length).toBeGreaterThan(0);
    expect(packed.length).toBeLessThan(BYTES.length); // compressible payload shrank
    expect(Buffer.from(transport.unpackBytes(packed)).equals(BYTES)).toBe(true);
  });

  it("produces a frame the file `unpack` can open (cross-form interop)", () => {
    const packed = transport.packBytes(BYTES);
    writeFileSync(p("frombytes.zst"), packed);
    transport.unpack(p("frombytes.zst"), p("frombytes.ags"));
    expect(readFileSync(p("frombytes.ags"), "utf8")).toBe(PAYLOAD);
  });
});

describe("lockBytes / unlockBytes (zstd + age passphrase, in-memory)", () => {
  it(
    "round-trips bytes → bytes without writing plaintext",
    () => {
      const sealed = transport.lockBytes(BYTES, "correct horse");
      expect(sealed.length).toBeGreaterThan(0);
      const opened = Buffer.from(transport.unlockBytes(sealed, "correct horse"));
      expect(opened.equals(BYTES)).toBe(true);
    },
    SCRYPT_TIMEOUT_MS,
  );

  it(
    "seals a blob the file `unlock` can open (cross-form interop)",
    () => {
      const sealed = transport.lockBytes(BYTES, "correct horse");
      writeFileSync(p("frombytes.zst.age"), sealed);
      transport.unlock(p("frombytes.zst.age"), p("frombytes.unlocked.ags"), "correct horse");
      expect(readFileSync(p("frombytes.unlocked.ags"), "utf8")).toBe(PAYLOAD);
    },
    SCRYPT_TIMEOUT_MS,
  );

  it(
    "rejects the wrong passphrase",
    () => {
      const sealed = transport.lockBytes(BYTES, "right");
      expect(() => transport.unlockBytes(sealed, "wrong")).toThrow();
    },
    SCRYPT_TIMEOUT_MS,
  );
});
