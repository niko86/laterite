// Generate the registry data + the 92 typed-graph classes from the AGS5
// dictionary — the TS analog of `tools/generate_pyi.py`. Single source of truth:
// `rust-packages/laterite-ags4-core/data/ags5_dictionary.json`. Run after a dictionary
// edit; `test/p3-typed-graph.test.ts` is the CI drift guard (byte-equality).
//
//   node tools/generate-typed-graph.mjs        # rewrite the generated files
//
// Exports `generateRegistry` / `generateTypedGraph` (pure, dict → string) so the
// drift test can regenerate in-memory and compare.
import { readFileSync, writeFileSync } from "node:fs";
import { fileURLToPath } from "node:url";
import { dirname, join } from "node:path";

const HERE = dirname(fileURLToPath(import.meta.url));
const DICT_PATH = join(HERE, "..", "..", "laterite-ags4-core", "data", "ags5_dictionary.json");
const REGISTRY_OUT = join(HERE, "..", "ts", "registry.generated.ts");
const TYPED_OUT = join(HERE, "..", "ts", "typed-graph.generated.ts");

const HEADER =
  "// AUTO-GENERATED from ags5_dictionary.json by tools/generate-typed-graph.mjs.\n" +
  "// DO NOT EDIT — re-run the generator after a dictionary change.\n\n";

// Port of `generate_pyi.py::_STRING_TYPES` / `_py_type`, to TS.
const STRING_TYPES = new Set(["ID", "X", "PA", "PT", "PU", "T", "U", "DMS", "MC", "XN"]);

/** AGS spec type code → the TS type a typed-graph field carries. */
export function tsType(agsType) {
  const t = (agsType || "").trim().toUpperCase();
  if (STRING_TYPES.has(t)) return "string";
  if (t === "0DP") return "number";
  if (t === "DT") return "Date";
  if (t === "YN") return "boolean";
  if (t === "RL") return "number";
  for (const suffix of ["DP", "SF", "SCI"]) {
    if (t.endsWith(suffix)) {
      const prefix = t.slice(0, -suffix.length);
      if (prefix && /^\d+$/.test(prefix)) return "number";
    }
  }
  return "string";
}

/** parent code → its direct child codes, alphabetically (stable output). */
function childrenOf(groups) {
  const byParent = new Map();
  for (const g of groups) {
    if (g.parent) {
      if (!byParent.has(g.parent)) byParent.set(g.parent, []);
      byParent.get(g.parent).push(g.code);
    }
  }
  for (const list of byParent.values()) list.sort();
  return byParent;
}

/** The registry data module — the GROUPS metadata as a typed literal. */
export function generateRegistry(groups) {
  const data = groups.map((g) => ({
    code: g.code,
    contents: g.contents,
    parent: g.parent ?? null,
    isHighVolume: Boolean(g.is_high_volume),
    headings: g.headings.map((h) => ({
      name: h.name,
      status: h.status,
      type: h.type,
      unit: h.unit ?? null,
      description: h.description ?? "",
    })),
  }));
  return (
    HEADER +
    'export type HeadingStatus = "KEY" | "REQUIRED" | "OTHER";\n\n' +
    "export interface GeneratedHeading {\n" +
    "  readonly name: string;\n" +
    "  readonly status: HeadingStatus;\n" +
    "  readonly type: string;\n" +
    "  readonly unit: string | null;\n" +
    "  readonly description: string;\n" +
    "}\n\n" +
    "export interface GeneratedGroup {\n" +
    "  readonly code: string;\n" +
    "  readonly contents: string;\n" +
    "  readonly parent: string | null;\n" +
    "  readonly isHighVolume: boolean;\n" +
    "  readonly headings: readonly GeneratedHeading[];\n" +
    "}\n\n" +
    `export const GROUPS_DATA: readonly GeneratedGroup[] = ${JSON.stringify(data, null, 2)};\n`
  );
}

/** The 92 typed-graph classes — scalar heading fields + child arrays. */
export function generateTypedGraph(groups) {
  const children = childrenOf(groups);
  const ordered = [...groups].sort((a, b) => a.code.localeCompare(b.code));
  const blocks = ordered.map((g) => emitClass(g, children.get(g.code) ?? []));
  return (
    HEADER +
    "/* eslint-disable */\n" +
    "// A typed builder graph: `new PROJ({ PROJ_ID: 'P1', locas: [new LOCA({…})] })`,\n" +
    "// then `emitAgs4(proj)` walks it into per-group rows. Each class carries a\n" +
    "// static `code` and extends AgsGroup; child arrays are\n" +
    "// `<childCode>`.toLowerCase() + 's'.\n" +
    'import { AgsGroup } from "./ags-group";\n\n' +
    blocks.join("\n\n") +
    "\n"
  );
}

function emitClass(g, childCodes) {
  const lines = [
    `export class ${g.code} extends AgsGroup {`,
    `  static readonly code = ${JSON.stringify(g.code)};`,
  ];
  for (const h of g.headings) lines.push(`  ${h.name}: ${tsType(h.type)} | null = null;`);
  for (const c of childCodes) lines.push(`  ${c.toLowerCase()}s: ${c}[] = [];`);
  lines.push(`  constructor(init: Partial<${g.code}> = {}) {`);
  lines.push(`    super();`);
  lines.push(`    Object.assign(this, init);`);
  lines.push(`  }`);
  lines.push(`}`);
  return lines.join("\n");
}

/** The 92 group descriptors (the dictionary JSON is `{format_version,
 * ags_edition, groups}`). */
export function loadDictionary() {
  return JSON.parse(readFileSync(DICT_PATH, "utf8")).groups;
}

// CLI: rewrite the generated files.
if (process.argv[1] && fileURLToPath(import.meta.url) === process.argv[1]) {
  const groups = loadDictionary();
  writeFileSync(REGISTRY_OUT, generateRegistry(groups));
  writeFileSync(TYPED_OUT, generateTypedGraph(groups));
  console.log(`generated ${groups.length} groups → registry.generated.ts + typed-graph.generated.ts`);
}
