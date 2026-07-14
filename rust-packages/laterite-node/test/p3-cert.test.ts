// The Node certificate lifecycle: `Ags4File.certify()` VALIDATES and mints an
// `.ags.idx`, `read(f, { index })` consumes + freshness-checks it, and a certificate the
// engine can fully answer from lets `.validate()` skip the rule pass. The cert wraps the
// ONE core `Sidecar` and the ONE trust model (`laterite-ags4-trust`), so a Node-minted
// `.ags.idx` is byte-compatible with Python / the `lat` binary / the browser — and, more
// to the point, is TRUSTED by exactly the same rule on every one of them.
//
// `certify()` no longer requires (or accepts) a prior verdict: it used to record whatever
// the caller''s last `.validate()` had found, with `warnings`/`fyi` as OPTIONAL mint
// arguments that nothing ever passed — so every certificate this package produced claimed
// to have measured zero warnings without having looked, and a later `validate({warnings:
// true})` read that zero and skipped the engine.
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
  it("mints a certificate beside the file — and runs the validation itself", () => {
    const f = tmpFile("clean.ags", CLEAN);
    const idxPath = read(f).certify(); // no prior .validate() — the mint does it
    expect(idxPath).toBe(`${f}.idx`);
    // The written file is a laterite certificate (JSON) with the expected stamp.
    const cert = JSON.parse(readFileSync(idxPath, "utf8"));
    expect(cert.validation.validator).toBe("laterite_ags4");
    expect(cert.validation.edition.resolved).toBe("4.2");
    expect(cert.file.sha256).toHaveLength(64);
    expect(cert.file.size).toBe(CLEAN.length);
  });

  it("MEASURES every tier it names — it is never told a verdict", () => {
    // The bug this closes: the stamp's warning/FYI counts were the caller''s assertion,
    // defaulted to zero, and a `warnings: true` request believed them. Now each tier says
    // whether it was MEASURED, and the mint measures all three because it ran all three.
    const f = tmpFile("clean.ags", CLEAN);
    const cert = JSON.parse(readFileSync(read(f).certify(), "utf8"));
    for (const tier of ["errors", "warnings", "fyi"]) {
      expect(cert.validation[tier]).toEqual({ state: "measured", count: 0 });
    }
  });

  it("cannot claim an on-disk FILE/ check — there is no field to say it in", () => {
    // A cert minted with `--check-files`, then the FILE/ tree deleted, was still trusted
    // and still "clean": the certified BYTES had not moved. Rule 20''s on-disk half is not
    // a function of those bytes, so no statement about them may stand in for it.
    const f = tmpFile("clean.ags", CLEAN);
    const raw = readFileSync(read(f).certify(), "utf8");
    expect(raw).not.toContain("check_files");
  });

  it("read(f, { index }) carries a fresh cert; validate is engine-skipped", () => {
    const f = tmpFile("clean.ags", CLEAN);
    const idx = read(f).certify();

    const rep = read(f, { index: idx }).validate({ warnings: false }).report;
    // `certified` is the proof the short-circuit fired. It used to be a VALUE of
    // `resolution` ("certified" in place of "exact"), which meant asking which dictionary
    // judged the file got you an answer to a different question.
    expect(rep?.certified).toBe(true);
    expect(rep?.count).toBe(0);
    expect(rep?.isValid).toBe(true);
    expect(rep?.dictVersion).toBe("4.2");
    expect(rep?.resolution).toBe("exact");
  });

  it("a cert that measured zero warnings covers a warnings request too", () => {
    const f = tmpFile("clean.ags", CLEAN);
    const idx = read(f).certify();
    // The fixture is warning-clean, and the mint MEASURED that — so the engine can be
    // skipped for a question it can fully answer, not merely for the one it was minted on.
    expect(read(f, { index: idx }).validate({ warnings: true }).report?.certified).toBe(true);
  });

  it("a stale cert (file changed) fails fast at read time", () => {
    const f = tmpFile("clean.ags", CLEAN);
    const idx = read(f).validate({ warnings: false }).certify();
    // Mutate the source after minting → size/SHA no longer match.
    writeFileSync(f, Buffer.concat([CLEAN, Buffer.from('"DATA","EXTRA"\r\n')]));
    expect(() => read(f, { index: idx })).toThrow(StaleCertError);
  });
});

describe("certifyBytes → in-memory cert (#390)", () => {
  it("returns the certificate bytes without writing a file", () => {
    const f = tmpFile("clean.ags", CLEAN);
    const blob = read(f).certifyBytes();
    expect(Buffer.isBuffer(blob)).toBe(true);
    const cert = JSON.parse(blob.toString("utf8"));
    expect(cert.validation.validator).toBe("laterite_ags4");
    expect(cert.file.size).toBe(CLEAN.length);
  });

  it("produces a usable cert — write it out and read({ index }) skips the engine", () => {
    const f = tmpFile("clean.ags", CLEAN);
    const blob = read(f).certifyBytes();
    const idx = `${f}.idx`;
    writeFileSync(idx, blob);
    const rep = read(f, { index: idx }).validate({ warnings: false }).report;
    expect(rep?.certified).toBe(true);
    expect(rep?.isValid).toBe(true);
  });

  it("matches the file certify() bar the mint timestamp (same file + groups)", () => {
    const f = tmpFile("clean.ags", CLEAN);
    const onDisk = JSON.parse(readFileSync(read(f).certify(), "utf8"));
    const inMem = JSON.parse(read(f).certifyBytes().toString("utf8"));
    expect(inMem.file).toEqual(onDisk.file);
    expect(inMem.groups).toEqual(onDisk.groups);
  });

  it("refuses a file with ERRORS — and needs no prior validate to know it", () => {
    const dirty = tmpFile("dirty.ags", Buffer.from('"GROUP","LOCA"\r\n'));
    expect(() => read(dirty).certifyBytes()).toThrow(/cannot certify/);
  });
});

describe("certify guards", () => {
  it("refuses a file with errors — the one rule there has ever been", () => {
    // A LOCA-only file has ERRORS → not certifiable. (Warnings and FYI findings are
    // recorded, not refused: a cert that measured a warning simply cannot answer a
    // warnings request, which is a different thing from being un-mintable.)
    const dirty = tmpFile("dirty.ags", Buffer.from('"GROUP","LOCA"\r\n'));
    expect(() => read(dirty).certify()).toThrow(/cannot certify/);
  });

  it("refuses to overwrite the source file (data-loss guard)", () => {
    const f = tmpFile("clean.ags", CLEAN);
    // Passing the .ags path as the OUTPUT would clobber the source.
    expect(() => read(f).certify(f)).toThrow(/refusing to overwrite/);
  });

  it("throws Ags4Error for a text handle with no source path", () => {
    expect(() => read(undefined, { text: CLEAN.toString("utf8") }).certify()).toThrow(Ags4Error);
  });
});

describe("the decoder is part of the verdict", () => {
  // The same clean file, but PROJ_NAME carries a Greek capital omega — UTF-8 bytes CE A9.
  // Read as UTF-8 that is ONE code point (937): above the extended-ASCII range Rule 1
  // tolerates, so a Rule 1 ERROR. Read as windows-1252 the very same two bytes are TWO
  // code points (206, 169), both inside it — only an FYI. One file, two decoders, two
  // verdicts, differing in the tier a certificate exists to assert.
  const OMEGA = Buffer.from(
    CLEAN.toString("utf8").replace(
      '"DATA","P1","Clean minimal AGS4 fixture (hand-authored, MIT, ours)"',
      '"DATA","P1","\u03a9 site"',
    ),
    "utf8",
  );

  it("the two decoders really do disagree (the premise, asserted)", () => {
    const f = tmpFile("omega.ags", OMEGA);
    expect(read(f).validate({ warnings: false }).report?.count).toBe(1);
    expect(read(f).validate({ warnings: false, encoding: "windows-1252" }).report?.count).toBe(0);
  });

  it("a certificate minted through another decoder does not answer", () => {
    const f = tmpFile("omega.ags", OMEGA);
    // Error-clean under windows-1252, so it mints — and the stamp records which decoder.
    const idx = read(f, { encoding: "windows-1252" }).certify();
    const cert = JSON.parse(readFileSync(idx, "utf8"));
    expect(cert.validation.encoding).toBe("windows-1252");

    // The same bytes read with the default decoder, offering that certificate.
    const rep = read(f, { index: idx }).validate({ warnings: false }).report;
    expect(rep?.certified).toBe(false);
    expect(rep?.count).toBe(1); // the engine ran, so the Rule 1 error is reported
    expect(rep?.isValid).toBe(false);

    // The decoder it WAS minted under still gets the fast path — a match, not a ban.
    const same = read(f, { index: idx, encoding: "windows-1252" }).validate({ warnings: false });
    expect(same.report?.certified).toBe(true);
  });
});
