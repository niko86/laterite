import { Button, Checkbox, Input, Select } from "@shared/components";
import {
  createEffect,
  createMemo,
  createSignal,
  For,
  onCleanup,
  Show,
  type Component,
} from "solid-js";
import type { GroupMeta } from "../../lib/duckTypes";
import { Chevron } from "../Chevron";
import {
  selectSql,
  type Cond,
  type JoinSpec,
  type Wildcard,
} from "../../lib/sqlgen";
import {
  relatedGroups,
  joinKeys,
  depthRangeOf,
  depthColumnFor,
  type DictMap,
} from "../../lib/relationships";

// Visual SQL builder: table → (optional related group) → columns → WHERE rows →
// ORDER BY → LIMIT, composing a SELECT handed to the console via onApply. With a
// related group it auto-joins on the shared compound key (no hand-written ON),
// and columns/filters can come from either table. A scaffold, not a replacement
// for the editor — anything it can't express, edit by hand after applying.

const OPS = [
  "=",
  "!=",
  ">",
  "<",
  ">=",
  "<=",
  "LIKE",
  "IS NULL",
  "IS NOT NULL",
] as const;
const WILDCARDS: { v: Wildcard; label: string }[] = [
  { v: "contains", label: "contains" },
  { v: "starts", label: "starts with" },
  { v: "ends", label: "ends with" },
  { v: "exact", label: "exact" },
];

const BASE = "c"; // base-table alias
const JOIN = "j"; // related-table alias

interface ColRef {
  alias: string;
  col: string;
  key: string; // unique pick key: col (single) or "alias.col" (joined)
  label: string;
}

export const SqlBuilder: Component<{
  groups: GroupMeta[];
  dict: DictMap | undefined;
  onApply: (sql: string) => void;
}> = (props) => {
  const codes = () => props.groups.map((g) => g.code);
  const [table, setTable] = createSignal("");
  const [joinCode, setJoinCode] = createSignal(""); // "" = single-table
  const [joinKind, setJoinKind] = createSignal<"LEFT" | "INNER">("LEFT");
  createEffect(() => {
    const first = codes()[0];
    if (!table() && first !== undefined) setTable(first);
  });

  const baseMeta = () => props.groups.find((g) => g.code === table());
  const joinMeta = () => props.groups.find((g) => g.code === joinCode());
  const joined = () => !!joinCode() && !!joinMeta();

  // Related groups (ancestors + present children) for the chosen base.
  const related = createMemo(() =>
    props.dict && table() ? relatedGroups(table(), codes(), props.dict) : [],
  );
  // The shared-key ON pairs for the active join (empty ⇒ no usable link).
  const joinPairs = createMemo(() => {
    const b = baseMeta();
    const j = joinMeta();
    if (!props.dict || !joined() || !b || !j) return [];
    return joinKeys(
      { code: b.code, cols: b.headings },
      { code: j.code, cols: j.headings },
      props.dict,
    );
  });

  // When the related group is a depth-range group (GEOL) and the base has a
  // depth column, the join gains a depth-band predicate — the stratum case.
  const rangePred = createMemo(() => {
    const b = baseMeta();
    const j = joinMeta();
    if (!props.dict || !joined() || !b || !j) return null;
    // cols-aware: only a band whose top+base columns the related table actually
    // carries (so we never emit a predicate on a missing *_BASE column).
    const dr = depthRangeOf(joinCode(), props.dict, j.headings);
    const dc = dr ? depthColumnFor(b.code, b.headings, props.dict) : null;
    return dr && dc
      ? {
          baseAlias: BASE,
          baseCol: dc.col,
          top: dr.top,
          base: dr.base,
          level: dc.level,
        }
      : null;
  });

  // Selectable columns: base (alias c); when joined, also related (alias j).
  const allCols = createMemo<ColRef[]>(() => {
    const b = baseMeta();
    if (!b) return [];
    const j = joinMeta();
    // `joined()` is `!!joinCode() && !!joinMeta()`, inlined here so `j` narrows.
    if (!joinCode() || !j)
      return b.headings.map((col) => ({
        alias: BASE,
        col,
        key: col,
        label: col,
      }));
    return [
      ...b.headings.map((col) => ({
        alias: BASE,
        col,
        key: `${BASE}.${col}`,
        label: `${b.code}.${col}`,
      })),
      ...j.headings.map((col) => ({
        alias: JOIN,
        col,
        key: `${JOIN}.${col}`,
        label: `${j.code}.${col}`,
      })),
    ];
  });
  const orderCols = () => baseMeta()?.headings ?? [];

  const [picked, setPicked] = createSignal<Set<string>>(new Set());
  const [conds, setConds] = createSignal<Cond[]>([]);
  const [orderBy, setOrderBy] = createSignal("");
  const [orderDir, setOrderDir] = createSignal<"ASC" | "DESC">("ASC");
  const [limit, setLimit] = createSignal(100);

  // Reset column-dependent selections when the base table changes (a related
  // group change keeps them — its columns just become newly available).
  createEffect(() => {
    table();
    setJoinCode("");
    setPicked(new Set<string>());
    setConds([]);
    setOrderBy("");
  });

  const togglePick = (key: string) =>
    setPicked((prev) => {
      const next = new Set(prev);
      if (next.has(key)) next.delete(key);
      else next.add(key);
      return next;
    });

  const addCond = () => {
    const c = allCols()[0];
    // Carry the alias only in join mode; single-table conditions stay unaliased
    // so the column <select> value matches its bare-named options.
    if (c)
      setConds((prev) => [
        ...prev,
        { col: c.col, alias: joined() ? c.alias : undefined, op: "=", val: "" },
      ]);
  };
  const setCond = (i: number, patch: Partial<Cond>) =>
    setConds((prev) => prev.map((c, j) => (j === i ? { ...c, ...patch } : c)));
  const delCond = (i: number) =>
    setConds((prev) => prev.filter((_, j) => j !== i));

  const sql = createMemo(() => {
    const base = table();
    if (!base) return "";
    const pickedRefs = allCols().filter((r) => picked().has(r.key));
    if (!joined() || joinPairs().length === 0) {
      // Single-table (legacy) path. A related group may be selected yet yield no
      // usable join key (joinPairs empty); any picked related-table (j) column
      // or filter must then be DROPPED, not emitted unqualified against the base
      // table (a DuckDB "column not found"). Keep base-alias refs only.
      return selectSql({
        table: base,
        columns: pickedRefs.filter((r) => r.alias === BASE).map((r) => r.col),
        conditions: conds()
          .filter((c) => (c.alias ?? BASE) === BASE)
          .map((c) => ({ ...c, alias: undefined })),
        orderBy: orderBy() || undefined,
        orderDir: orderDir(),
        limit: limit(),
      });
    }
    const rp = rangePred();
    const join: JoinSpec = {
      table: joinCode(),
      alias: JOIN,
      kind: joinKind(),
      leftAlias: BASE,
      on: joinPairs(),
      range: rp
        ? {
            baseAlias: rp.baseAlias,
            baseCol: rp.baseCol,
            top: rp.top,
            base: rp.base,
          }
        : undefined,
    };
    return selectSql({
      table: base,
      alias: BASE,
      joins: [join],
      select: pickedRefs.map((r) => ({ alias: r.alias, col: r.col })),
      columns: [],
      conditions: conds(),
      orderBy: orderBy() || undefined,
      orderDir: orderDir(),
      limit: limit(),
    });
  });

  // Group dropdowns show "CODE — Full name" (the dictionary `contents`),
  // truncated to the viewport so a long name (e.g. GEOL's "Field Geological
  // Descriptions") can't blow out the control on a phone. Reactive to resize so
  // it re-fits on rotate / window-drag rather than freezing at first paint.
  const [vw, setVw] = createSignal(
    typeof window !== "undefined" ? window.innerWidth : 1024,
  );
  if (typeof window !== "undefined") {
    const onResize = () => setVw(window.innerWidth);
    window.addEventListener("resize", onResize);
    onCleanup(() => {
      window.removeEventListener("resize", onResize);
    });
  }
  const nameCap = () => (vw() >= 1024 ? 44 : vw() >= 640 ? 26 : 16);
  const trunc = (s: string, n: number) =>
    s.length > n ? `${s.slice(0, n - 1).trimEnd()}…` : s;
  const groupLabel = (code: string) => {
    const c = props.dict?.get(code)?.contents ?? "";
    return c ? `${code} — ${trunc(c, nameCap())}` : code;
  };

  // The four field controls share the app-wide standard control look; the
  // captions label the blocks (Source / Columns / Filters / Output) so the
  // builder reads as sections instead of one flat row of look-alike selects.
  const caption =
    "mb-1.5 text-[11px] font-semibold uppercase tracking-wide text-fg-faint";

  return (
    <details class="group rounded-lg border border-line bg-surface">
      <summary class="flex cursor-pointer list-none select-none items-center gap-2 px-3 py-2 text-sm font-medium text-fg-soft [&::-webkit-details-marker]:hidden">
        <Chevron />
        Build a query with controls
      </summary>
      <div class="flex flex-col gap-4 border-t border-line-subtle p-3">
        {/* SOURCE — the FROM / JOIN clause, read like SQL. */}
        <div>
          <p class={caption}>Source</p>
          <div class="flex flex-col gap-2 rounded-md border border-line bg-surface p-2.5">
            <div class="flex flex-wrap items-center gap-2">
              <span class="mono w-11 shrink-0 text-[11px] font-semibold tracking-wide text-accent">
                FROM
              </span>
              <Select
                aria-label="table"
                class="font-mono min-w-0 flex-1 max-w-md"
                value={table()}
                onChange={(e) => setTable(e.currentTarget.value)}
              >
                <For each={codes()}>
                  {(c) => <option value={c}>{groupLabel(c)}</option>}
                </For>
              </Select>
            </div>
            <Show when={related().length > 0}>
              <div class="flex flex-wrap items-center gap-2">
                <span
                  class={`mono w-11 shrink-0 text-[11px] font-semibold tracking-wide ${joined() ? "text-accent" : "text-fg-dim"}`}
                >
                  JOIN
                </span>
                <Show when={joined()}>
                  <Select
                    aria-label="join type"
                    class="w-auto"
                    value={joinKind()}
                    onChange={(e) =>
                      setJoinKind(e.currentTarget.value as "LEFT" | "INNER")
                    }
                  >
                    <option value="LEFT">LEFT</option>
                    <option value="INNER">INNER</option>
                  </Select>
                </Show>
                <Select
                  aria-label="related group"
                  class="font-mono min-w-0 flex-1 max-w-md"
                  value={joinCode()}
                  onChange={(e) => {
                    const v = e.currentTarget.value;
                    setJoinCode(v);
                    // Un-joining: drop any picked column / filter that referenced
                    // the join table (alias j) so no stale row lingers pointing at
                    // a column that's no longer in allCols().
                    if (!v) {
                      setPicked(
                        (p) =>
                          new Set(
                            [...p].filter((k) => !k.startsWith(`${JOIN}.`)),
                          ),
                      );
                      setConds((cs) =>
                        cs.filter((c) => (c.alias ?? BASE) === BASE),
                      );
                    }
                  }}
                >
                  <option value="">(none — single table)</option>
                  <For each={related()}>
                    {(r) => (
                      <option value={r.code}>
                        {`${groupLabel(r.code)} · ${r.direction}`}
                      </option>
                    )}
                  </For>
                </Select>
              </div>
            </Show>
          </div>
        </div>

        <Show when={rangePred()}>
          {(rp) => (
            <p class="text-xs text-fg-faint">
              Depth-band join: <span class="mono">{rp().baseCol}</span> within{" "}
              <span class="mono">{rp().top}</span>…
              <span class="mono">{rp().base}</span> ({rp().level}-level) — each
              row gets the stratum it sits in.
            </p>
          )}
        </Show>

        <div>
          <p class={caption}>
            Columns{" "}
            <span class="font-normal normal-case tracking-normal text-fg-dim">
              — none ticked ⇒ <span class="mono">SELECT *</span>
            </span>
          </p>
          <div class="flex flex-wrap gap-x-3 gap-y-1">
            <For each={allCols()}>
              {(r) => (
                <Checkbox
                  mono
                  label={r.label}
                  checked={picked().has(r.key)}
                  onChange={() => togglePick(r.key)}
                />
              )}
            </For>
          </div>
        </div>

        <div class="flex flex-col gap-1.5">
          <div class="flex items-center gap-2">
            <span class={`${caption} mb-0`}>
              Filters{" "}
              <span class="font-normal normal-case tracking-normal text-fg-dim">
                (WHERE)
              </span>
            </span>
            <Button variant="add" onClick={addCond}>
              + add
            </Button>
          </div>
          <For each={conds()}>
            {(cond, i) => (
              <div class="flex flex-wrap items-center gap-1.5 text-xs">
                <Select
                  aria-label="filter column"
                  class="w-auto"
                  value={
                    joined() ? `${cond.alias ?? BASE}.${cond.col}` : cond.col
                  }
                  onChange={(e) => {
                    const ref = allCols().find(
                      (r) => r.key === e.currentTarget.value,
                    );
                    if (ref)
                      setCond(i(), {
                        col: ref.col,
                        alias: joined() ? ref.alias : undefined,
                      });
                  }}
                >
                  <For each={allCols()}>
                    {(r) => <option value={r.key}>{r.label}</option>}
                  </For>
                </Select>
                <Select
                  aria-label="filter operator"
                  class="w-auto"
                  value={cond.op}
                  onChange={(e) => setCond(i(), { op: e.currentTarget.value })}
                >
                  <For each={OPS}>{(o) => <option value={o}>{o}</option>}</For>
                </Select>
                <Show when={cond.op === "LIKE"}>
                  <Select
                    aria-label="filter wildcard"
                    class="w-auto"
                    value={cond.wildcard ?? "contains"}
                    onChange={(e) =>
                      setCond(i(), {
                        wildcard: e.currentTarget.value as Wildcard,
                      })
                    }
                  >
                    <For each={WILDCARDS}>
                      {(w) => <option value={w.v}>{w.label}</option>}
                    </For>
                  </Select>
                </Show>
                <Show when={cond.op !== "IS NULL" && cond.op !== "IS NOT NULL"}>
                  <Input
                    class="w-32"
                    placeholder="value"
                    value={cond.val}
                    onInput={(e) =>
                      setCond(i(), { val: e.currentTarget.value })
                    }
                  />
                </Show>
                {/* Native, not Button: the states contract's destructive
                    ghost (muted at rest, err on hover) — the Button
                    primitive's danger tone is the ARMED repaint, constant
                    err, which would shout from every row. */}
                <button
                  type="button"
                  class="text-fg-dim hover:text-err"
                  title="Remove filter"
                  onClick={() => delCond(i())}
                >
                  ×
                </button>
              </div>
            )}
          </For>
        </div>

        {/* OUTPUT — how the result is shaped, split out from the source selects
            so ORDER BY / LIMIT no longer compete with FROM / JOIN for the eye. */}
        <div>
          <p class={caption}>Output</p>
          <div class="flex flex-wrap items-center gap-x-5 gap-y-2">
            <label class="flex items-center gap-2 text-xs text-fg-muted">
              Order by
              <Select
                class="w-auto"
                value={orderBy()}
                onChange={(e) => setOrderBy(e.currentTarget.value)}
              >
                <option value="">(none)</option>
                <For each={orderCols()}>
                  {(c) => <option value={c}>{c}</option>}
                </For>
              </Select>
              <Select
                aria-label="order direction"
                class="w-auto"
                value={orderDir()}
                onChange={(e) =>
                  setOrderDir(e.currentTarget.value as "ASC" | "DESC")
                }
              >
                <option value="ASC">ASC</option>
                <option value="DESC">DESC</option>
              </Select>
            </label>
            <label class="flex items-center gap-2 text-xs text-fg-muted">
              Limit
              <Input
                type="number"
                min="0"
                title="0 = no limit (every row)"
                class="w-20"
                value={limit()}
                onInput={(e) => setLimit(Number(e.currentTarget.value) || 0)}
              />
              {/* The generated SQL omits LIMIT when this is 0 — make that
                  explicit so an emptied box doesn't look like a stuck 100. */}
              <span class="text-fg-dim">{limit() === 0 ? "no limit" : ""}</span>
            </label>
          </div>
        </div>

        {/* The generated SQL is always multi-line, so align the button to the
            top of the preview (not centred against a tall block) and stack them
            on a phone instead of competing for the narrow width. */}
        <div class="flex flex-col gap-2 sm:flex-row sm:items-start sm:gap-3">
          <Button
            class="shrink-0 self-start"
            onClick={() => {
              props.onApply(sql());
            }}
          >
            Use this SQL ↓
          </Button>
          <pre class="mono min-w-0 flex-1 overflow-x-auto rounded-sm border border-line bg-surface-raised p-2 text-xs text-fg-soft">
            {sql()}
          </pre>
        </div>
      </div>
    </details>
  );
};
