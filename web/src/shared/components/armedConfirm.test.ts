import { describe, expect, it } from "vitest";
import { onArmedTrigger } from "./armedConfirm";

// The armed-confirm contract (#408): a destructive action repaints to the
// error colour on first click and acts on the second, with the label changing
// to the question it is asking. Everything that DISARMS is here too — the
// component only forwards events.

describe("armed confirm", () => {
  it("arms on the first press without firing", () => {
    expect(onArmedTrigger("idle", "press")).toEqual({
      state: "armed",
      fire: false,
    });
  });

  it("fires on the second press and returns to idle", () => {
    expect(onArmedTrigger("armed", "press")).toEqual({
      state: "idle",
      fire: true,
    });
  });

  // Safari never focuses a button on click, so blur alone cannot be the only
  // pointer path out of the armed state — walking away must disarm too.
  it("disarms without firing when the pointer leaves", () => {
    expect(onArmedTrigger("armed", "pointer-leave")).toEqual({
      state: "idle",
      fire: false,
    });
  });

  it("disarms without firing on blur", () => {
    expect(onArmedTrigger("armed", "blur")).toEqual({
      state: "idle",
      fire: false,
    });
  });

  it("disarms without firing on Escape", () => {
    expect(onArmedTrigger("armed", "escape")).toEqual({
      state: "idle",
      fire: false,
    });
  });

  // The action's own guard flipping (busy, empty selection) while armed:
  // a control that re-enables must ask again, never fire from stale intent.
  it("resets when the control is disabled while armed", () => {
    expect(onArmedTrigger("armed", "disable")).toEqual({
      state: "idle",
      fire: false,
    });
  });

  it("stays idle on every non-press event", () => {
    for (const ev of ["pointer-leave", "blur", "escape", "disable"] as const) {
      expect(onArmedTrigger("idle", ev)).toEqual({
        state: "idle",
        fire: false,
      });
    }
  });
});
