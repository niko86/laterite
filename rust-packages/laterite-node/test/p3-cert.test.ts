// #294 Batch E / #14 — the Node certificate lifecycle: `Ags4File.certify()`
// mints an `.ags.idx`, `read(f, { index })` consumes + freshness-checks it, and a
// fresh + engine-matching cert lets an errors-only `.validate()` skip the rule
// engine. The cert wraps the ONE core `Sidecar`, so a Node-minted `.ags.idx` is
// byte-compatible with Python / `lat-check --emit-index`.
import { mkdtempSync, readFileSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { fileURLToPath } from "node:url";
import { describe, expect, it } from "vitest";
import { Ags4Error, StaleCertError, read } from "../ts/index";

// The established clean fixture (0 errors, 0 warnings) — the same file the CLI's
// --emit-index test mints from, so the cert values line up cross-surface.
const CLEAN = readFileSync(
  fileURLToPath(
    new URL("../../laterite-ags4-validator/tests/fixtures/clean_minimal.ags", import.meta.url),
  ),
);

function tmpFile(name: string, bytes: Uint8Array): string {
  const dir = mkdtempSync(join(tmpdir(), "laterite-cert-"));
  const p = join(dir, name);
  writeFileSync(p, bytes);
  return p;
}

describe("certify → .ags.idx (#294 Batch E / #14)", () => {
  it("mints a certificate beside the file after a clean validate", () => {
    const f = tmpFile("clean.ags", CLEAN);
    const idxPath = read(f).validate({ warnings: false }).certify();
    expect(idxPath).toBe(`${f}.idx`);
    // The written file is a laterite certificate (JSON) with the expected stamp.
    const cert = JSON.parse(readFileSync(idxPath, "utf8"));
    expect(cert.validation.validator).toBe("laterite_ags4");
    expect(cert.file.edition).toBe("4.2");
    expect(cert.file.sha256).toHaveLength(64);
    expect(cert.file.size).toBe(CLEAN.length);
  });

  it("read(f, { index }) carries a fresh cert; errors-only validate is engine-skipped", () => {
    const f = tmpFile("clean.ags", CLEAN);
    const idx = read(f).validate({ warnings: false }).certify();

    const rep = read(f, { index: idx }).validate({ warnings: false }).report;
    // The sentinel resolution the engine never emits — proof the cert short-circuit fired.
    expect(rep?.resolution).toBe("certified");
    expect(rep?.count).toBe(0);
    expect(rep?.isValid).toBe(true);
    expect(rep?.dictVersion).toBe("4.2");
  });

  it("a stale cert (file changed) fails fast at read time", () => {
    const f = tmpFile("clean.ags", CLEAN);
    const idx = read(f).validate({ warnings: false }).certify();
    // Mutate the source after minting → size/SHA no longer match.
    writeFileSync(f, Buffer.concat([CLEAN, Buffer.from('"DATA","EXTRA"\r\n')]));
    expect(() => read(f, { index: idx })).toThrow(StaleCertError);
  });
});

describe("certify guards", () => {
  it("refuses without a prior validate, and with findings", () => {
    const f = tmpFile("clean.ags", CLEAN);
    expect(() => read(f).certify()).toThrow(/call \.validate\(\)/);
    // A LOCA-only file has findings → not certifiable.
    const dirty = tmpFile("dirty.ags", Buffer.from('"GROUP","LOCA"\r\n'));
    expect(() => read(dirty).validate().certify()).toThrow(/cannot certify/);
  });

  it("refuses to overwrite the source file (data-loss guard)", () => {
    const f = tmpFile("clean.ags", CLEAN);
    // Passing the .ags path as the OUTPUT would clobber the source.
    expect(() => read(f).validate({ warnings: false }).certify(f)).toThrow(/refusing to overwrite/);
  });

  it("throws Ags4Error for a text handle with no source path", () => {
    expect(() => read(undefined, { text: CLEAN.toString("utf8") }).validate({ warnings: false }).certify()).toThrow(
      Ags4Error,
    );
  });
});
