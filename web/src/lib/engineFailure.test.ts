import { describe, expect, it } from "vitest";
import { engineFailureMessage } from "./engineFailure";
import { EngineUnavailableError } from "./workerChannel";

// The one-voice mapping the three engine panes render (#391). The copy shapes
// are pinned loosely (noun + the load/crash discriminator + the "rest of the
// app" reassurance), not word-for-word — wording can be tuned without a test
// edit, but a pane must never lose its noun or show "check your connection"
// for a crash, which is a false lead about an engine that died holding a file.
// The one exception is the untyped fallback line, pinned exactly: its defects
// are word-level (a doubled "Error:" prefix, a bare trailing colon — #415).

describe("engineFailureMessage", () => {
  it("maps a load failure to download copy carrying the pane's noun", () => {
    const msg = engineFailureMessage(
      new EngineUnavailableError("fetch failed", "load"),
      "The fix engine",
    );
    expect(msg).toMatch(/^The fix engine couldn't be downloaded/);
    expect(msg).toMatch(/rest of the app is unaffected/);
    expect(msg).toMatch(/connection/);
  });

  it("maps a crash to stopped copy, without the connection false-lead", () => {
    const msg = engineFailureMessage(
      new EngineUnavailableError("worker died", "crash"),
      "The validator",
    );
    expect(msg).toMatch(/^The validator stopped/);
    expect(msg).toMatch(/rest of the app is unaffected/);
    expect(msg).not.toMatch(/connection/);
  });

  it("unwraps a plain Error in the fallback — no doubled prefix (#415)", () => {
    // Why plain Errors reach this branch: the comment on the fallback itself.
    expect(engineFailureMessage(new Error("boom"), "The validator")).toBe(
      "The validator failed: boom",
    );
  });

  it("keeps the name of a named error — it's half the information", () => {
    expect(
      engineFailureMessage(
        new TypeError("Failed to fetch"),
        "The explorer's engine",
      ),
    ).toBe("The explorer's engine failed: TypeError: Failed to fetch");
  });

  it("never ends the line at a bare colon for an empty-message Error", () => {
    expect(engineFailureMessage(new Error(""), "The validator")).toBe(
      "The validator failed: Error",
    );
  });

  it("falls back to the stringified value for a non-Error rejection", () => {
    // Rejections aren't always Error instances at a runtime boundary.
    expect(engineFailureMessage("boom", "The fix engine")).toBe(
      "The fix engine failed: boom",
    );
  });

  it("lets a pane override only the untyped fallback (Explore's offline case)", () => {
    const offline = "The data engine isn't cached for offline use yet.";
    expect(
      engineFailureMessage(new TypeError("Failed to fetch"), "X", offline),
    ).toBe(offline);
    // A typed failure already explains itself — the override must not mask it.
    expect(
      engineFailureMessage(
        new EngineUnavailableError("fetch failed", "load"),
        "The explorer's engine",
        offline,
      ),
    ).toMatch(/^The explorer's engine couldn't be downloaded/);
  });
});
