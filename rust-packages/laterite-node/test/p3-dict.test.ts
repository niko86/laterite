// The `--dict` custom-dictionary overlay (#568) on the Node surface — the library
// (`validate({ dictionary })`, `Report.revalidateReason`, `certify({ dictionary })`) and
// the `lat` CLI. These assert the OUTPUT — the count of `XTRA` findings, `certified`, and
// the `revalidateReason` token — never that the flag merely parses: a flag that is
// accepted and dropped looks identical to one that works until the findings differ.
//
// Referenced, not copied: the dictionary + delivery fixtures are the same ones the Rust
// E2E test (`custom_dict.rs`) and the Python test (`test_custom_dict.py`) exercise.
import { spawnSync } from "node:child_process";
import { copyFileSync, mkdtempSync, readFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { describe, expect, it } from "vitest";
import { BadDictError, read, validate } from "../ts/index";

const pkgDir = resolve(fileURLToPath(new URL("..", import.meta.url)));
const BIN = join(pkgDir, "bin.mjs");
const fixtures = resolve(pkgDir, "../laterite-ags4-validator/tests/fixtures");
const custom = join(fixtures, "custom_dict");
const DELIVERY = join(custom, "delivery_with_xtra.ags");
const DICT_JSON = join(custom, "xtra.dict.json");
const DICT_AGS = join(custom, "xtra.dict.ags");
// TRAN_AGS 4.2, validates error-clean — the file the cert round-trip mints over.
const CLEAN = join(fixtures, "clean_minimal.ags");

const xtra = (report: ReturnType<typeof validate>): number =>
  report.findings.filter((f) => f.group === "XTRA").length;

const cli = (...args: string[]) =>
  spawnSync("node", [BIN, ...args], { encoding: "utf8" });

describe("lat node — custom-dictionary overlay (#568)", () => {
  it("the bundled dictionary flags the unknown XTRA group", () => {
    expect(xtra(validate(DELIVERY))).toBeGreaterThan(0);
  });

  it("a JSON dictionary overlay makes XTRA a first-class group", () => {
    const r = validate(DELIVERY, { dictionary: DICT_JSON });
    expect(xtra(r)).toBe(0);
    // A purely-additive dict overlays the latest edition, not a replacement.
    expect(r.dictVersion).toBe("4.2");
  });

  it("the .ags, JSON, and raw-bytes spellings of the dictionary agree", () => {
    const nJson = validate(DELIVERY, { dictionary: DICT_JSON }).count;
    const nAgs = validate(DELIVERY, { dictionary: DICT_AGS }).count;
    const nBytes = validate(DELIVERY, {
      dictionary: readFileSync(DICT_JSON),
    }).count;
    expect(nAgs).toBe(nJson);
    expect(nBytes).toBe(nJson);
    // And the overlay strictly reduces findings vs the bundled dictionary.
    expect(nJson).toBeLessThan(validate(DELIVERY).count);
  });

  it("dictReplace cannot be combined with dictVersion", () => {
    expect(() =>
      validate(DELIVERY, {
        dictionary: DICT_JSON,
        dictReplace: true,
        dictVersion: "4.1",
      }),
    ).toThrow(BadDictError);
  });

  it("a bad dictionary throws BadDictError", () => {
    expect(() =>
      validate(DELIVERY, { dictionary: join(custom, "nope.json") }),
    ).toThrow(BadDictError);
  });

  it("revalidateReason is undefined without a certificate", () => {
    expect(
      validate(DELIVERY, { dictionary: DICT_JSON }).revalidateReason,
    ).toBeUndefined();
  });
});

describe("lat node — cert records which dictionary judged (O-48)", () => {
  const mkSrc = (): string => {
    const dir = mkdtempSync(join(tmpdir(), "laterite-dict-"));
    const src = join(dir, "clean.ags");
    copyFileSync(CLEAN, src);
    return src;
  };

  it("a matching config uses the certificate", () => {
    const src = mkSrc();
    const idx = read(src).certify();
    const r = read(src, { index: idx }).validate().report!;
    expect(r.certified).toBe(true);
    expect(r.revalidateReason).toBeUndefined();
  });

  it("adding a dictionary to a bare cert revalidates (dictionary_changed)", () => {
    const src = mkSrc();
    const idx = read(src).certify(); // minted WITHOUT a custom dict
    const r = read(src, { index: idx }).validate({
      dictionary: DICT_JSON,
    }).report!;
    expect(r.certified).toBe(false);
    expect(r.revalidateReason).toBe("dictionary_changed");
  });

  it("certify stamps the dict; a matching read is certified, a bare read revalidates", () => {
    const src = mkSrc();
    const idx = read(src).certify(undefined, { dictionary: DICT_JSON });
    const same = read(src, { index: idx }).validate({
      dictionary: DICT_JSON,
    }).report!;
    expect(same.certified).toBe(true);
    const bare = read(src, { index: idx }).validate().report!;
    expect(bare.certified).toBe(false);
    expect(bare.revalidateReason).toBe("dictionary_changed");
  });
});

describe("lat node CLI — --dict / --dict-replace", () => {
  it("--dict overlay removes the XTRA findings", () => {
    const r = cli("validate", DELIVERY, "--dict", DICT_JSON, "--json");
    const findings = JSON.parse(r.stdout).findings as Record<
      string,
      { group: string }[]
    >;
    const n = Object.values(findings)
      .flat()
      .filter((f) => f.group === "XTRA").length;
    expect(n).toBe(0);
    expect([0, 1]).toContain(r.status); // residual findings, not a parse failure
  });

  it("--dict-replace contradicts --dict-version (exit 5)", () => {
    const r = cli(
      "validate",
      DELIVERY,
      "--dict",
      DICT_JSON,
      "--dict-replace",
      "--dict-version",
      "4.1",
    );
    expect(r.status).toBe(5);
    expect(r.stderr.toLowerCase()).toContain("replace");
  });

  it("a bad --dict is exit 5", () => {
    expect(
      cli("validate", DELIVERY, "--dict", join(custom, "nope.json")).status,
    ).toBe(5);
  });

  it("fix accepts --dict and writes a file", () => {
    const dir = mkdtempSync(join(tmpdir(), "laterite-dict-fix-"));
    const out = join(dir, "fixed.ags");
    const r = cli("fix", DELIVERY, "--dict", DICT_JSON, "--fix-out", out);
    expect([0, 1]).toContain(r.status);
    expect(readFileSync(out).length).toBeGreaterThan(0);
  });
});
