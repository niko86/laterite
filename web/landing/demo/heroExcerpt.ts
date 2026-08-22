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

/** The build-time render: the committed fixture's own opening lines, and the
 *  card's content until the engine wakes. The drift gate compares THESE
 *  against the file on disk — which is why this module stays store-free: the
 *  gate imports it under plain vitest, where the store's Solid graph cannot
 *  initialise. The live half (heroLines) lives with the Hero component. */
export const HERO_LINES: readonly string[] = seeded
  .split(/\r?\n/)
  .slice(0, HERO_LINE_COUNT);
