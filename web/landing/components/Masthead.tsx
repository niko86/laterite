/* The landing page's masthead: the shared chrome (MastheadShell, where the
 * design notes live) with the demo's verdict docked in it.
 *
 * The verdict rides the masthead (#691) rather than floating over the page. It
 * renders nothing until the engine's first report lands and a table is on
 * screen, so the header is unchanged everywhere else. It is passed in rather
 * than imported by the shell so that a page without the demo can wear the same
 * chrome without loading the demo's store.
 */

import type { Component } from "solid-js";
import { DockedScoreboard } from "../demo/Scoreboard";
import { MastheadShell } from "./MastheadShell";

export const Masthead: Component = () => (
  <MastheadShell verdict={<DockedScoreboard />} />
);
