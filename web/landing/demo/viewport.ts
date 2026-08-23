/* Which PRESENTATION the findings get is a width question, not a modality one
 * (#592) — the inverse of pointer.ts's split. A stack of callouts costs
 * vertical space, and a narrow window pays that cost whether its pointer is a
 * finger or a mouse. 64rem is the page's own layout breakpoint: the pairing
 * grids flip to two columns there (FileAndFindings, GroupSection), so the
 * findings change dress exactly where the page changes shape.
 *
 * Module scope with no owner, like pointer.ts: this signal IS the page's
 * lifetime, and a media-query listener holds no DOM to leak.
 */

import { createSignal } from "solid-js";

const query = window.matchMedia("(min-width: 64rem)");
const [wide, setWide] = createSignal(query.matches);
query.addEventListener("change", (e) => setWide(e.matches));

/** True below the page's layout breakpoint — the finding carousels' audience. */
export const narrowViewport = () => !wide();
