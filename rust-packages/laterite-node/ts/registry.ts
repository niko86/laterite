/**
 * laterite.registry — the read-only AGS group registry, the Node port of
 * laterite-py's `registry`. Generated from the SAME dictionary JSON as Python's
 * (see `registry.generated.ts`, drift-tested), so the metadata here can't drift
 * from the authoritative source.
 *
 * @module
 */
import { Ags4Error } from "./errors";
import {
  GROUPS_DATA,
  type GeneratedGroup,
  type GeneratedHeading,
  type HeadingStatus,
} from "./registry.generated";

/** A heading's status as it appears in the union dictionary. Beyond the base
 * `KEY`/`REQUIRED`/`OTHER` it carries the combined marker `KEY+REQUIRED` and a
 * standalone `DEPRECATED` value, which is why key-ness is decided by
 * {@link isKeyStatus} rather than a bare `=== "KEY"`. */
export type { HeadingStatus };
/** One heading's descriptor: `{name, status, type, unit, description}`. */
export type Heading = GeneratedHeading;

/** A heading is a KEY if any `+`-separated status part is KEY — the union
 * dictionary carries combined statuses like `KEY+REQUIRED`, so a bare
 * `status === "KEY"` would wrongly miss them (matches the Rust/Python check). */
export function isKeyStatus(status: string): boolean {
  return status.split("+").some((p) => p.trim().toUpperCase() === "KEY");
}

/** A typed, immutable view onto one standard AGS group: its 4-letter `code`,
 * human `contents` description, `parent` code (or `null` for a root group), and
 * its ordered `headings`. Wraps a single entry from the generated dictionary so
 * the registry never hands out mutable raw rows. */
export class GroupDescriptor {
  readonly code: string;
  readonly contents: string;
  readonly parent: string | null;
  readonly headings: readonly Heading[];

  constructor(g: GeneratedGroup) {
    this.code = g.code;
    this.contents = g.contents;
    this.parent = g.parent;
    this.headings = g.headings;
  }

  /** DuckDB table name (`g_<code>`) — for parity with the Python descriptor. */
  get table(): string {
    return `g_${this.code.toLowerCase()}`;
  }
  /** DuckDB view name (`v_<code>`). */
  get view(): string {
    return `v_${this.code.toLowerCase()}`;
  }
  /** This group's KEY headings (status part-matched via {@link isKeyStatus}), in
   * declaration order. */
  get keyHeadings(): readonly Heading[] {
    return this.headings.filter((h) => isKeyStatus(h.status));
  }
  /** This group's non-KEY headings — the complement of {@link keyHeadings} — in
   * declaration order. */
  get nonKeyHeadings(): readonly Heading[] {
    return this.headings.filter((h) => !isKeyStatus(h.status));
  }
}

/** Every standard AGS group, keyed by 4-letter code. */
export const GROUPS: Readonly<Record<string, GroupDescriptor>> = Object.freeze(
  Object.fromEntries(GROUPS_DATA.map((g) => [g.code, new GroupDescriptor(g)])),
);

/** Single-group lookup; `undefined` for unknown codes. */
export function get(code: string): GroupDescriptor | undefined {
  return GROUPS[code];
}

/** Every direct child group of `parentCode`, alphabetically. */
export function childGroups(parentCode: string): GroupDescriptor[] {
  return Object.values(GROUPS)
    .filter((g) => g.parent === parentCode)
    .sort((a, b) => a.code.localeCompare(b.code));
}

/** Parent chain from `code` to root: `[code, parent, …, root]`. Throws for an
 * unknown code (so root groups — `[code]` — are distinguishable). */
export function ancestorChain(code: string): string[] {
  if (GROUPS[code] === undefined) {
    throw new Ags4Error(`unknown group code: ${JSON.stringify(code)}`);
  }
  const chain: string[] = [];
  let current: string | null = code;
  while (current !== null) {
    chain.push(current);
    current = GROUPS[current]?.parent ?? null;
  }
  return chain;
}

/** KEY heading names a group inherits from its ancestors (its own KEYs are not
 * included — the chain is walked from the parent up).
 *
 * @param code The group whose inherited KEY names to gather.
 * @returns The set of KEY heading names contributed by every ancestor.
 * @throws {Ags4Error} If `code` isn't in the registry (via {@link ancestorChain}). */
export function inheritedKeyNames(code: string): Set<string> {
  const names = new Set<string>();
  for (const ancestor of ancestorChain(code).slice(1)) {
    for (const h of GROUPS[ancestor]!.keyHeadings) names.add(h.name);
  }
  return names;
}
