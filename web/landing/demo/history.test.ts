/* The undo stack (#525). Pure data over arrays, node-testable like the rest of
 * the model — the store wires it to the delivery signal, but nothing here
 * knows Solid exists.
 */

import { describe, expect, it } from "vitest";
import { EMPTY, record, redo, undo, type History } from "./history";

describe("record", () => {
  it("remembers the state being replaced and clears the redo branch", () => {
    let h: History<string> = EMPTY;
    h = record(h, "a");
    h = record(h, "b");
    expect(h.past).toEqual(["a", "b"]);
    expect(h.future).toEqual([]);
  });

  it("drops the oldest snapshot at the cap instead of growing unbounded", () => {
    let h: History<number> = EMPTY;
    for (let i = 0; i < 150; i++) h = record(h, i, 100);
    expect(h.past).toHaveLength(100);
    expect(h.past.at(0)).toBe(50);
    expect(h.past.at(-1)).toBe(149);
  });
});

describe("undo / redo", () => {
  it("walks back through recorded states and forward again", () => {
    let h: History<string> = EMPTY;
    h = record(h, "a");
    h = record(h, "b");
    // Present is "c"; the stack holds what it replaced.
    const back = undo(h, "c");
    expect(back?.present).toBe("b");
    expect(back?.history.future).toEqual(["c"]);
    const backTwice = undo(back!.history, back!.present);
    expect(backTwice?.present).toBe("a");
    const forward = redo(backTwice!.history, backTwice!.present);
    expect(forward?.present).toBe("b");
    // The present is never IN the stack: after redo lands on "b", the past
    // holds only what precedes it and the future what was undone past it.
    expect(forward?.history.past).toEqual(["a"]);
    expect(forward?.history.future).toEqual(["c"]);
  });

  it("answers null at either end rather than inventing a state", () => {
    expect(undo(EMPTY, "only")).toBeNull();
    expect(redo(EMPTY, "only")).toBeNull();
  });

  it("a new edit after undo abandons the redo branch — one timeline", () => {
    let h: History<string> = EMPTY;
    h = record(h, "a");
    const back = undo(h, "b");
    const edited = record(back!.history, back!.present);
    expect(edited.future).toEqual([]);
    expect(redo(edited, "b2")).toBeNull();
  });
});
