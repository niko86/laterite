// Pure SQL-string composition for the Explore builders (ChartBuilder +
// SqlBuilder). Kept out of the components so the quoting / aggregate / WHERE /
// JOIN logic — the part most prone to subtle regressions — is unit-tested.

/** Quote a SQL identifier (table/column), doubling internal quotes so a
 *  heading with a `"` can't break out. */
export const q = (id: string) => `"${id.replace(/"/g, '""')}"`;

/** A single-quoted string literal, internal quotes doubled. The one place a
 *  user value is escaped into SQL; `lit` is its number-aware caller. */
const textLit = (v: string) => `'${v.replace(/'/g, "''")}'`;

/** A WHERE value literal: a bare number stays unquoted (numeric comparison);
 *  anything else becomes a single-quoted string literal (quotes escaped). */
export const lit = (v: string) =>
  v.trim() !== "" && !Number.isNaN(Number(v)) ? v.trim() : textLit(v);

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

/** What the plot query and the probe that ranks for it must agree on: the same
 *  table, the same joins and aliasing, and the same X/Y completeness. */
interface ChartQueryBase {
  table: string;
  /** base alias (join mode). */
  alias?: string;
  joins?: JoinSpec[];
  x: string | QualifiedCol;
  y: string | QualifiedCol;
  chartType: ChartType;
  agg: Agg;
}

/** The tail fold, on the aggregating bar path: which colour values keep a group
 *  of their own, and the label everything else collapses onto. */
export interface ChartFold {
  /** The survivors, as `chartRankSql` returned them and as the series
   *  assembler will match them — text, not values. */
  keep: readonly string[];
  /** `foldLabel`'s answer for exactly that list. The query MATERIALISES this
   *  as data, so it has to be the string the legend will show, not a second
   *  guess at it — see `chartSeries.foldLabel`. */
  label: string;
}

export interface ChartSqlOpts extends ChartQueryBase {
  colour?: string | QualifiedCol;
  rowCap: number;
  /** Required on the aggregating path with a colour-by, ignored everywhere
   *  else. Without it that path composes NO query — see `chartSql`. */
  fold?: ChartFold;
}

export interface ChartRankOpts extends ChartQueryBase {
  /** Required here, unlike the plot query: there is nothing to rank without it. */
  colour: string | QualifiedCol;
  /** How many values may keep a colour. Nothing needs to know whether a tail
   *  exists — a value the probe did not return folds by not being in the list,
   *  which is the same thing that happens to one ranked past the cap. */
  cap: number;
}

/** The colour value AS TEXT — DuckDB's own rendering, NULL read as the empty
 *  string, which is `scalarText`'s two rules moved into SQL.
 *
 *  Aggregating with a colour-by is the one path where the query has to NAME the
 *  fold, and a name is a string. `CASE WHEN … THEN <a DOUBLE> ELSE 'Other' END`
 *  does not widen to VARCHAR in DuckDB: it resolves to DOUBLE and fails the
 *  whole query converting 'Other'. So the colour becomes text BEFORE the fold
 *  is applied — and the probe renders it the same way, because the assembler
 *  matches the two by string and DuckDB writes a DOUBLE as '1.0' where JS
 *  writes '1'. Rendering them differently would leave no survivor matching its
 *  own rows: every series in the neutral, under a legend naming none of them. */
const colourText = (cr: string) => `COALESCE(CAST(${cr} AS VARCHAR), '')`;

/** The colour expression an aggregating plot query groups by: the value as
 *  text, with everything the probe did not rank collapsed onto one label. */
function foldedColour(cr: string, fold: ChartFold): string {
  const ce = colourText(cr);
  // No survivors is written as a predicate that cannot hold, NOT as a colour
  // expression without the fold. SQL has no empty `IN ()` — it is a syntax
  // error — and dropping the CASE instead would compose a differently SHAPED
  // query off an empty list: every distinct value keeping its own group, which
  // is the behaviour the fold exists to prevent. One shape, whatever the probe
  // answered, so an empty list can only ever mean "everything folds".
  const holds =
    fold.keep.length === 0
      ? "FALSE"
      : `${ce} IN (${fold.keep.map(textLit).join(", ")})`;
  return `CASE WHEN ${holds} THEN ${ce} ELSE ${textLit(fold.label)} END`;
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

/** The POPULATION a chart query runs over: the FROM (with its joins and their
 *  range predicates) and the WHERE that decides which rows are plotted at all,
 *  plus the alias-aware column resolver both clauses were built with.
 *
 *  Shared with `chartRankSql` deliberately, not as a tidy-up. A ranking
 *  composed over a different population than the plot is wrong in a way
 *  nothing downstream can see: the fold would call a value "Other" on the
 *  strength of rows the chart never draws, and the legend would be making a
 *  claim about the delivery that the query behind it does not support.
 *
 *  Returns null when the selection is incomplete (no table/X, or no Y unless
 *  counting) — the one place both queries decide that. */
function chartScope(o: ChartQueryBase): {
  from: string;
  where: string;
  ref: (c: string | QualifiedCol) => string;
  /** Handed back rather than re-derived by the caller: the WHERE above and the
   *  SELECT list `chartSql` builds branch on the same two facts, and deriving
   *  them twice is how a filter and an aggregate come to disagree. */
  counting: boolean;
  aggregating: boolean;
} | null {
  const counting = o.agg === "count";
  if (!o.table || !o.x) return null;
  if (!counting && !o.y) return null;

  const joins = o.joins ?? [];
  const joined = joins.length > 0;
  const a = o.alias ?? "t0";
  const ref = (c: string | QualifiedCol) => colRef(c, a, joined);
  const yr = o.y ? ref(o.y) : "";
  const aggregating = o.chartType === "bar" && o.agg !== "none";
  return {
    from: joined ? fromJoins(o.table, a, joins) : `FROM ${q(o.table)}`,
    // An aggregate has already collapsed X, so only Y's nulls matter (and a
    // COUNT counts rows, so none do); a raw plot needs both coordinates.
    where: aggregating
      ? counting
        ? ""
        : ` WHERE ${yr} IS NOT NULL`
      : ` WHERE ${ref(o.x)} IS NOT NULL AND ${yr} IS NOT NULL`,
    ref,
    counting,
    aggregating,
  };
}

/** Compose the chart query. Scatter/line select raw X/Y (line is ordered by X);
 *  bar with an aggregate GROUP BYs the X category (+ the folded colour). With
 *  `joins`, X/Y/colour are alias-qualified and the JOINs are emitted; the output
 *  aliases (x/y/c) are unchanged so the ECharts mapping is untouched.
 *
 *  Returns "" when there is no query to compose: an incomplete selection (no
 *  table/X, or no Y unless counting), or — on the aggregating path with a
 *  colour-by — no `fold`.
 *
 *  That second case is the two-phase dependency, held here rather than in the
 *  component (#457). On that path the colour is a GROUP KEY, so the tail has to
 *  fold inside the GROUP BY or not at all, and what survives is the probe's
 *  answer. Composing without it would emit a `c` the assembler cannot match
 *  against the ranking — every series painted neutral under a legend naming
 *  none of them, a chart that looks drawn and is wrong. "" is the loud answer,
 *  and it makes the wait the composer's rule rather than a caller's discipline. */
export function chartSql(o: ChartSqlOpts): string {
  const scope = chartScope(o);
  if (!scope) return "";
  const { from, where, ref, counting, aggregating } = scope;
  const { x, y, colour, chartType, agg, rowCap, fold } = o;
  const xr = ref(x);
  const yr = y ? ref(y) : "";
  const cr = colour ? ref(colour) : "";
  if (aggregating) {
    if (cr && !fold) return "";
    const ce = cr && fold ? foldedColour(cr, fold) : "";
    const yExpr = counting ? "COUNT(*)" : `${agg.toUpperCase()}(${yr})`;
    return (
      `SELECT ${xr} AS x, ${yExpr} AS y${ce ? `, ${ce} AS c` : ""} ${from}${where}` +
      ` GROUP BY ${ce ? `${xr}, ${ce}` : xr} ORDER BY x LIMIT ${rowCap}`
    );
  }
  const selC = cr ? `, ${cr} AS c` : "";
  const order = chartType === "line" ? ` ORDER BY ${xr}` : "";
  return (
    `SELECT ${xr} AS x, ${yr} AS y${selC} ${from}${where}${order}` +
    ` LIMIT ${rowCap}`
  );
}

/** Compose the CARDINALITY PROBE that decides which colour-by values keep a
 *  palette slot: every distinct value with its row count, most rows first.
 *
 *  It ranks over the WHOLE table rather than over the plotted rows, and that is
 *  the point of it. The scatter/line plot query is a bare row `LIMIT` with no
 *  `ORDER BY`, so the values it happens to return are an arbitrary slice of the
 *  delivery — folding on their first-appearance order would put a genuinely
 *  common value into "Other" because it sorted late on disk.
 *
 *  Ties break on the value, so the same data always assigns the same colours. */
export function chartRankSql(o: ChartRankOpts): string {
  const scope = chartScope(o);
  if (!scope) return "";
  // The aggregating path renders the colour as text so its plot query can name
  // the fold, and this ranking has to be a ranking of exactly the values that
  // query emits — the assembler matches the two by string. See `colourText`.
  const cr = scope.aggregating
    ? colourText(scope.ref(o.colour))
    : scope.ref(o.colour);
  return (
    `SELECT ${cr} AS c, COUNT(*) AS n ${scope.from}${scope.where}` +
    ` GROUP BY ${cr} ORDER BY n DESC, c ASC LIMIT ${o.cap}`
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
