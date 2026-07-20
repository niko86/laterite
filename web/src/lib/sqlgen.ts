// Pure SQL-string composition for the Explore builders (ChartBuilder +
// SqlBuilder). Kept out of the components so the quoting / aggregate / WHERE /
// JOIN logic — the part most prone to subtle regressions — is unit-tested.

/** Quote a SQL identifier (table/column), doubling internal quotes so a
 *  heading with a `"` can't break out. */
export const q = (id: string) => `"${id.replace(/"/g, '""')}"`;

/** A WHERE value literal: a bare number stays unquoted (numeric comparison);
 *  anything else becomes a single-quoted string literal (quotes escaped). */
export const lit = (v: string) =>
  v.trim() !== "" && !Number.isNaN(Number(v))
    ? v.trim()
    : `'${v.replace(/'/g, "''")}'`;

// --- LIKE wildcard control ---------------------------------------------------
export type Wildcard = "contains" | "starts" | "ends" | "exact";

/** Build a LIKE pattern literal with the chosen wildcard placement. User input
 *  is treated as LITERAL: the LIKE metacharacters `% _ \` are backslash-escaped
 *  so only the wildcard(s) WE add are patterns — so the predicate must carry
 *  `ESCAPE '\'`. `contains`→%v%, `starts`→v%, `ends`→%v, `exact`→v. */
export function likeLiteral(v: string, w: Wildcard = "contains"): string {
  const esc = v.replace(/([\\%_])/g, "\\$1").replace(/'/g, "''");
  const body =
    w === "contains"
      ? `%${esc}%`
      : w === "starts"
        ? `${esc}%`
        : w === "ends"
          ? `%${esc}`
          : esc;
  return `'${body}'`;
}

// --- joins -------------------------------------------------------------------
/** A column reference qualified by a table alias (for join queries). `as` is
 *  the output column name (defaults to `col`, deduped on collision). */
export interface QualifiedCol {
  alias: string;
  col: string;
  as?: string;
}

/** One joined table. `on` is the equi-join column pairs (leftAlias.left =
 *  alias.right). `range`, when set, ADDS a half-open depth band
 *  `baseAlias.baseCol >= alias.top AND baseAlias.baseCol < alias.base` — the
 *  GEOL stratum case. `kind` defaults LEFT so unmatched base rows survive. */
export interface JoinSpec {
  table: string;
  alias: string;
  kind: "LEFT" | "INNER";
  leftAlias: string;
  on: { left: string; right: string }[];
  range?: { baseAlias: string; baseCol: string; top: string; base: string };
}

/** Emit the FROM + JOIN clauses for an aliased base + its joins. */
function fromJoins(table: string, alias: string, joins: JoinSpec[]): string {
  let s = `FROM ${q(table)} ${alias}`;
  for (const j of joins) {
    const preds = [
      ...j.on.map(
        (p) => `${j.leftAlias}.${q(p.left)} = ${j.alias}.${q(p.right)}`,
      ),
      ...(j.range
        ? [
            `${j.range.baseAlias}.${q(j.range.baseCol)} >= ${j.alias}.${q(j.range.top)}`,
            `${j.range.baseAlias}.${q(j.range.baseCol)} < ${j.alias}.${q(j.range.base)}`,
          ]
        : []),
    ].join(" AND ");
    s += `\n${j.kind} JOIN ${q(j.table)} ${j.alias} ON ${preds}`;
  }
  return s;
}

/** Give each picked column a unique output name (a join can surface two
 *  same-named columns, e.g. LOCA_ID from both sides). First wins its plain
 *  name; later collisions get `<alias>_<name>` (then `_2`, … if still clashing). */
function dedupeOut(cols: QualifiedCol[]): QualifiedCol[] {
  const seen = new Set<string>();
  return cols.map((c) => {
    let out = c.as ?? c.col;
    if (seen.has(out)) out = `${c.alias}_${out}`;
    let n = out;
    let i = 2;
    while (seen.has(n)) n = `${out}_${i++}`;
    seen.add(n);
    return { ...c, as: n };
  });
}

// --- WHERE conditions (shared by single-table + join paths) ------------------
export interface Cond {
  col: string;
  /** table alias (join mode); when set the ref is `alias."col"`. */
  alias?: string;
  op: string;
  val: string;
  /** Only meaningful when op === "LIKE". Default "contains". */
  wildcard?: Wildcard;
}

/** Drop an INCOMPLETE filter: a value-operator with no value would emit
 *  `"COL" = ''`, a DuckDB conversion error on numeric/date columns that can
 *  wedge the engine. Applies only once the user has entered a value. */
const keepCond = (c: Cond): boolean =>
  !!c.col &&
  (c.op === "IS NULL" || c.op === "IS NOT NULL" || c.val.trim() !== "");

/** Render one predicate against an already-built column reference (`ref` is
 *  `"col"` single-table, or `alias."col"` in join mode). */
function renderCond(ref: string, c: Cond): string {
  if (c.op === "IS NULL" || c.op === "IS NOT NULL") return `${ref} ${c.op}`;
  if (c.op === "LIKE")
    return `${ref} LIKE ${likeLiteral(c.val, c.wildcard)} ESCAPE '\\'`;
  return `${ref} ${c.op} ${lit(c.val)}`;
}

// --- chart query -------------------------------------------------------------
export type ChartType = "scatter" | "line" | "bar";
export type Agg = "none" | "count" | "sum" | "avg" | "min" | "max";

export interface ChartSqlOpts {
  table: string;
  /** base alias (join mode). */
  alias?: string;
  joins?: JoinSpec[];
  x: string | QualifiedCol;
  y: string | QualifiedCol;
  colour?: string | QualifiedCol;
  chartType: ChartType;
  agg: Agg;
  rowCap: number;
}

/** Resolve a column to its SQL reference: `"col"` single-table; `alias."col"`
 *  when joins are present (string ⇒ the base alias, QualifiedCol ⇒ its alias). */
function colRef(
  c: string | QualifiedCol,
  baseAlias: string,
  joined: boolean,
): string {
  if (typeof c !== "string") return `${c.alias}.${q(c.col)}`;
  return joined ? `${baseAlias}.${q(c)}` : q(c);
}

/** Compose the chart query. Returns "" when the selection is incomplete (no
 *  table/X, or no Y unless counting). Scatter/line select raw X/Y (line is
 *  ordered by X); bar with an aggregate GROUP BYs the X category (+ colour).
 *  With `joins`, X/Y/colour are alias-qualified and the JOINs are emitted; the
 *  output aliases (x/y/c) are unchanged so the ECharts mapping is untouched. */
export function chartSql(o: ChartSqlOpts): string {
  const { table, x, y, colour, chartType, agg, rowCap } = o;
  const counting = agg === "count";
  const aggregating = chartType === "bar" && agg !== "none";
  const joins = o.joins ?? [];
  const joined = joins.length > 0;
  const a = o.alias ?? "t0";
  if (!table || !x) return "";
  if (!counting && !y) return "";

  const xr = colRef(x, a, joined);
  const yr = y ? colRef(y, a, joined) : "";
  const cr = colour ? colRef(colour, a, joined) : "";
  const from = joined ? fromJoins(table, a, joins) : `FROM ${q(table)}`;
  const selC = cr ? `, ${cr} AS c` : "";
  if (aggregating) {
    const yExpr = counting ? "COUNT(*)" : `${agg.toUpperCase()}(${yr})`;
    const groupBy = cr ? `${xr}, ${cr}` : xr;
    const where = counting ? "" : ` WHERE ${yr} IS NOT NULL`;
    return (
      `SELECT ${xr} AS x, ${yExpr} AS y${selC} ${from}${where}` +
      ` GROUP BY ${groupBy} ORDER BY x LIMIT ${rowCap}`
    );
  }
  const order = chartType === "line" ? ` ORDER BY ${xr}` : "";
  return (
    `SELECT ${xr} AS x, ${yr} AS y${selC} ${from}` +
    ` WHERE ${xr} IS NOT NULL AND ${yr} IS NOT NULL${order} LIMIT ${rowCap}`
  );
}

// --- SELECT query ------------------------------------------------------------
export interface SelectOpts {
  table: string;
  /** SELECT list for the single-table path; empty ⇒ `SELECT *`. */
  columns: string[];
  conditions: Cond[];
  orderBy?: string;
  orderDir: "ASC" | "DESC";
  limit: number;
  // --- join mode (when `joins` is non-empty) ---
  /** base-table alias. */
  alias?: string;
  joins?: JoinSpec[];
  /** SELECT list (qualified) for join mode; empty ⇒ `<baseAlias>.*`. */
  select?: QualifiedCol[];
}

/** Compose a SELECT from the visual builder's controls. Returns "" with no
 *  table. The single-table path (no `joins`) is byte-identical to before
 *  EXCEPT LIKE now applies the wildcard. Join mode emits an aliased,
 *  qualified query with deduped output names. */
export function selectSql(o: SelectOpts): string {
  if (!o.table) return "";
  const kept = o.conditions.filter(keepCond);

  if (!o.joins || o.joins.length === 0) {
    // Single-table (legacy) path.
    const cols = o.columns.length === 0 ? "*" : o.columns.map(q).join(", ");
    let s = `SELECT ${cols}\nFROM ${q(o.table)}`;
    const where = kept.map((c) => renderCond(q(c.col), c));
    if (where.length) s += `\nWHERE ${where.join("\n  AND ")}`;
    if (o.orderBy) s += `\nORDER BY ${q(o.orderBy)} ${o.orderDir}`;
    if (o.limit > 0) s += `\nLIMIT ${Math.floor(o.limit)}`;
    return s;
  }

  // Join mode.
  const a = o.alias ?? "t0";
  const selList =
    o.select && o.select.length
      ? dedupeOut(o.select)
          .map((c) => `${c.alias}.${q(c.col)} AS ${q(c.as ?? c.col)}`)
          .join(", ")
      : `${a}.*`;
  let s = `SELECT ${selList}\n${fromJoins(o.table, a, o.joins)}`;
  const where = kept.map((c) => renderCond(`${c.alias ?? a}.${q(c.col)}`, c));
  if (where.length) s += `\nWHERE ${where.join("\n  AND ")}`;
  if (o.orderBy) s += `\nORDER BY ${a}.${q(o.orderBy)} ${o.orderDir}`;
  if (o.limit > 0) s += `\nLIMIT ${Math.floor(o.limit)}`;
  return s;
}
