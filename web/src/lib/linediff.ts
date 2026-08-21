// Pure-TS line diff for the original-vs-fixed audit trail. No dependency —
// the engine's fixes are line edits, so the two files we compare are nearly
// identical and the right algorithm is Myers' greedy O(ND) SES (Eugene W.
// Myers, "An O(ND) Difference Algorithm and Its Variations", 1986): fast
// exactly when the edit distance D is small, which is our case.
//
// Two guards keep it bounded on a pathological input (e.g. a huge file
// rewritten wholesale):
//   * common prefix/suffix are trimmed up front (O(n)) — almost always the
//     bulk of the file, since fixes touch a handful of lines.
//   * the Myers search is capped at MAX_D; past that the still-differing
//     middle is emitted as a single replace block (every old line removed,
//     every new line added) and `capped` is set so the UI can say so —
//     no silent truncation.
//
// Every `!` below is a non-null assertion on an array / typed-array index that
// the surrounding loop bounds PROVE in range (each is justified inline). Under
// noUncheckedIndexedAccess TypeScript still types those reads `T | undefined`;
// the assertion is the honest expression of a hand-verified invariant, and
// narrowing each read would add branches to correct, test-pinned hot loops for
// zero runtime benefit — so no-non-null-assertion is disabled file-wide (laterite-dev#615).
/* eslint-disable @typescript-eslint/no-non-null-assertion */

export type DiffType = "eq" | "del" | "ins";

/** One diff operation over lines. `aLine`/`bLine` are 1-based line numbers
 *  in the original (a) / current (b); `eq` carries both, `del` only `aLine`,
 *  `ins` only `bLine`. */
export interface DiffOp {
  type: DiffType;
  aLine?: number;
  bLine?: number;
  text: string;
}

export interface DiffResult {
  ops: DiffOp[];
  added: number;
  removed: number;
  /** True if the Myers search hit MAX_D and a block fallback was used for
   *  the changed middle (line-level pairing within it is approximate). */
  capped: boolean;
}

// Beyond this edit distance the line-by-line script stops being worth the
// memory (the trace stores one V-band per d). A fix diff is in the tens;
// this only trips on near-unrelated files.
const MAX_D = 2500;

/** Diff two arrays of lines. */
export function diffLines(a: string[], b: string[]): DiffResult {
  const ops: DiffOp[] = [];
  let added = 0;
  let removed = 0;

  // --- trim common prefix ---
  let lo = 0;
  const minLen = Math.min(a.length, b.length);
  while (lo < minLen && a[lo] === b[lo]) lo++;
  for (let i = 0; i < lo; i++) {
    // i < lo ≤ min(a.length, b.length) → a[i] is in-bounds.
    ops.push({ type: "eq", aLine: i + 1, bLine: i + 1, text: a[i]! });
  }

  // --- trim common suffix (not into the already-matched prefix) ---
  let aHi = a.length;
  let bHi = b.length;
  while (aHi > lo && bHi > lo && a[aHi - 1] === b[bHi - 1]) {
    aHi--;
    bHi--;
  }

  const aMid = a.slice(lo, aHi);
  const bMid = b.slice(lo, bHi);

  // --- diff the middle ---
  const middle = diffMiddle(aMid, bMid, lo);
  for (const op of middle.ops) {
    ops.push(op);
    if (op.type === "ins") added++;
    else if (op.type === "del") removed++;
  }

  // --- suffix (the common tail trimmed above) ---
  for (let i = aHi; i < a.length; i++) {
    const offset = i - aHi;
    // i < a.length → a[i] is in-bounds.
    ops.push({
      type: "eq",
      aLine: i + 1,
      bLine: bHi + offset + 1,
      text: a[i]!,
    });
  }

  // eslint-disable-next-line @typescript-eslint/no-unnecessary-condition -- Vite types import.meta.env as always-injected, but it's absent outside a Vite bundle (bare ts-node/vitest paths)
  if (import.meta.env?.DEV && !middle.capped) checkInvariant(ops, a, b);

  return { ops, added, removed, capped: middle.capped };
}

// The reconstruction invariant is load-bearing (a wrong backtrack silently
// shows the wrong lines): the eq+del ops must rebuild `a` in order, the
// eq+ins ops must rebuild `b`. Dev-only; tree-shaken from prod.
function checkInvariant(ops: DiffOp[], a: string[], b: string[]): void {
  const fromA = ops.filter((o) => o.type !== "ins").map((o) => o.text);
  const fromB = ops.filter((o) => o.type !== "del").map((o) => o.text);
  const eq = (x: string[], y: string[]) =>
    x.length === y.length && x.every((v, i) => v === y[i]);
  if (!eq(fromA, a) || !eq(fromB, b)) {
    console.error("diffLines reconstruction invariant broken", {
      aOk: eq(fromA, a),
      bOk: eq(fromB, b),
    });
  }
}

interface MiddleResult {
  ops: DiffOp[];
  capped: boolean;
}

/** Diff the trimmed middle. `base` is the 0-based offset of this slice in the
 *  full arrays, so emitted line numbers are absolute. */
function diffMiddle(a: string[], b: string[], base: number): MiddleResult {
  if (a.length === 0 && b.length === 0) return { ops: [], capped: false };
  if (a.length === 0) {
    return {
      ops: b.map((text, i) => ({ type: "ins", bLine: base + i + 1, text })),
      capped: false,
    };
  }
  if (b.length === 0) {
    return {
      ops: a.map((text, i) => ({ type: "del", aLine: base + i + 1, text })),
      capped: false,
    };
  }

  const trace = myersTrace(a, b);
  if (!trace) {
    // Over MAX_D — emit the whole middle as a replace block.
    const ops: DiffOp[] = [
      ...a.map<DiffOp>((text, i) => ({
        type: "del",
        aLine: base + i + 1,
        text,
      })),
      ...b.map<DiffOp>((text, i) => ({
        type: "ins",
        bLine: base + i + 1,
        text,
      })),
    ];
    return { ops, capped: true };
  }
  return { ops: backtrack(trace, a, b, base), capped: false };
}

// --- Myers greedy SES (forward D-path search, full trace for backtracking) ---

/** Run the forward search, recording the V-band after each round so we can
 *  backtrack the edit script. Returns null if D exceeds MAX_D. */
function myersTrace(a: string[], b: string[]): Int32Array[] | null {
  const n = a.length;
  const m = b.length;
  const max = n + m;
  const off = max; // index shift so diagonal k ∈ [-max, max] maps to ≥ 0
  const v = new Int32Array(2 * max + 1);
  const trace: Int32Array[] = [];

  const limit = Math.min(max, MAX_D);
  for (let d = 0; d <= limit; d++) {
    trace.push(v.slice());
    for (let k = -d; k <= d; k += 2) {
      // Choose to extend the furthest-reaching path: move down (insertion)
      // when at the lower edge or the down-neighbour reached further.
      let x: number;
      // v-band indices off+k±1 ∈ [0, 2*max] by construction (k ∈ [−d, d],
      // d ≤ max, off = max) → every band read is in-bounds.
      if (k === -d || (k !== d && v[off + k - 1]! < v[off + k + 1]!)) {
        x = v[off + k + 1]!;
      } else {
        x = v[off + k - 1]! + 1;
      }
      let y = x - k;
      while (x < n && y < m && a[x] === b[y]) {
        x++;
        y++;
      }
      v[off + k] = x;
      if (x >= n && y >= m) return trace;
    }
  }
  return null; // unreachable within MAX_D
}

/** Walk the trace backwards into a forward-ordered op list. */
function backtrack(
  trace: Int32Array[],
  a: string[],
  b: string[],
  base: number,
): DiffOp[] {
  const n = a.length;
  const m = b.length;
  const off = n + m;
  const ops: DiffOp[] = [];
  let x = n;
  let y = m;

  for (let d = trace.length - 1; d > 0; d--) {
    // d ∈ [1, trace.length−1] → trace[d] is in-bounds; the v-band and snake
    // indices below are all provably in range by the Myers path invariant.
    const v = trace[d]!;
    const k = x - y;
    // Which neighbour the furthest path came from (mirror of the forward step).
    const prevK =
      k === -d || (k !== d && v[off + k - 1]! < v[off + k + 1]!)
        ? k + 1
        : k - 1;
    const prevX = v[off + prevK]!;
    const prevY = prevX - prevK;

    // Diagonal (equal) moves between the snake end and the previous point.
    while (x > prevX && y > prevY) {
      x--;
      y--;
      ops.push({
        type: "eq",
        aLine: base + x + 1,
        bLine: base + y + 1,
        text: a[x]!,
      });
    }
    if (x === prevX) {
      // moved down → insertion of b[prevY]
      ops.push({ type: "ins", bLine: base + prevY + 1, text: b[prevY]! });
    } else {
      // moved right → deletion of a[prevX]
      ops.push({ type: "del", aLine: base + prevX + 1, text: a[prevX]! });
    }
    x = prevX;
    y = prevY;
  }
  // d === 0: any remaining snake is the leading diagonal of equal lines.
  while (x > 0 && y > 0) {
    x--;
    y--;
    ops.push({
      type: "eq",
      aLine: base + x + 1,
      bLine: base + y + 1,
      text: a[x]!,
    });
  }

  ops.reverse();
  return ops;
}

// --- hunk grouping (unified-diff style) ---

/** A display row: a diff op, or a `gap` collapsing `count` unchanged lines. */
export type DiffRow =
  (DiffOp & { type: DiffType }) | { type: "gap"; count: number };

/**
 * Collapse long runs of unchanged lines, keeping `context` equal lines on
 * either side of every change. Each collapsed run becomes one `gap` row.
 * Mirrors the "⋯ N more" windowing the findings view uses for GROUP blocks.
 */
export function toHunks(ops: DiffOp[], context = 3): DiffRow[] {
  const n = ops.length;
  // Mark each eq op for keeping if it's within `context` of a change.
  const keep = new Array<boolean>(n).fill(false);
  for (let i = 0; i < n; i++) {
    // i, j < n = ops.length → in-bounds throughout this loop.
    if (ops[i]!.type === "eq") continue;
    for (
      let j = Math.max(0, i - context);
      j <= Math.min(n - 1, i + context);
      j++
    ) {
      keep[j] = true;
    }
  }

  const rows: DiffRow[] = [];
  let gap = 0;
  for (let i = 0; i < n; i++) {
    const op = ops[i]!; // i < n = ops.length → in-bounds.
    if (op.type === "eq" && !keep[i]) {
      gap++;
      continue;
    }
    if (gap > 0) {
      rows.push({ type: "gap", count: gap });
      gap = 0;
    }
    rows.push(op);
  }
  if (gap > 0) rows.push({ type: "gap", count: gap });
  return rows;
}
