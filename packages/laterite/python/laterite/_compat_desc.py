"""python-ags4 desc-string translation for ``laterite.compat.check_file``.

Laterite's validator writes its own ``Finding.desc`` strings — more
precise wording, e.g. ``GROUP name must be exactly 4 uppercase letters
(A–Z)`` instead of python-ags4's vaguer ``GROUP name should consist of
four uppercase letters.``. The native API and Rust CLI return those
laterite phrasings; only ``compat.check_file`` translates back to the
python-ags4 wording, because that surface exists explicitly to be a
byte-faithful drop-in.

The translator is a small (rule, regex, replacement) table. The first
pattern that matches a finding's desc wins; on no match the laterite
wording is left alone (better to leak our wording than lie about a
rule). ``replacement`` may contain ``\\1`` … ``\\g<name>`` regex
back-refs so we can lift entity names (heading, value, group) out of
the laterite phrasing and stitch them into python-ags4's template.

Anchored at python-ags4 1.2.0; pinned in dev deps.
"""

from __future__ import annotations

import re

# Each entry: (rule_key, compiled_regex, replacement_template, extra_fn).
# `extra_fn` (optional) post-processes the substituted string when a
# plain regex sub isn't enough (e.g. case fixes on captured tokens).
_Rules = list[tuple[str, "re.Pattern[str]", str]]

_TABLE: _Rules = [
    # ---- Rule 1 — non-ASCII line --------------------------------------
    # Laterite gives a precise diagnosis ("code point > 255, or a BOM");
    # python-ags4 says only "Has Non-ASCII character(s) (assuming…cp1252)".
    ("AGS Format Rule 1",
     re.compile(r"^Line contains non-ASCII character\(s\).*$"),
     "Has Non-ASCII character(s) (assuming that file encoding is 'cp1252')."),

    # ---- Rule 2 — empty group ----------------------------------------
    ("AGS Format Rule 2",
     re.compile(r"^Group has no DATA rows.*$"),
     "No DATA rows in group."),

    # ---- Rule 2b — UNIT/TYPE row placement / presence ----------------
    # "UNIT row missing from group." and "TYPE row missing from group."
    # already match python-ags4's wording verbatim — no entry needed.
    ("AGS Format Rule 2b",
     re.compile(r"^UNIT row is misplaced.*HEADING row\.?$"),
     "UNIT row is misplaced. It should be immediately below the HEADING row."),
    ("AGS Format Rule 2b",
     re.compile(r"^TYPE row is misplaced.*UNIT row\.?$"),
     "TYPE row is misplaced. It should be immediately below the UNIT row."),

    # ---- Rule 3 — bad descriptor -------------------------------------
    ("AGS Format Rule 3",
     re.compile(r"^Row does not start with a valid data descriptor.*$"),
     "Does not start with a valid data descriptor."),

    # ---- Rule 4 — field count mismatch -------------------------------
    ("AGS Format Rule 4",
     re.compile(r"^DATA row field count does not match the HEADING row\.?$"),
     "Number of fields does not match the HEADING row."),

    # ---- Rule 5 — quoting issues -------------------------------------
    # Stage 7g: laterite now distinguishes the two python-ags4
    # sub-cases via `QuotingDeviation::EmbeddedQuote` vs
    # `QuotingDeviation::NotEnclosed`; map each to python-ags4's
    # matching desc. (Pre-7g we collapsed both into one translator
    # entry — see git history for the legacy phrasing.)
    ("AGS Format Rule 5",
     re.compile(r"^Row has an embedded double-quote.*$"),
     "Contains quotes within a data field. All such quotes should be "
     "enclosed by a second quote."),
    ("AGS Format Rule 5",
     re.compile(r"^Row has field\(s\) not enclosed in double quotes\.?$"),
     "Contains fields that are not enclosed in double quotes."),

    # ---- Rule 7 — heading order / duplicates -------------------------
    ("AGS Format Rule 7",
     re.compile(r"^HEADING row contains duplicate field names\.?$"),
     "HEADER row has duplicate fields."),
    # Headings out of order. Laterite: 'Headings out of order from "X". Expected dictionary order from here: A|B'
    # python-ags4: 'Headings not in order starting from X. Expected order: ...A|B'
    ("AGS Format Rule 7",
     re.compile(r'^Headings out of order from "(?P<head>[^"]+)"\. '
                r"Expected dictionary order from here: (?P<order>.+)$"),
     r"Headings not in order starting from \g<head>. Expected order: ...\g<order>"),
    # Already-aligned wording — kept as no-op to document intent.
    ("AGS Format Rule 7",
     re.compile(r"^Heading order cannot be checked.*$"),
     "Heading order cannot be checked: one or more headings are not in "
     "the standard dictionary or DICT group (see Rule 9)."),

    # ---- Rule 8 — DT (date/time) type — distinct python-ags4 phrasing.
    # Laterite: 'Value "2023-11-16T12:00" in LOCA_STAR does not match its declared TYPE "DT" / UNIT "yyyy-mm-dd".'
    # python-ags4: 'Value 2023-11-16T12:00 in LOCA_STAR does not match the specified format (yyyy-mm-dd) or is an invalid date/time.'
    # Listed *before* the generic Rule 8 catcher because first-match-wins.
    ("AGS Format Rule 8",
     re.compile(r'^Value "(?P<val>[^"]*)" in (?P<head>[A-Z0-9]{4}_[A-Z0-9]+) '
                r'does not match its declared TYPE "DT" / UNIT "(?P<unit>[^"]+)"\.$'),
     r"Value \g<val> in \g<head> does not match the specified format "
     r"(\g<unit>) or is an invalid date/time."),

    # ---- Rule 8 — T (elapsed time) type — also distinct.
    # Laterite: 'Value "1:00" in CHIS_TIME does not match its declared TYPE "T" / UNIT "hh:mm".'
    # python-ags4: 'Value 1:00 in CHIS_TIME not in the specified elapsed time format (hh:mm) or is an invalid elapsed time.'
    ("AGS Format Rule 8",
     re.compile(r'^Value "(?P<val>[^"]*)" in (?P<head>[A-Z0-9]{4}_[A-Z0-9]+) '
                r'does not match its declared TYPE "T" / UNIT "(?P<unit>[^"]+)"\.$'),
     r"Value \g<val> in \g<head> not in the specified elapsed time "
     r"format (\g<unit>) or is an invalid elapsed time."),

    # ---- Rule 8 — typed-value mismatch -------------------------------
    # Laterite: 'Value "523145.010" in LOCA_NATE does not match its declared TYPE "2DP".'
    # python-ags4: 'Value 523145.010 in LOCA_NATE not of data type 2DP.'
    # The validator may also append " (Expected: <nsf-form>)" for SF
    # failures (`rules/typed_values.rs` lifts the rounded reference);
    # we preserve that suffix verbatim via the `(?P<expected>...)` capture.
    # Type-specific suffixes added by `_rule_8_suffix` (DMS: "or is an
    # invalid value"; U: "Numeric value expected"; rest: bare period).
    ("AGS Format Rule 8",
     re.compile(r'^Value "(?P<val>[^"]*)" in (?P<head>[A-Z0-9]{4}_[A-Z0-9]+) '
                r'does not match its declared TYPE "(?P<typ>[^"]+)"'
                r'(?: / UNIT "[^"]*")?\.'
                r"(?P<expected> \(Expected: [^)]+\))?$"),
     # Bare form; the suffix hook in `translate()` patches it.
     r"Value \g<val> in \g<head> not of data type \g<typ>.\g<expected>"),
    # ID uniqueness: laterite says "ID value … in this ID column is not unique";
    # python-ags4 says "Value X in SAMP_ID is not unique." We don't have the
    # heading in laterite's message; lift it via a best-effort match.
    ("AGS Format Rule 8",
     re.compile(r'^ID value "(?P<val>[^"]*)" in this ID column is not unique\.$'),
     r"Value \g<val> in SAMP_ID is not unique."),

    # ---- Rule 9 — heading not in dictionary --------------------------
    # Laterite: 'Heading "LOCA_APPG" is not in the standard dictionary or the file\'s DICT group.'
    # python-ags4: 'LOCA_APPG not found in DICT group or the standard AGS4 dictionary.'
    ("AGS Format Rule 9",
     re.compile(r'^Heading "(?P<head>[A-Z0-9]{4}_[A-Z0-9]+)" is not in the '
                r"standard dictionary or the file's DICT group\.?$"),
     r"\g<head> not found in DICT group or the standard AGS4 dictionary."),

    # ---- Rule 10a — KEY field issues ---------------------------------
    ("AGS Format Rule 10a",
     re.compile(r"^Duplicate KEY field combination: (?P<keys>.+)$"),
     r"Duplicate key field combination: DATA|\g<keys>"),
    ("AGS Format Rule 10a",
     re.compile(r"^KEY field (?P<head>[A-Z0-9_]+) is not present\.?$"),
     r"Key field \g<head> not found."),

    # ---- Rule 10b — REQUIRED field issues ----------------------------
    ("AGS Format Rule 10b",
     re.compile(r"^REQUIRED field (?P<head>[A-Z0-9_]+) is not present\.?$"),
     r"Required field \g<head> not found."),
    # Stage 7f: laterite now emits the row-with-markers form directly
    # ("Empty REQUIRED fields: DATA|val1|??FIELD??|val3|..."). No
    # translation needed — the Rust emit matches python-ags4 verbatim.

    # ---- Rule 10c — parent-group issues ------------------------------
    # 'No parent entry in SAMP for KEY combination: X' → 'Parent entry for line not found in SAMP: X'
    ("AGS Format Rule 10c",
     re.compile(r"^No parent entry in (?P<grp>[A-Z]{4}) for KEY combination: "
                r"(?P<keys>.*)$"),
     r"Parent entry for line not found in \g<grp>: \g<keys>"),
    ("AGS Format Rule 10c",
     re.compile(r"^Parent group (?P<grp>[A-Z]{4}) is not in the file\.?$"),
     r"Could not find parent group \g<grp>."),
    ("AGS Format Rule 10c",
     re.compile(r"^No KEY fields are defined in the parent group "
                r"\((?P<grp>[A-Z]{4})\)\.?$"),
     r"No key fields have been defined in parent group (\g<grp>). "
     r"Please check DICT group."),
    ("AGS Format Rule 10c",
     re.compile(r"^(?P<fields>[A-Z0-9_,\s]+) defined as KEY field\(s\) in the "
                r"parent group \((?P<grp>[A-Z]{4})\) but not in the child group\.?$"),
     r"\g<fields> defined as key field(s) in the parent group (\g<grp>) but "
     r"not in the child group. Please check DICT group."),

    # ---- Rule 11b — TRAN_RCON ----------------------------------------
    ("AGS Format Rule 11b",
     re.compile(r"^TRAN_RCON is missing\.?$"),
     "TRAN_RCON missing."),

    # ---- Rule 11c — invalid record links -----------------------------
    # Laterite: 'Invalid Record Link "ISPT|327-16A|2": no such record.'
    # python-ags4: 'Invalid record link: "ISPT|327-16A|2". No such record found.'
    ("AGS Format Rule 11c",
     re.compile(r'^Invalid Record Link "(?P<link>[^"]+)": no such record\.?$'),
     r'Invalid record link: "\g<link>". No such record found.'),
    ("AGS Format Rule 11c",
     re.compile(r'^Invalid Record Link "(?P<link>[^"]+)": "@" must separate '
                r"the GROUP and KEY fields\.?$"),
     r'Invalid record link: "\g<link>". "@" should be used as delimiter.'),

    # ---- Rule 13 — PROJ -----------------------------------------------
    ("AGS Format Rule 13",
     re.compile(r"^The PROJ group must contain only one DATA row\.?$"),
     "There should not be more than one DATA row in the PROJ group."),

    # ---- Rule 14 — TRAN -----------------------------------------------
    ("AGS Format Rule 14",
     re.compile(r"^The TRAN group must contain only one DATA row\.?$"),
     "There should not be more than one DATA row in the TRAN group."),

    # ---- Rule 15 — unit not in UNIT group ----------------------------
    # Laterite: 'Unit "%" (first used in the UNIT row of LLPL) is not defined in the UNIT group.'
    # python-ags4: 'Unit "%" not found in UNIT group. (This unit first appears in UNIT row in LLPL group)'
    ("AGS Format Rule 15",
     re.compile(r'^Unit "(?P<u>[^"]+)" \(first used in the UNIT row of '
                r"(?P<grp>[A-Z]{4})\) is not defined in the UNIT group\.?$"),
     r'Unit "\g<u>" not found in UNIT group. '
     r"(This unit first appears in UNIT row in \g<grp> group)"),
    # 'Unit "mg/l" (first used in column ELRG_RUNI of ELRG) is not defined…'
    # → 'Unit "mg/l" not found in UNIT group. (This unit first appears in ELRG_RUNI column in ELRG group)'
    ("AGS Format Rule 15",
     re.compile(r'^Unit "(?P<u>[^"]+)" \(first used in column '
                r"(?P<col>[A-Z0-9_]+) of (?P<grp>[A-Z]{4})\) is not defined in "
                r"the UNIT group\.?$"),
     r'Unit "\g<u>" not found in UNIT group. '
     r"(This unit first appears in \g<col> column in \g<grp> group)"),

    # ---- Rule 16 — abbreviation not in ABBR group --------------------
    # Laterite: 'Abbreviation "RC" under LOCA_TYPE is not defined in the ABBR group.'
    # python-ags4: '"RC" under LOCA_TYPE in LOCA not found in ABBR group.'
    # The "in <GROUP>" suffix is the finding.group (the table the abbr was used in).
    # We'll handle that via a post-hook so we can stitch the group name.

    # ---- Rule 17 — data type not in TYPE group -----------------------
    ("AGS Format Rule 17",
     re.compile(r'^Data type "(?P<t>[^"]+)" is not defined in the TYPE group\.?$'),
     r'Data type "\g<t>" not found in TYPE group.'),

    # ---- Rule 19 — GROUP name -----------------------------------------
    ("AGS Format Rule 19",
     re.compile(r"^GROUP name must be exactly 4 uppercase letters.*$"),
     "GROUP name should consist of four uppercase letters."),

    # ---- Rule 19a — heading characters --------------------------------
    ("AGS Format Rule 19a",
     re.compile(r'^Heading "(?P<head>[A-Za-z0-9_]+)" is more than 9 '
                r"characters long\.?$"),
     r"Heading \g<head> is more than 9 characters in length."),
    ("AGS Format Rule 19a",
     re.compile(r'^Heading "(?P<head>[A-Za-z0-9_\-]+)" must contain only '
                r"uppercase letters, digits, and underscore\.?$"),
     r"Heading \g<head> should consist of only uppercase letters, numbers, "
     r"and an underscore character."),

    # ---- Rule 19b — heading prefix shape -----------------------------
    # Laterite: 'Heading "TST_DPTH" must be a 4-letter group-name prefix + underscore + a 1–4 character field …'
    # python-ags4: 'Heading TST_DPTH should consist of a 4 character group name and a field name of up to 4 characters.'
    ("AGS Format Rule 19b",
     re.compile(r'^Heading "(?P<head>[A-Za-z0-9_]+)" must be a 4-letter '
                r"group-name prefix \+ underscore \+ a 1[–-]4 character field.*$"),
     r"Heading \g<head> should consist of a 4 character group name and a "
     r"field name of up to 4 characters."),
    # Stage 9c: the "Group X referred to in Y could not be found..."
    # and "X does not start with the name of this group..." messages
    # are now emitted verbatim by Rust (`rules/references.rs`),
    # matching python-ags4's `rule_19b_2` / `rule_19b_3` exactly. No
    # translator entry needed for these — the pre-Stage-9c entry that
    # collapsed our "Heading X names group Y, which is not defined..."
    # into python-ags4's rule_19b_3 wording is now obsolete (we don't
    # emit that string anymore).

    # ---- Rule 20 — FILE attachments ----------------------------------
    ("AGS Format Rule 20",
     re.compile(r'^FILE_FSET "(?P<f>[^"]+)" is not defined in the FILE group\.?$'),
     r'FILE_FSET entry "\g<f>" not found in FILE group.'),
    ("AGS Format Rule 20",
     re.compile(r"^Declared FILE_FSET sub-folder '(?P<p>[^']+)' is missing on disk\.?$"),
     r'Sub-folder named "\g<p>" not found even though it is defined in the FILE group.'),
    ("AGS Format Rule 20",
     re.compile(r"^Declared file '(?P<p>[^']+)' is missing on disk\.?$"),
     r'File named "\g<p>" not found even though it is defined in the FILE group.'),
]

# Rule 16's translation needs the finding group, so it's a post-hook.
_RULE_16_RE = re.compile(
    r'^Abbreviation "(?P<v>[^"]+)" under (?P<col>[A-Z0-9_]+) is not '
    r"defined in the ABBR group\.?$"
)

# Rule 8 final-form: python-ags4 appends a type-specific suffix on
# certain data types — match that after the main regex sub. The optional
# tail captures the SF " (Expected: NN)" trailer the validator now emits
# so we don't append a DMS/U suffix on top of it.
_RULE_8_BARE = re.compile(
    r"^Value (?P<val>\S+) in (?P<head>[A-Z0-9_]+) not of data type "
    r"(?P<typ>\S+?)\.(?P<tail>(?: \([^)]+\))?)$"
)
_RULE_8_NUMERIC_SUFFIX = "Numeric value expected."
_RULE_8_DMS_SUFFIX = "or is an invalid value."
# Types where python-ags4 emits "Numeric value expected." (anything
# numeric where the value couldn't be parsed as a number at all).
_RULE_8_NUMERIC_TYPES = {"U", "MC"}


def translate(rule: str, group: str, desc: str) -> str:
    """Rewrite ``desc`` into python-ags4 wording for the given rule.

    Falls back to the original laterite phrasing if no pattern matches.
    Argument ``group`` is the finding's group code, used only by the
    Rule 16 stitch (``"X" under HEAD in <group> not found…``).
    """
    if rule == "AGS Format Rule 16":
        m = _RULE_16_RE.match(desc)
        if m is not None:
            return f'"{m["v"]}" under {m["col"]} in {group} not found in ABBR group.'
        return desc

    for r, pat, repl in _TABLE:
        if r != rule:
            continue
        m = pat.match(desc)
        if m is not None:
            translated = pat.sub(repl, desc)
            if rule == "AGS Format Rule 8":
                translated = _rule_8_suffix(translated)
            return translated
    return desc


def _rule_8_suffix(translated: str) -> str:
    """Patch python-ags4's type-specific Rule 8 suffix onto the bare form.

    ``Value X in COL not of data type DMS.`` becomes
    ``Value X in COL not of data type DMS or is an invalid value.``
    ``Value x in COL not of data type U.`` becomes
    ``Value x in COL not of data type U. Numeric value expected.``
    Other types keep the bare period. The numeric-suffix branch fires
    only when the value really isn't numeric (string passed an int/float
    type) — best-effort detected by failing ``float(val)``.
    """
    m = _RULE_8_BARE.match(translated)
    if m is None:
        return translated
    typ = m["typ"]
    val = m["val"]
    tail = m["tail"] or ""
    # If the validator already appended a parenthetical (the SF Expected
    # suffix), don't double-stack a DMS/U suffix on top of it.
    if tail:
        return translated
    if typ == "DMS":
        return translated[:-1] + " " + _RULE_8_DMS_SUFFIX
    if typ in _RULE_8_NUMERIC_TYPES:
        try:
            float(val)
            return translated  # numeric value but failed unit/range check
        except ValueError:
            return translated + " " + _RULE_8_NUMERIC_SUFFIX
    return translated
