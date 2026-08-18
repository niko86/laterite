/* The page's vertical rhythm, as data (#395, read by #399's rail).
 *
 * The page is a sequence of sections and each one owns a band of the strata
 * ramp. That is not decoration: the borehole rail turns scrolling into
 * descending a borehole, and it reads THIS list to know how many bands there
 * are, which section each belongs to, and what to label its depth tick. A rail
 * with its own copy of the sequence is a rail that goes out of step with the
 * page the first time a section is added.
 *
 * Seven sections, seven bands, `--laterite-300` through `--laterite-900` — the
 * ramp's own steps, in order, so band N is simply `300 + N * 100`. The four
 * group sections land on 400/500/600/700, which is exactly the group keying
 * #396 specifies (PROJ #db7841, LOCA #ce5640, SAMP #be3b2e, LLPL #9b3932). The
 * two sets agreeing is the point: a group's colour IS its depth.
 */

export type Section = {
  /** The DOM id, and the fragment a nav link targets. */
  readonly id: string;
  /** The rail's depth-scale label. Group sections use the group code. */
  readonly label: string;
  /** The AGS4 group this section teaches, where it teaches one. */
  readonly group?: "PROJ" | "LOCA" | "SAMP" | "LLPL";
};

export const SECTIONS: readonly Section[] = [
  { id: "top", label: "Surface" },
  { id: "proj", label: "PROJ", group: "PROJ" },
  { id: "loca", label: "LOCA", group: "LOCA" },
  { id: "samp", label: "SAMP", group: "SAMP" },
  { id: "llpl", label: "LLPL", group: "LLPL" },
  { id: "file", label: "File" },
  { id: "install", label: "Install" },
];

/** The band CSS variable for section `index` — the ramp step, not a copy of it.
 *  Callers pass this to `var()`, so a ramp retune (the dark shift in #400) moves
 *  every band without touching this module. */
export function bandVar(index: number): string {
  return `--laterite-${300 + index * 100}`;
}

/** The band a group owns, by its position in the chain. Returns "" for a code
 *  the page does not draw, so a caller cannot silently colour something with
 *  band identity it has no right to. */
export function groupBandVar(code: string): string {
  const at = SECTIONS.findIndex((s) => s.group === code);
  return at === -1 ? "" : bandVar(at);
}
