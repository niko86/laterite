/* The hero card's lines (#531): the seed itself, never a picture of it.
 *
 * The previous card was a hand-written fragment, and it did exactly what
 * hand-written copies of a live file do — drifted, until it showed a
 * corrected decimal where the fixture deliberately carries the teaching
 * defect. Slicing the committed fixture is what makes that impossible: these
 * are the same bytes the engine validates, and a vitest drift gate compares
 * them against the file on disk.
 */

import seeded from "./seeded-delivery.ags?raw";

/** How much of the file the card shows: the PROJ block, exactly — the four
 *  declaration lines plus its one DATA row, the stanza that opens every
 *  AGS4 delivery. */
export const HERO_LINE_COUNT = 5;

export const HERO_LINES: readonly string[] = seeded
  .split(/\r?\n/)
  .slice(0, HERO_LINE_COUNT);
