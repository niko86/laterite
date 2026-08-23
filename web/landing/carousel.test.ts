/* The finding carousel's paging arithmetic (#592). Pure and tested here per
 * dec-web-test-altitude: the infinite wrap is an acceptance criterion, and a
 * modulo with a sign mistake wraps one way and walls the other — exactly the
 * kind of guard worth pinning below e2e.
 */

import { describe, expect, it } from "vitest";
import { clampIndex, stepIndex, swipeStep } from "./carousel";

describe("stepIndex", () => {
  it("advances and retreats within range", () => {
    expect(stepIndex(1, 1, 4)).toBe(2);
    expect(stepIndex(2, -1, 4)).toBe(1);
  });

  it("wraps in BOTH directions — the infinite loop the issue asks for", () => {
    expect(stepIndex(3, 1, 4)).toBe(0);
    expect(stepIndex(0, -1, 4)).toBe(3);
  });

  it("stays put on a one-card list, whatever the delta", () => {
    expect(stepIndex(0, 1, 1)).toBe(0);
    expect(stepIndex(0, -1, 1)).toBe(0);
  });

  it("answers 0 for an empty list — no modulo by zero", () => {
    expect(stepIndex(0, 1, 0)).toBe(0);
  });
});

describe("clampIndex", () => {
  it("keeps an in-range position", () => {
    expect(clampIndex(2, 4)).toBe(2);
  });

  it("pulls a stranded position back when the list shrinks", () => {
    // A fix can remove the card the reader is on: revalidation shrinks the
    // list under a live index, and the position must land on a real card.
    expect(clampIndex(3, 2)).toBe(1);
  });

  it("answers 0 for an empty list rather than -1", () => {
    expect(clampIndex(3, 0)).toBe(0);
  });
});

describe("swipeStep", () => {
  it("reads a leftward drag as forward, a rightward one as back", () => {
    expect(swipeStep(-80)).toBe(1);
    expect(swipeStep(80)).toBe(-1);
  });

  it("ignores a drag too small to be a swipe — a tap must not page", () => {
    expect(swipeStep(0)).toBe(0);
    expect(swipeStep(-20)).toBe(0);
    expect(swipeStep(20)).toBe(0);
  });
});
