//! `laterite-ags4-censor` — the shared AGS4 anonymisation engine.
//!
//! ONE copy of the scrub logic, extracted (#581) out of `ags4-corpus-qa`'s
//! `censor.rs` so the browser `Anonymiser` drives the same engine (through the
//! engine wasm) instead of a hand-written TS reimplementation. Part of the #527
//! cross-surface convergence arc — the sibling of the #533 tokenizer/quoter
//! work: the scrub now reads fields through the shared [`scan_line`] parse
//! leaf and re-quotes through `laterite-ags4-types`, so no fourth AGS4 tokenizer.
//!
//! **Cell-surgical, defect-preserving.** Only DATA cells that actually change
//! are rewritten; every other byte — GROUP/HEADING/UNIT/TYPE rows, blank lines,
//! line endings, and format defects (a Rule-2a CRLF breach, stray whitespace) —
//! passes through verbatim. Within a changed row, untouched cells keep their
//! original bytes (a scrubbed sibling never re-quotes them). A row whose columns
//! are *dropped* (custom-column removal) necessarily re-emits its kept fields
//! canonically, since its structure changes.
//!
//! Actions (per the SSOT `scrub_policy`, resolved into a [`Policy`]):
//! - `filehash` (project IDs): the cell becomes the source file's content hash
//!   (a caller-provided `file_id`), so `PROJ_ID` is stable + non-identifying.
//! - `pseudonym` (location IDs): each distinct value → a stable token
//!   (`ID0001`…), the SAME map reused wherever that column appears, so
//!   cross-group references survive and the file still validates.
//! - `blank` (coordinates): emptied.
//! - `token` (names / labs / accreditation): replaced with the options' token.
//! - `brackets` (free-text): each `[LONDON CLAY]` bracketed unit → `[<token>]`,
//!   the rest of the description kept.
//! - `skip` (free-text remarks): left intact unless `include_freetext`
//!   (in [`Policy::from_sensitive_json`]) promotes it to `token`.

use std::collections::{HashMap, HashSet};
use std::sync::OnceLock;

use laterite_ags4_parse::scan::{DISPLAY, RawField, scan_line};
use laterite_ags4_types::quote_field;
use serde::Deserialize;

// --- classification (the SSOT `sensitive_headings.json` shape) --------------

#[derive(Debug, Deserialize)]
struct SensitiveDoc {
    scrub_policy: HashMap<String, String>,
    headings: HashMap<String, HeadingEntry>,
}

#[derive(Debug, Deserialize)]
struct HeadingEntry {
    category: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Action {
    /// Cell → the source file's content hash (a caller-provided `file_id`).
    FileHash,
    Pseudonym,
    Blank,
    Token,
    /// Keep the cell but replace each bracketed geological unit
    /// (`[LONDON CLAY]`) with `[<token>]`.
    Brackets,
    Skip,
}

fn parse_action(s: &str) -> Action {
    match s {
        "filehash" => Action::FileHash,
        "pseudonym" => Action::Pseudonym,
        "blank" => Action::Blank,
        "token" => Action::Token,
        "brackets" => Action::Brackets,
        _ => Action::Skip,
    }
}

/// Heading code → action, resolved from the classification (heading → category
/// → `scrub_policy`). Build it once per file with [`Policy::from_sensitive_json`]
/// (or restrict it to a subset with [`Policy::retain_codes`], which the browser
/// uses to honour the user's column selection).
#[derive(Debug, Clone, Default)]
pub struct Policy {
    action: HashMap<String, Action>,
}

impl Policy {
    /// Resolve the SSOT `sensitive_headings.json` text into a policy. Each
    /// heading's `category` is looked up in `scrub_policy`; unmapped →
    /// [`Action::Skip`]. `include_freetext` promotes the kept-text actions
    /// (`skip`/`brackets`) to a full `token`, tokenising descriptions.
    pub fn from_sensitive_json(
        json: &str,
        include_freetext: bool,
    ) -> Result<Self, serde_json::Error> {
        let doc: SensitiveDoc = serde_json::from_str(json)?;
        let mut action = HashMap::new();
        for (code, h) in &doc.headings {
            let mut a = doc
                .scrub_policy
                .get(&h.category)
                .map_or(Action::Skip, |s| parse_action(s));
            if include_freetext && (a == Action::Skip || a == Action::Brackets) {
                a = Action::Token;
            }
            action.insert(code.clone(), a);
        }
        Ok(Policy { action })
    }

    /// Restrict the policy to `keep` heading codes — every other code becomes
    /// unclassified (untouched). The browser passes its user-selected column
    /// set so deselecting a sensitive column leaves it alone.
    pub fn retain_codes(&mut self, keep: &HashSet<String>) {
        self.action.retain(|code, _| keep.contains(code));
    }
}

/// Everything a run needs beyond the classification: the replacement token, the
/// keyword safety-net list, and whether custom (non-dictionary) groups/columns
/// are dropped.
#[derive(Debug, Clone)]
pub struct CensorOptions {
    /// Replacement string for `token`/`brackets`/keyword hits.
    pub token: String,
    /// Substrings replaced (ASCII case-insensitive) wherever they appear in any
    /// data cell — a safety net for known client names.
    pub keywords: Vec<String>,
    /// Drop whole custom (non-dictionary) groups + custom columns + their
    /// orphaned DICT/ABBR definition rows.
    pub drop_custom: bool,
}

impl Default for CensorOptions {
    fn default() -> Self {
        CensorOptions {
            token: "REDACTED".to_string(),
            keywords: Vec::new(),
            drop_custom: true,
        }
    }
}

/// Per-category cell/structure tallies for a run.
#[derive(Debug, Default, Clone, PartialEq, Eq, serde::Serialize)]
pub struct Tally {
    pub pseudonym: u64,
    pub blank: u64,
    pub token: u64,
    /// Bracketed geological units stripped from description cells.
    pub brackets: u64,
    /// Substrings replaced by the keyword pass.
    pub keyword: u64,
    /// Custom (non-dictionary) columns deleted.
    pub dropped_cols: u64,
    /// Custom (non-dictionary) groups deleted.
    pub dropped_groups: u64,
    /// Orphaned DICT/ABBR definition rows of dropped custom groups/headings.
    pub dropped_defs: u64,
}

impl Tally {
    pub fn merge(&mut self, o: &Tally) {
        self.pseudonym += o.pseudonym;
        self.blank += o.blank;
        self.token += o.token;
        self.brackets += o.brackets;
        self.keyword += o.keyword;
        self.dropped_cols += o.dropped_cols;
        self.dropped_groups += o.dropped_groups;
        self.dropped_defs += o.dropped_defs;
    }
}

// --- dictionary: the standard group/heading code sets -----------------------

/// The set of standard group codes and standard heading codes (across every
/// edition) from the reference leaf's dictionary SSOT — cached once. `drop_custom`
/// deletes anything not in these sets.
fn standard_codes() -> &'static (HashSet<String>, HashSet<String>) {
    static STD: OnceLock<(HashSet<String>, HashSet<String>)> = OnceLock::new();
    STD.get_or_init(|| {
        let reg = laterite_ags4_reference::union::registry();
        let mut groups = HashSet::new();
        let mut headings = HashSet::new();
        for g in reg.iter() {
            groups.insert(g.code.clone());
            for h in &g.headings {
                headings.insert(h.name.clone());
            }
        }
        (groups, headings)
    })
}

// --- line reading (strict all-quoted gate over the shared tokenizer) --------

/// Split text into (content, terminator) chunks preserving the exact line
/// ending of each line (CRLF / LF / none on a final unterminated line).
fn lines_with_terminators(text: &str) -> Vec<(&str, &str)> {
    text.split_inclusive('\n')
        .map(|chunk| {
            if let Some(stripped) = chunk.strip_suffix("\r\n") {
                (stripped, "\r\n")
            } else if let Some(stripped) = chunk.strip_suffix('\n') {
                (stripped, "\n")
            } else {
                (chunk, "")
            }
        })
        .collect()
}

/// Decode a physical AGS4 line into its field VALUES, but only if the line is
/// well-formed all-quoted AGS4 (`"a","b,c","he ""hi"""`). Returns `None` for
/// anything else — an unquoted field, junk after a close, an unterminated
/// quote, a trailing comma (a phantom empty field), or a blank line — so the
/// caller passes that line through verbatim and we never corrupt what we can't
/// cleanly read (a malformed row, a quoted-newline continuation).
///
/// Tokenizing is the shared parse leaf's [`scan_line`]; this is only the
/// strict all-quoted *gate* the byte-faithful scrub needs on top of it (the
/// tolerant tokenizer never rejects).
fn clean_quoted_fields(content: &str, spans: &[RawField]) -> Option<Vec<String>> {
    // A well-formed line's final field carries no trailing delimiter; a trailing
    // comma means a phantom (unquoted, empty) field → malformed. The scanner
    // records this directly, so it is no longer inferred from a string suffix.
    if spans.last().is_none_or(|s| s.had_comma) {
        return None;
    }
    let mut out = Vec::with_capacity(spans.len());
    for s in spans {
        // Byte bounds into the line we were given — no per-field copy. This is
        // what `AgsSpan.text` was standing in for when the offsets were code
        // points and therefore unusable from Rust.
        let raw = &content[s.token_start..token_content_end(s)];
        // Every field must be `"…"`. The outer quotes are ASCII (1 byte each),
        // so the slice is on char boundaries; un-double the escaped inner quotes.
        if raw.len() < 2 || !raw.starts_with('"') || !raw.ends_with('"') {
            return None;
        }
        out.push(raw[1..raw.len() - 1].replace("\"\"", "\""));
    }
    Some(out)
}

/// Tokenize one line's content (terminator already split off) and decode it if
/// it's clean all-quoted AGS4. Returns the field spans (for cell-surgical
/// re-emit) alongside the decoded values.
fn read_line(content: &str) -> Option<(Vec<RawField>, Vec<String>)> {
    let body = content.trim_end_matches('\r');
    let spans = scan_line(body, DISPLAY);
    let values = clean_quoted_fields(body, &spans)?;
    Some((spans, values))
}

/// One past a token's content, excluding the trailing comma when one closed it.
fn token_content_end(s: &RawField) -> usize {
    if s.had_comma {
        s.token_end - 1
    } else {
        s.token_end
    }
}

/// Canonical AGS4 emission of a field list — every field quoted (inner quotes
/// doubled), joined by commas. Used when a row's structure changes (custom
/// columns dropped), where the original per-field bytes can't be preserved.
fn emit_canonical(fields: &[String]) -> String {
    fields
        .iter()
        .map(|f| quote_field(f))
        .collect::<Vec<_>>()
        .join(",")
}

// --- the scrub actions ------------------------------------------------------

/// Replace `[<unit>]` runs with `[<token>]` — strips location-revealing
/// geological formations from a description, keeping the rest. Returns the new
/// string and how many bracket groups were stripped. Empty/whitespace/unclosed
/// brackets are kept literal (don't invent content).
fn strip_brackets(val: &str, token: &str) -> (String, u64) {
    if !val.contains('[') {
        return (val.to_string(), 0);
    }
    let mut out = String::with_capacity(val.len());
    let mut n = 0;
    let mut chars = val.chars().peekable();
    while let Some(c) = chars.next() {
        if c != '[' {
            out.push(c);
            continue;
        }
        let mut inner = String::new();
        let mut closed = false;
        for d in chars.by_ref() {
            if d == ']' {
                closed = true;
                break;
            }
            inner.push(d);
        }
        if closed && !inner.trim().is_empty() {
            out.push('[');
            out.push_str(token);
            out.push(']');
            n += 1;
        } else {
            out.push('[');
            out.push_str(&inner);
            if closed {
                out.push(']');
            }
        }
    }
    (out, n)
}

/// Replace every keyword (ASCII case-insensitive) in `val` with `token`. Resumes
/// past the inserted token so a keyword that's a token substring can't loop.
fn redact_keywords(val: &str, keywords: &[String], token: &str) -> (String, u64) {
    let mut out = val.to_string();
    let mut n = 0;
    for kw in keywords {
        if kw.is_empty() {
            continue;
        }
        let klow = kw.to_ascii_lowercase();
        let mut start = 0;
        while start <= out.len() {
            let Some(rel) = out[start..].to_ascii_lowercase().find(&klow) else {
                break;
            };
            let pos = start + rel;
            out.replace_range(pos..pos + kw.len(), token);
            n += 1;
            start = pos + token.len();
        }
    }
    (out, n)
}

/// Is this a DICT/ABBR row that *defines* a custom (non-standard) group or
/// heading? Such rows are orphaned once their target is dropped, and the
/// definition itself (the custom name/description) can be client-specific.
fn is_orphan_def(
    group: &str,
    headings: &[String],
    fields: &[String],
    std_groups: &HashSet<String>,
    std_headings: &HashSet<String>,
) -> bool {
    let cell = |name: &str| {
        headings
            .iter()
            .position(|h| h == name)
            .and_then(|i| fields.get(i))
            .map_or("", String::as_str)
    };
    match group {
        "DICT" => {
            let grp = cell("DICT_GRP");
            let hdng = cell("DICT_HDNG");
            (!grp.is_empty() && !std_groups.contains(grp))
                || (!hdng.is_empty() && !std_headings.contains(hdng))
        }
        "ABBR" => {
            let h = cell("ABBR_HDNG");
            !h.is_empty() && !std_headings.contains(h)
        }
        _ => false,
    }
}

/// Scrub one cell's value in place — returns the new value (== input when
/// unchanged) and updates `tally`. The row tag (index 0) is never scrubbed.
#[allow(clippy::too_many_arguments)]
fn scrub_cell(
    i: usize,
    val: &str,
    headings: &[String],
    abbr_sensitive: bool,
    file_id: &str,
    policy: &Policy,
    token: &str,
    keywords: &[String],
    pseudo: &HashMap<String, HashMap<String, String>>,
    tally: &mut Tally,
) -> String {
    if i == 0 {
        return val.to_string();
    }
    let mut v = val.to_string();
    let code = headings.get(i).map(String::as_str);
    if abbr_sensitive && matches!(code, Some("ABBR_CODE" | "ABBR_DESC")) {
        if !v.is_empty() {
            v = token.to_string();
            tally.token += 1;
        }
    } else if let Some(action) = code.and_then(|c| policy.action.get(c)) {
        if !v.is_empty() {
            match action {
                Action::FileHash => {
                    if v != file_id {
                        v = file_id.to_string();
                        tally.pseudonym += 1;
                    }
                }
                Action::Blank => {
                    v.clear();
                    tally.blank += 1;
                }
                Action::Token => {
                    v = token.to_string();
                    tally.token += 1;
                }
                Action::Pseudonym => {
                    if let Some(p) = code.and_then(|c| pseudo.get(c)).and_then(|m| m.get(val)) {
                        v.clone_from(p);
                        tally.pseudonym += 1;
                    }
                }
                Action::Brackets => {
                    let (nv, k) = strip_brackets(&v, token);
                    if k > 0 {
                        v = nv;
                        tally.brackets += k;
                    }
                }
                Action::Skip => {}
            }
        }
    }
    // Keyword safety net over every kept cell, post-scrub.
    if !keywords.is_empty() && !v.is_empty() {
        let (nv, k) = redact_keywords(&v, keywords, token);
        if k > 0 {
            v = nv;
            tally.keyword += k;
        }
    }
    v
}

// --- the engine -------------------------------------------------------------

/// Anonymise one file's text. Two passes: pass 1 mints the per-file pseudonym
/// maps (first-seen order ⇒ deterministic); pass 2 rewrites. A row passes
/// through byte-for-byte unless it actually changes (a scrub, a dropped custom
/// column, or a keyword hit) — so format defects survive. Whole custom
/// (non-dictionary) groups are dropped when `opts.drop_custom`.
///
/// `file_id` is the value the `filehash` action writes into `PROJ_ID` (the
/// caller's source-content hash — full-width, so a KEY collision is
/// cryptographically nil).
pub fn censor(text: &str, file_id: &str, policy: &Policy, opts: &CensorOptions) -> (String, Tally) {
    let (std_groups, std_headings) = standard_codes();
    let chunks = lines_with_terminators(text);

    // --- pass 1: build pseudonym maps (code -> value -> pseudonym) ----------
    let mut pseudo: HashMap<String, HashMap<String, String>> = HashMap::new();
    let mut headings: Vec<String> = Vec::new();
    for (content, _) in &chunks {
        let Some((_, fields)) = read_line(content) else {
            continue;
        };
        match fields.first().map(String::as_str) {
            Some("HEADING") => headings.clone_from(&fields),
            Some("DATA") => {
                for (i, val) in fields.iter().enumerate().skip(1) {
                    let Some(code) = headings.get(i) else {
                        continue;
                    };
                    if policy.action.get(code) == Some(&Action::Pseudonym) && !val.is_empty() {
                        let map = pseudo.entry(code.clone()).or_default();
                        if !map.contains_key(val) {
                            let prefix = code
                                .rsplit('_')
                                .next()
                                .unwrap_or("ANON")
                                .to_ascii_uppercase();
                            let next = format!("{prefix}{:04}", map.len() + 1);
                            map.insert(val.clone(), next);
                        }
                    }
                }
            }
            _ => {}
        }
    }

    // --- pass 2: rewrite ----------------------------------------------------
    let mut out = String::with_capacity(text.len());
    let mut tally = Tally::default();
    headings.clear();
    let mut drop_idx: HashSet<usize> = HashSet::new();
    let mut skipping = false; // inside a custom group being deleted
    let mut cur_group = String::new();

    for (content, term) in &chunks {
        let Some((spans, values)) = read_line(content) else {
            // Unparseable line: a continuation of a dropped group's data is
            // dropped too; otherwise verbatim.
            if !skipping {
                out.push_str(content);
                out.push_str(term);
            }
            continue;
        };
        let tag = values.first().map_or("", String::as_str);

        if tag == "GROUP" {
            let code = values.get(1).map_or("", String::as_str);
            cur_group = code.to_string();
            skipping = opts.drop_custom && !code.is_empty() && !std_groups.contains(code);
            headings.clear();
            drop_idx.clear();
            if skipping {
                tally.dropped_groups += 1;
                continue;
            }
            out.push_str(content);
            out.push_str(term);
            continue;
        }
        if skipping {
            continue;
        }

        if tag == "HEADING" {
            headings.clone_from(&values);
            drop_idx = if opts.drop_custom {
                (1..values.len())
                    .filter(|&i| !std_headings.contains(&values[i]))
                    .collect()
            } else {
                HashSet::new()
            };
            tally.dropped_cols += drop_idx.len() as u64;
        }

        // Non-DATA descriptor rows (HEADING/UNIT/TYPE/blank): only custom-column
        // deletion applies. No dropped columns ⇒ byte-verbatim.
        if tag != "DATA" {
            if drop_idx.is_empty() {
                out.push_str(content);
            } else {
                let kept: Vec<String> = values
                    .iter()
                    .enumerate()
                    .filter(|(i, _)| !drop_idx.contains(i))
                    .map(|(_, f)| f.clone())
                    .collect();
                out.push_str(&emit_canonical(&kept));
            }
            out.push_str(term);
            continue;
        }

        // Orphaned DICT/ABBR definition of a dropped custom group/heading → drop
        // the whole row (the custom name itself can be client data).
        if opts.drop_custom
            && is_orphan_def(&cur_group, &headings, &values, std_groups, std_headings)
        {
            tally.dropped_defs += 1;
            continue;
        }

        // An ABBR row defines the allowed pick-list values FOR a heading
        // (ABBR_HDNG). If that heading is itself sensitive, the value
        // (ABBR_CODE) and its gloss (ABBR_DESC) are sensitive too.
        let abbr_sensitive = cur_group == "ABBR" && {
            let hv = headings
                .iter()
                .position(|h| h == "ABBR_HDNG")
                .and_then(|i| values.get(i))
                .map_or("", String::as_str);
            policy.action.contains_key(hv)
        };

        if drop_idx.is_empty() {
            // Cell-surgical: untouched cells keep their original bytes; only a
            // changed cell is re-quoted (via the shared quoter), its delimiter
            // comma preserved.
            let mut line = String::new();
            let mut changed = false;
            for (i, span) in spans.iter().enumerate() {
                let val = &values[i];
                let new = scrub_cell(
                    i,
                    val,
                    &headings,
                    abbr_sensitive,
                    file_id,
                    policy,
                    &opts.token,
                    &opts.keywords,
                    &pseudo,
                    &mut tally,
                );
                if new == *val {
                    // `read_line` scanned `content` minus any trailing CR — a
                    // SUFFIX trim, so start-relative byte offsets index either
                    // string identically.
                    line.push_str(&content[span.token_start..span.token_end]);
                } else {
                    changed = true;
                    line.push_str(&quote_field(&new));
                    if span.had_comma {
                        line.push(',');
                    }
                }
            }
            if changed {
                out.push_str(&line);
            } else {
                out.push_str(content);
            }
        } else {
            // Structure changes (columns dropped) → canonical re-emit of the
            // kept, scrubbed fields.
            let kept: Vec<String> = values
                .iter()
                .enumerate()
                .filter(|(i, _)| !drop_idx.contains(i))
                .map(|(i, val)| {
                    scrub_cell(
                        i,
                        val,
                        &headings,
                        abbr_sensitive,
                        file_id,
                        policy,
                        &opts.token,
                        &opts.keywords,
                        &pseudo,
                        &mut tally,
                    )
                })
                .collect();
            out.push_str(&emit_canonical(&kept));
        }
        out.push_str(term);
    }
    (out, tally)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn set(items: &[&str]) -> HashSet<String> {
        items.iter().map(std::string::ToString::to_string).collect()
    }

    fn policy(pairs: &[(&str, Action)]) -> Policy {
        Policy {
            action: pairs.iter().map(|(k, a)| (k.to_string(), *a)).collect(),
        }
    }

    fn opts(token: &str, keywords: &[&str], drop_custom: bool) -> CensorOptions {
        CensorOptions {
            token: token.to_string(),
            keywords: keywords
                .iter()
                .map(std::string::ToString::to_string)
                .collect(),
            drop_custom,
        }
    }

    #[test]
    fn clean_quoted_fields_handles_quoting_and_rejects_malformed() {
        let read = |line: &str| clean_quoted_fields(line, &scan_line(line, DISPLAY));
        assert_eq!(
            read(r#""DATA","a","b,c","he ""hi""""#).unwrap(),
            vec!["DATA", "a", "b,c", "he \"hi\""]
        );
        // Unquoted / junk / unterminated / trailing-comma / blank → None (the
        // caller keeps such a line verbatim). Trailing comma matters: it's a
        // phantom empty field the old strict `parse_fields` also rejected.
        assert!(read("DATA,a,b").is_none());
        assert!(read(r#""a" junk"#).is_none());
        assert!(read(r#""unterminated"#).is_none());
        assert!(read(r#""a","b","#).is_none());
        assert!(read("").is_none());
    }

    #[test]
    fn strip_brackets_replaces_units_only() {
        let (out, n) = strip_brackets("silty CLAY [LONDON CLAY] firm", "REDACTED");
        assert_eq!(out, "silty CLAY [REDACTED] firm");
        assert_eq!(n, 1);
        // Empty / whitespace / unclosed brackets are left literal.
        assert_eq!(
            strip_brackets("a [] b [ ] [oops", "X"),
            ("a [] b [ ] [oops".into(), 0)
        );
    }

    #[test]
    fn redact_keywords_is_case_insensitive_and_cannot_loop() {
        let (out, n) = redact_keywords("the RED and red car", &["red".into()], "REDACTED");
        assert_eq!(out, "the REDACTED and REDACTED car");
        assert_eq!(n, 2);
    }

    #[test]
    fn is_orphan_def_spots_custom_definitions() {
        let g = set(&["LOCA", "DICT", "ABBR"]);
        let h = set(&["LOCA_ID", "LOCA_NATE", "ABBR_HDNG", "DICT_GRP", "DICT_HDNG"]);
        assert!(is_orphan_def(
            "DICT",
            &["DICT_GRP".into(), "DICT_HDNG".into()],
            &["ZZZZ".into(), String::new()],
            &g,
            &h
        ));
        assert!(!is_orphan_def(
            "DICT",
            &["DICT_GRP".into(), "DICT_HDNG".into()],
            &["LOCA".into(), String::new()],
            &g,
            &h
        ));
        assert!(is_orphan_def(
            "ABBR",
            &["ABBR_HDNG".into()],
            &["ZZZZ_X".into()],
            &g,
            &h
        ));
    }

    const SAMPLE: &str = concat!(
        "\"GROUP\",\"PROJ\"\r\n",
        "\"HEADING\",\"PROJ_ID\",\"PROJ_NAME\",\"CUST_FOO\"\r\n",
        "\"UNIT\",\"\",\"\",\"\"\r\n",
        "\"TYPE\",\"ID\",\"X\",\"X\"\r\n",
        "\"DATA\",\"P1\",\"Acme Corp\",\"clientsecret\"\r\n",
        "\r\n",
        "\"GROUP\",\"LOCA\"\r\n",
        "\"HEADING\",\"LOCA_ID\",\"LOCA_NATE\",\"GEOL_DESC\"\r\n",
        "\"UNIT\",\"\",\"m\",\"\"\r\n",
        "\"TYPE\",\"ID\",\"2DP\",\"X\"\r\n",
        "\"DATA\",\"BH01\",\"123456.78\",\"silty CLAY [LONDON CLAY] near Zephyrco\"\r\n",
        "\"DATA\",\"BH02\",\"999.99\",\"sand\"\r\n",
        "\"DATA\",\"BH01\",\"111.11\",\"clay\"\r\n",
        "\"GROUP\",\"ZZZZ\"\r\n",
        "\"HEADING\",\"ZZZZ_X\"\r\n",
        "\"DATA\",\"customleak\"\r\n",
    );

    fn sample_policy() -> Policy {
        policy(&[
            ("PROJ_ID", Action::FileHash),
            ("PROJ_NAME", Action::Token),
            ("LOCA_ID", Action::Pseudonym),
            ("LOCA_NATE", Action::Blank),
            ("GEOL_DESC", Action::Brackets),
        ])
    }

    #[test]
    fn censor_applies_every_action() {
        let pol = sample_policy();
        let o = opts("REDACTED", &["Zephyrco"], true);
        let (out, t) = censor(SAMPLE, "HASH123", &pol, &o);

        for gone in [
            "Acme",
            "clientsecret",
            "123456.78",
            "999.99",
            "111.11",
            "BH01",
            "BH02",
            "Zephyrco",
            "LONDON CLAY",
            "ZZZZ",
            "customleak",
            "CUST_FOO",
        ] {
            assert!(!out.contains(gone), "leaked {gone:?}:\n{out}");
        }
        assert!(out.contains("\"HASH123\"")); // PROJ_ID == file hash
        assert!(out.contains("[REDACTED]")); // bracket unit stripped
        assert_eq!(out.matches("\"ID0001\"").count(), 2); // BH01 twice
        assert_eq!(out.matches("\"ID0002\"").count(), 1);

        assert_eq!(t.pseudonym, 4); // 1 filehash + 3 LOCA_ID
        assert_eq!(t.blank, 3);
        assert_eq!(t.token, 1);
        assert_eq!(t.brackets, 1);
        assert_eq!(t.keyword, 1);
        assert_eq!(t.dropped_cols, 1); // CUST_FOO
        assert_eq!(t.dropped_groups, 1); // ZZZZ
    }

    #[test]
    fn censor_is_byte_exact_on_canonical_input() {
        // Golden: on well-formed all-quoted input, the output is byte-for-byte
        // predictable — descriptor rows verbatim, only the two scrubbed DATA
        // cells re-quoted, terminators preserved. Pins the extraction so a
        // regression in the port shows up as a byte diff, not just a tally miss.
        let pol = policy(&[("PROJ_ID", Action::FileHash), ("PROJ_NAME", Action::Token)]);
        let input = concat!(
            "\"GROUP\",\"PROJ\"\r\n",
            "\"HEADING\",\"PROJ_ID\",\"PROJ_NAME\"\r\n",
            "\"UNIT\",\"\",\"\"\r\n",
            "\"TYPE\",\"ID\",\"X\"\r\n",
            "\"DATA\",\"P1\",\"Acme Corp\"\r\n",
        );
        let expected = concat!(
            "\"GROUP\",\"PROJ\"\r\n",
            "\"HEADING\",\"PROJ_ID\",\"PROJ_NAME\"\r\n",
            "\"UNIT\",\"\",\"\"\r\n",
            "\"TYPE\",\"ID\",\"X\"\r\n",
            "\"DATA\",\"HASH\",\"REDACTED\"\r\n",
        );
        let (out, _) = censor(input, "HASH", &pol, &opts("REDACTED", &[], false));
        assert_eq!(out, expected);
    }

    #[test]
    fn censor_is_deterministic() {
        let pol = policy(&[("LOCA_ID", Action::Pseudonym)]);
        let o = opts("REDACTED", &[], true);
        let a = censor(SAMPLE, "HASH123", &pol, &o).0;
        let b = censor(SAMPLE, "HASH123", &pol, &o).0;
        assert_eq!(a, b, "same input ⇒ byte-identical output");
    }

    #[test]
    fn abbr_pick_list_of_sensitive_heading_is_tokenised() {
        let pol = policy(&[("PROJ_CONT", Action::Token)]);
        let o = opts("REDACTED", &[], true);
        let ags = concat!(
            "\"GROUP\",\"ABBR\"\r\n",
            "\"HEADING\",\"ABBR_HDNG\",\"ABBR_CODE\",\"ABBR_DESC\"\r\n",
            "\"UNIT\",\"\",\"\",\"\"\r\n",
            "\"TYPE\",\"X\",\"X\",\"X\"\r\n",
            "\"DATA\",\"PROJ_CONT\",\"Some Contractor Ltd\",\"SCL\"\r\n",
            "\"DATA\",\"GEOL_GEOL\",\"CLAY\",\"Clay\"\r\n",
        );
        let (out, _) = censor(ags, "HASH", &pol, &o);
        assert!(!out.contains("Some Contractor Ltd"));
        assert!(out.contains("CLAY"));
    }

    #[test]
    fn untouched_rows_and_terminators_pass_through_verbatim() {
        // No policy, no drop → the file is returned byte-for-byte, mixed line
        // endings + missing final newline intact (the defect-preserving contract).
        let messy = "\"GROUP\",\"PROJ\"\r\n\"HEADING\",\"PROJ_ID\"\n\"DATA\",\"P1\"";
        let (out, t) = censor(
            messy,
            "HASH",
            &Policy::default(),
            &opts("REDACTED", &[], false),
        );
        assert_eq!(out, messy);
        assert_eq!(t, Tally::default());
    }

    #[test]
    fn cell_surgical_keeps_untouched_sibling_cell_bytes() {
        // A changed row whose sibling cell is quoted but carries content the
        // canonical quoter would leave alone — cell-surgical keeps the sibling's
        // exact original bytes, only re-quoting the scrubbed cell.
        let pol = policy(&[("PROJ_ID", Action::Token)]);
        let ags = "\"GROUP\",\"PROJ\"\r\n\"HEADING\",\"PROJ_ID\",\"PROJ_NAME\"\r\n\"DATA\",\"P1\",\"keep me\"\r\n";
        let (out, _) = censor(ags, "H", &pol, &opts("X", &[], false));
        // PROJ_ID scrubbed; the sibling PROJ_NAME cell kept verbatim.
        assert!(
            out.contains("\"DATA\",\"X\",\"keep me\"\r\n"),
            "got:\n{out}"
        );
    }

    #[test]
    fn filehash_uses_full_width_id() {
        // 64-hex file ids flow through unchanged (no truncation): PROJ_ID
        // becomes exactly the caller's id.
        let pol = policy(&[("PROJ_ID", Action::FileHash)]);
        let id = "a".repeat(64);
        let ags = "\"GROUP\",\"PROJ\"\r\n\"HEADING\",\"PROJ_ID\"\r\n\"DATA\",\"P1\"\r\n";
        let (out, _) = censor(ags, &id, &pol, &opts("X", &[], false));
        assert!(out.contains(&format!("\"{id}\"")), "got:\n{out}");
    }

    #[test]
    fn retain_codes_honours_a_column_subset() {
        // Simulates the browser deselecting PROJ_NAME: only PROJ_ID is scrubbed.
        let mut pol = policy(&[("PROJ_ID", Action::Token), ("PROJ_NAME", Action::Token)]);
        pol.retain_codes(&set(&["PROJ_ID"]));
        let ags = "\"GROUP\",\"PROJ\"\r\n\"HEADING\",\"PROJ_ID\",\"PROJ_NAME\"\r\n\"DATA\",\"P1\",\"Acme\"\r\n";
        let (out, _) = censor(ags, "H", &pol, &opts("X", &[], false));
        assert!(
            out.contains("\"Acme\""),
            "deselected column must be untouched:\n{out}"
        );
        assert!(!out.contains("\"P1\""));
    }
}
