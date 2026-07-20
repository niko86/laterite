//! The `lat read` output formats — ONE renderer per format, for every surface.
//!
//! `read` hands each surface the same raw shape (a group's `headings` plus its
//! `rows`, cells already in heading order — see `read_groups_raw` in the PyO3 /
//! napi bindings), and each surface used to serialise it itself: the `lat`
//! binary in `commands/read.rs`, Node in `ts/cli.ts`, Python in `_cli.py`.
//!
//! That is a thinner promise than it looks. The three CSV writers were three
//! hand-ports of RFC-4180 quoting, and the three JSON writers were three
//! *different JSON libraries* — `serde_json`, JS's `JSON.stringify(x, null, 2)`
//! and Python's `json.dumps(indent=2, ensure_ascii=False)` — held byte-identical
//! only by hand-discipline, with no gate on `lat read` output at all (#530).
//! Rendering here means one writer per format, and `read --json` / `--csv`
//! cannot mean three different things.
//!
//! Rows are positional (already projected through `headings`), matching what the
//! bindings expose; a short row is padded with empty cells so a ragged group
//! still renders a rectangle.

use serde_json::{Map, Value};

/// One group as pretty JSON: an array of `{heading: cell}` objects, in heading
/// order (`preserve_order` — see Cargo.toml). Trailing newline.
#[must_use]
pub fn render_rows_json(headings: &[String], rows: &[Vec<String>]) -> String {
    let arr: Vec<Value> = rows
        .iter()
        .map(|row| {
            let mut o = Map::new();
            for (i, h) in headings.iter().enumerate() {
                let cell = row.get(i).cloned().unwrap_or_default();
                o.insert(h.clone(), Value::from(cell));
            }
            Value::Object(o)
        })
        .collect();
    serde_json::to_string_pretty(&Value::Array(arr)).unwrap_or_default() + "\n"
}

/// One group as CSV: the heading row, then one row per record. Trailing newline
/// on every line (`csv_row`).
pub fn render_rows_csv(headings: &[String], rows: &[Vec<String>]) -> String {
    let mut s = csv_row(headings.iter().map(String::as_str));
    for row in rows {
        // Project through `headings` so a short row still emits the full width.
        s.push_str(&csv_row(
            (0..headings.len()).map(|i| row.get(i).map_or("", String::as_str)),
        ));
    }
    s
}

/// One RFC-4180-ish CSV line: quote a field iff it contains `,` / `"` / CR / LF,
/// doubling internal quotes. Trailing `\n`.
pub fn csv_row<'a>(cells: impl Iterator<Item = &'a str>) -> String {
    let fields: Vec<String> = cells
        .map(|c| {
            if c.contains([',', '"', '\r', '\n']) {
                format!("\"{}\"", c.replace('"', "\"\""))
            } else {
                c.to_string()
            }
        })
        .collect();
    fields.join(",") + "\n"
}

#[cfg(test)]
mod tests {
    use super::*;

    fn hs(v: &[&str]) -> Vec<String> {
        v.iter().map(std::string::ToString::to_string).collect()
    }

    #[test]
    fn csv_quotes_only_when_needed() {
        assert_eq!(csv_row(["a", "b"].into_iter()), "a,b\n");
        // comma / quote / CR / LF force quoting; internal quotes double.
        assert_eq!(csv_row(["a,b"].into_iter()), "\"a,b\"\n");
        assert_eq!(csv_row(["say \"hi\""].into_iter()), "\"say \"\"hi\"\"\"\n");
        assert_eq!(csv_row(["a\nb"].into_iter()), "\"a\nb\"\n");
        assert_eq!(csv_row(["a\rb"].into_iter()), "\"a\rb\"\n");
    }

    #[test]
    fn csv_renders_headings_then_rows() {
        let out = render_rows_csv(
            &hs(&["LOCA_ID", "LOCA_GL"]),
            &[hs(&["BH01", "10.00"]), hs(&["BH02", "20.00"])],
        );
        assert_eq!(out, "LOCA_ID,LOCA_GL\nBH01,10.00\nBH02,20.00\n");
    }

    #[test]
    fn json_is_an_array_of_heading_keyed_objects_in_heading_order() {
        let out = render_rows_json(&hs(&["LOCA_ID", "LOCA_GL"]), &[hs(&["BH01", "10.00"])]);
        // Heading order, not alphabetical — LOCA_ID precedes LOCA_GL.
        assert_eq!(
            out,
            "[\n  {\n    \"LOCA_ID\": \"BH01\",\n    \"LOCA_GL\": \"10.00\"\n  }\n]\n"
        );
    }

    #[test]
    fn a_short_row_pads_rather_than_shifting_cells() {
        // A ragged row must not slide values under the wrong heading.
        assert_eq!(
            render_rows_csv(&hs(&["A", "B", "C"]), &[hs(&["1"])]),
            "A,B,C\n1,,\n"
        );
        assert_eq!(
            render_rows_json(&hs(&["A", "B"]), &[hs(&["1"])]),
            "[\n  {\n    \"A\": \"1\",\n    \"B\": \"\"\n  }\n]\n"
        );
    }

    #[test]
    fn no_rows_still_emits_the_csv_header_and_an_empty_json_array() {
        assert_eq!(render_rows_csv(&hs(&["A", "B"]), &[]), "A,B\n");
        assert_eq!(render_rows_json(&hs(&["A", "B"]), &[]), "[]\n");
    }
}
