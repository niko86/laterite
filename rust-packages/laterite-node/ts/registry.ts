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
  registryAncestorChain,
  registryDictionaryJson,
  registryInheritedKeyNames,
} from "./native";
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

/** One heading in a {@link DictionarySnapshot} group (`type` is the AGS data type). */
export interface DictHeading {
  name: string;
  status: string;
  type: string;
  unit?: string;
  description: string;
}
/** One group in a {@link DictionarySnapshot}. */
export interface DictGroup {
  code: string;
  contents: string;
  parent?: string;
  headings: DictHeading[];
}
/** The bundled standard dictionary for one edition. */
export interface DictionarySnapshot {
  ags_edition: string;
  groups: DictGroup[];
}

/**
 * The bundled STANDARD dictionary for one AGS `edition` — the per-edition view
 * of the official dictionary (canonical group + heading names, descriptions,
 * UNIT/TYPE, status). Where {@link GROUPS} is the *union* registry across all
 * editions (the default), this is a single edition's standard dictionary — the
 * same content the browser and `laterite.registry.dictionary()` render, from
 * one shared Rust builder (#294 F#6).
 *
 * @param edition `"4.0.3" | "4.0.4" | "4.1" | "4.1.1" | "4.2"`; omit (or
 *   `"auto"`) for the fallback edition.
 * @throws {Error} if `edition` is not a recognised edition.
 */
export function dictionary(edition?: string): DictionarySnapshot {
  return JSON.parse(registryDictionaryJson(edition)) as DictionarySnapshot;
}

/** Every direct child group of `parentCode`, alphabetically. */
export function childGroups(parentCode: string): GroupDescriptor[] {
  return Object.values(GROUPS)
    .filter((g) => g.parent === parentCode)
    .sort((a, b) => a.code.localeCompare(b.code));
}

/** Parent chain from `code` to root: `[code, parent, …, root]`. Throws for an
 * unknown code (so root groups — `[code]` — are distinguishable).
 *
 * Delegates to the native `laterite_ags4_core::registry::ancestor_chain` — the
 * ONE Rust definition of the group tree, the same walk the Python wheel binds —
 * rather than re-walking `.parent` pointers in TS (#532). The native error for an
 * unknown code is re-typed to {@link Ags4Error} to keep this facade's contract. */
export function ancestorChain(code: string): string[] {
  try {
    return registryAncestorChain(code);
  } catch (e) {
    throw new Ags4Error(e instanceof Error ? e.message : String(e));
  }
}

/** KEY heading names a group inherits from its **direct parent** — the
 * intersection of this group's KEY headings with its immediate parent's (NOT the
 * whole ancestor chain: AGS re-declares inherited keys at every level, so the
 * direct-parent intersection already captures every key carried from above).
 *
 * Delegates to the native `laterite_ags4_core::registry::inherited_key_names`
 * (the same leaf the Python wheel binds) rather than re-implementing the
 * KEY-intersection in TS (#532); native returns the names sorted, wrapped here in
 * a Set. The native error for an unknown code is re-typed to {@link Ags4Error}.
 *
 * @param code The group whose inherited KEY names to gather.
 * @returns The set of KEY heading names shared with the direct parent (empty for a root).
 * @throws {Ags4Error} If `code` isn't in the registry. */
export function inheritedKeyNames(code: string): Set<string> {
  try {
    return new Set(registryInheritedKeyNames(code));
  } catch (e) {
    throw new Ags4Error(e instanceof Error ? e.message : String(e));
  }
}
