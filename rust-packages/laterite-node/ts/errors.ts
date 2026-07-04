// Exception hierarchy + the two native-failure mappers — the Node port of
// laterite-py's `_errors.py`. The engine never raises for un-validatable input;
// it reports a `kind` + `exit_code` that these map to the right error class
// (byte-faithful to the `lat-check` exit codes), so callers can branch on the
// type, not a brittle message match.
//
// Two protocols feed `makeError` (see the error-protocol note in `lib.rs`):
//   • `runCheck` returns a `{ok:false, errorKind, error, exitCode}` report
//     → `raiseFor` (the direct analog of Python's `raise_for`);
//   • `parseArrow` returns a *handle*, so it THROWS a `kind␟code␟message`
//     string → `fromNativeError` recovers and re-throws the mapped class.
import type { ValidationReport } from "./native";

export class Ags4Error extends Error {
  /** The validator exit code this failure carries (mirrors `lat-check`). */
  readonly exitCode: number;
  constructor(message: string, exitCode = 1) {
    super(message);
    this.name = "Ags4Error";
    this.exitCode = exitCode;
  }
}

/** Input the OS couldn't open (the Node analog of Python's `FileNotFoundError`). */
export class FileNotFoundError extends Ags4Error {
  constructor(message: string, exitCode = 3) {
    super(message, exitCode);
    this.name = "FileNotFoundError";
  }
}

/** Input has no GROUP rows — not a parseable AGS4 file. */
export class NotAgs4Error extends Ags4Error {
  constructor(message: string, exitCode = 4) {
    super(message, exitCode);
    this.name = "NotAgs4Error";
  }
}

/** A recognised but unsupported edition (AGS3) — refused, not silently
 * validated against an AGS4 schema (O-30). */
export class UnsupportedEditionError extends Ags4Error {
  constructor(message: string, exitCode = 4) {
    super(message, exitCode);
    this.name = "UnsupportedEditionError";
  }
}

/** Bad `dictVersion` / unimplemented external dictionary (O-28). */
export class BadDictError extends Ags4Error {
  constructor(message: string, exitCode = 5) {
    super(message, exitCode);
    this.name = "BadDictError";
  }
}

/** A passed `index=` certificate (`.ags.idx`) does not match the file it was read
 * for — its size / SHA-256 differ, so its byte offsets and clean verdict are now
 * lies. Raised at `read` time (fail-fast): an explicit `index=` asserts "this cert
 * is for this file", so a mismatch is an error, never a silent fall-back. Rebuild
 * it (`read(p).validate().certify()`). (#294 Batch E / #14) */
export class StaleCertError extends Ags4Error {
  constructor(message: string, exitCode = 4) {
    super(message, exitCode);
    this.name = "StaleCertError";
  }
}

/** Build the exception for a `(kind, exitCode, message)` failure — the data-
 * driven map (mirrors `_errors.py::_KIND_TO_EXC`), never message-matching. */
export function makeError(kind: string, exitCode: number, message: string): Ags4Error {
  switch (kind) {
    case "not_found":
    case "io":
      return new FileNotFoundError(message, exitCode);
    case "not_ags4":
    case "not_utf8":
      return new NotAgs4Error(message, exitCode);
    case "unsupported_edition":
      return new UnsupportedEditionError(message, exitCode);
    case "bad_dict":
    case "bad_args":
      return new BadDictError(message, exitCode);
    default:
      return new Ags4Error(message, exitCode);
  }
}

/** Pass a successful report through; throw the mapped exception for an
 * `{ok:false}` failure report (the analog of Python's `raise_for`). */
export function raiseFor(report: ValidationReport): ValidationReport {
  if (report.ok) return report;
  throw makeError(report.errorKind ?? "", report.exitCode, report.error ?? "unknown error");
}

/** Recover the mapped exception from a thrown native error whose message is the
 * `kind␟code␟message` the handle-returning `parseArrow` emits; fall back to a
 * generic `Ags4Error` for any other throw. */
export function fromNativeError(e: unknown): Ags4Error {
  const message = e instanceof Error ? e.message : String(e);
  // "\u001f" (unit separator) — the delimiter `lib.rs::thrown` joins on.
  const parts = message.split("\u001f");
  if (parts.length === 3) {
    const [kind, code, msg] = parts as [string, string, string];
    return makeError(kind, Number.parseInt(code, 10) || 1, msg);
  }
  return new Ags4Error(message);
}
