/* Which editor a reader gets is a MODALITY question, not a width one (#525):
 * a phone in landscape is still a touch device, and a narrow desktop window
 * still has a mouse. So the split reads `pointer: coarse`, live — plugging a
 * tablet into a keyboard-and-mouse dock flips it without a reload.
 *
 * Module scope with no owner, like the store's graph: this signal IS the
 * page's lifetime, and a media-query listener holds no DOM to leak.
 */

import { createSignal } from "solid-js";

const query = window.matchMedia("(pointer: coarse)");
const [coarse, setCoarse] = createSignal(query.matches);
query.addEventListener("change", (e) => setCoarse(e.matches));

/** True on touch-first devices — the row carousel's audience. */
export const coarsePointer = coarse;
