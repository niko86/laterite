// How much of the chart-categorical vocabulary a chart form may spend (#445).
//
// These two numbers are facts about the palette in charts.css beside this file,
// discovered by the separation gate: how many numbered slots there are, and how
// far into the sequence the ALL-PAIRS pairlist is validated. They live in a
// module rather than in the gate because the BUILDER has to cap its series
// count at exactly what the gate checked, and neither side can import the
// other — a shipped component may not import a test file, and the gate is what
// decides what these mean. One definition, two readers.
//
// They were a comment before this, at the top of the chart-token reader, and a
// comment is not a place a limit survives: the builder handed the whole palette
// to every form, so the default scatter was already drawing pairs the gate had
// never checked for it, and past the last slot ECharts cycles rather than
// failing — two series, one colour, a legend saying they differ.

/** Numbered slots in the palette. The ceiling for bar and line, which are
 *  validated adjacent-only because only touching marks have to separate. */
export const SLOT_COUNT = 5;

/** Slots validated on the ALL-PAIRS pairlist — the ceiling for scatter, bubble
 *  and map, where any two marks can land side by side, which is the strictly
 *  harder test. */
export const ALL_PAIRS_CAP = 3;
