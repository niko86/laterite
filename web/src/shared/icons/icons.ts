// The vendored Lucide set — the manifest that decides what we carry (#406).
//
// This file IS the set: `scripts/sync-icons.mjs` reads these import specifiers,
// refreshes each .svg from `lucide-static`, and fails on an icon that upstream
// no longer ships or that is vendored but no longer named here. So adding an
// icon is one line plus `npm run sync-icons`, and removing one cannot leave a
// stale file behind.
//
// `?raw` rather than `?url`: the markup is inlined into the bundle, so an icon
// costs no request and cannot 404 offline — which is the whole reason these are
// vendored rather than fetched from the CDN the design system's own Icon uses.
//
// The starting set is the working set the design system names by name. It is
// deliberately not "all of Lucide": upstream ships around two thousand icons and
// this is a PWA that precaches its shell. Extend it as screens need icons, not
// in advance.
//
// NEVER hand-draw a replacement. If Lucide has no match for what you need, say
// so — that is the system's rule, and an invented glyph is the one thing that
// makes an icon set stop reading as one.

import download from "./download.svg?raw";
import fileDown from "./file-down.svg?raw";
import funnel from "./funnel.svg?raw";
import gitCompareArrows from "./git-compare-arrows.svg?raw";
import gripVertical from "./grip-vertical.svg?raw";
import history from "./history.svg?raw";
import keyRound from "./key-round.svg?raw";
import redo2 from "./redo-2.svg?raw";
import search from "./search.svg?raw";
import shieldCheck from "./shield-check.svg?raw";
import trash2 from "./trash-2.svg?raw";
import triangleAlert from "./triangle-alert.svg?raw";
import undo2 from "./undo-2.svg?raw";
import x from "./x.svg?raw";

import { iconBody } from "./iconBody";

/** Icon name → the icon's inner markup, wrapper stripped (see `iconBody`). */
export const ICONS = {
  download: iconBody(download),
  "file-down": iconBody(fileDown),
  funnel: iconBody(funnel),
  "git-compare-arrows": iconBody(gitCompareArrows),
  "grip-vertical": iconBody(gripVertical),
  history: iconBody(history),
  "key-round": iconBody(keyRound),
  "redo-2": iconBody(redo2),
  search: iconBody(search),
  "shield-check": iconBody(shieldCheck),
  "trash-2": iconBody(trash2),
  "triangle-alert": iconBody(triangleAlert),
  "undo-2": iconBody(undo2),
  x: iconBody(x),
} as const;

/**
 * The names an `<Icon>` will accept.
 *
 * Derived from the manifest rather than written out, so a typo — or an icon
 * someone hoped existed — is a type error at the call site rather than an empty
 * square at runtime.
 */
export type IconName = keyof typeof ICONS;
