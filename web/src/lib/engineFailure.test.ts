import { describe, expect, it } from "vitest";
import {
  engineFailureMessage,
  tier1EngineFailureMessage,
} from "./engineFailure";
import { EngineUnavailableError } from "./workerChannel";

// The one-voice mapping the engine-failure surfaces render (#391; the Excel
// converter joined in #414). The copy shapes
// are pinned loosely (noun + the load/crash discriminator + the "rest of the
// app" reassurance), not word-for-word — wording can be tuned without a test
// edit, but a pane must never lose its noun or show "check your connection"
// for a crash, which is a false lead about an engine that died holding a file.
// Nor for a tier-1 load (#413), where the advice ends at a Try again button
// that Validate and Fix do not render. The one exception is the untyped
// fallback line, pinned exactly: its defects are word-level (a doubled
// "Error:" prefix, a bare trailing colon — #415).

describe("engineFailureMessage", () => {
  it("maps a load failure to download copy carrying the pane's noun", () => {
    // A tier-2 noun: the default recovery is the one those panes can honour,
    // and since #413 the tier-1 panes no longer take this path.
    const msg = engineFailureMessage(
      new EngineUnavailableError("fetch failed", "load"),
      "The explorer's engine",
    );
    expect(msg).toMatch(/^The explorer's engine couldn't be downloaded/);
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

  it("gives a tier-1 load the recovery it has, not a control it hasn't (#413)", () => {
    const e = new EngineUnavailableError("fetch failed", "load");
    const msg = tier1EngineFailureMessage(e, "The validator");
    expect(msg).toMatch(/^The validator couldn't be downloaded/);
    // The two false leads on a pane with no Try again button: the control
    // itself, and connection advice that only reads as advice beside one.
    expect(msg).not.toMatch(/try again/i);
    expect(msg).not.toMatch(/connection/);
    // Saying nothing would be the worse fix — name what does recover.
    expect(msg).toMatch(/load your file again/i);
    // Everything above also passes if the door is a bare alias. This is what
    // fails then, and it is the whole point of there being two.
    expect(msg).not.toBe(engineFailureMessage(e, "The validator"));
  });

  it("changes nothing but the recovery — a tier-1 crash reads identically", () => {
    // The crash line never named a control, so it had nothing to overpromise.
    const e = new EngineUnavailableError("worker died", "crash");
    expect(tier1EngineFailureMessage(e, "The fix engine")).toBe(
      engineFailureMessage(e, "The fix engine"),
    );
  });
});
