//! `dictionary()` — the bundled STANDARD dictionary for an edition.
//!
//! The Tools reference (Dictionary browser / Template generator) used to fetch a
//! static scaffolded dictionary — a single fixed edition where most headings had
//! EMPTY descriptions. This exposes the validator's real per-edition standard
//! dictionary instead: canonical names + descriptions + units + types + status,
//! selectable across 4.0.3 … 4.2 (the same data the engine validates against).
use crate::boundary::to_js;
use crate::resolve::resolve_dict_override;
use laterite_ags4_validator::dict::FALLBACK;
use wasm_bindgen::prelude::*;

// The `dictionary` result — `laterite-ags4-reference`'s `DictionaryDto`, which
// PyO3 and Node also render, from the one shared builder. Bound to that struct
// by `ts_interfaces_match_the_serde_structs`.
ts_section! {
    TS_DICT_RESULT,
    TS_DICT_RESULT_SECTION,
    r#"
/** One heading in the standard dictionary. */
export interface DictHeading {
  name: string;
  /** `KEY` | `REQUIRED` | `OTHER` — whether the AGS standard requires it. */
  status: string;
  /** AGS TYPE code (`ID`, `X`, `2DP`, `DT`, …). */
  type: string;
  /** Absent when the heading is unitless — not `""`. */
  unit?: string;
  description: string;
}

/** One group in the standard dictionary. */
export interface DictGroup {
  code: string;
  /** The group's standard description — its "contents". */
  contents: string;
  /** Absent for a root group (`PROJ`). */
  parent?: string;
  headings: DictHeading[];
}

/** One bundled edition of the AGS4 standard dictionary: groups sorted by code,
 *  each group's headings in canonical dictionary order. */
export interface StandardDict {
  /** The edition this is for (`"4.1.1"`, …). */
  ags_edition: string;
  groups: DictGroup[];
}
"#
}

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(typescript_type = "StandardDict")]
    pub type StandardDictJs;
}

/// Serialise the bundled standard dictionary for `dict_version`
/// (`None`/`"auto"` → the [`FALLBACK`] edition; else `4.0.3|4.0.4|4.1|4.1.1|
/// 4.2`). Groups are sorted by code; each group's headings keep the canonical
/// dictionary order. Returns the web reference UI's `{ags_edition, groups:[…]}`
/// shape — built by the shared `dict::dictionary_dto` (#294 F#6), the same
/// source `laterite.registry.dictionary()` and Node's render.
#[wasm_bindgen]
pub fn dictionary(dict_version: Option<String>) -> Result<StandardDictJs, JsError> {
    console_error_panic_hook::set_once();
    let dto = dictionary_core(dict_version.as_deref()).map_err(|m| JsError::new(&m))?;
    to_js(&dto)
}

/// The host-testable core of [`dictionary`]: resolve the edition (`None`/`auto`
/// → [`FALLBACK`]) and build the shared DTO.
fn dictionary_core(
    dict_version: Option<&str>,
) -> Result<laterite_ags4_validator::dict::DictionaryDto, String> {
    let version = resolve_dict_override(dict_version)?.unwrap_or(FALLBACK);
    Ok(laterite_ags4_validator::dict::dictionary_dto(version))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testdata::err;
    use laterite_ags4_validator::{DictVersion, dict::Dictionary};

    // --- dictionary() data source ---
    // The JsValue wrapper can't be built off-wasm, so assert the per-edition
    // standard dictionary the export serialises actually carries real names +
    // descriptions (the scaffolded merged JSON the UI used to fetch did not).

    #[test]
    fn bundled_dictionary_has_real_names_and_descriptions() {
        let d = Dictionary::bundled(DictVersion::V4_1_1);
        let codes: Vec<&str> = d.group_codes().collect();
        assert!(codes.contains(&"LOCA"), "LOCA must be a standard group");
        assert!(
            !d.group("LOCA").unwrap().desc.trim().is_empty(),
            "LOCA must have a real group description",
        );
        let e = d.heading("LOCA", "LOCA_ID").expect("LOCA_ID heading");
        assert_eq!(e.status, "KEY");
        assert_eq!(e.ags_type, "ID");
        assert!(
            !e.desc.trim().is_empty(),
            "LOCA_ID must have a real description, got {:?}",
            e.desc,
        );
    }

    #[test]
    fn bundled_dictionary_differs_across_editions() {
        // 4.2 added groups over 4.0.3 — the per-edition dicts are not identical,
        // which is the whole point of making the browser edition-selectable.
        let n_403 = Dictionary::bundled(DictVersion::V4_0_3)
            .group_codes()
            .count();
        let n_42 = Dictionary::bundled(DictVersion::V4_2).group_codes().count();
        assert!(n_403 > 0 && n_42 > 0);
        assert!(
            n_42 >= n_403,
            "4.2 should have at least as many groups as 4.0.3"
        );
    }

    // ---------------------------------------------------------------
    // dictionary_core
    // ---------------------------------------------------------------

    #[test]
    fn dictionary_defaults_to_the_fallback_edition() {
        let dto = dictionary_core(None).expect("default");
        assert_eq!(
            dto.ags_edition,
            dictionary_core(Some("auto")).unwrap().ags_edition
        );
        assert!(!dto.groups.is_empty(), "the dictionary must have groups");
    }

    #[test]
    fn dictionary_refuses_an_unknown_edition() {
        let msg = err(dictionary_core(Some("4.9")));
        assert!(msg.contains("4.9") || msg.contains("unknown"), "got: {msg}");
    }

    #[test]
    fn each_edition_returns_its_own_dictionary() {
        // A resolver that ignored its argument would return the fallback for
        // every edition and pass any single-edition assertion.
        let a = dictionary_core(Some("4.0.3")).expect("4.0.3");
        let b = dictionary_core(Some("4.2")).expect("4.2");
        assert_ne!(a.ags_edition, b.ags_edition);
        assert!(
            a.groups.len() != b.groups.len()
                || a.groups.iter().map(|g| g.headings.len()).sum::<usize>()
                    != b.groups.iter().map(|g| g.headings.len()).sum::<usize>(),
            "4.0.3 and 4.2 returned identical dictionaries"
        );
    }

    #[test]
    fn a_dictionary_group_carries_the_descriptions_the_reference_ui_needs() {
        // The reason this door exists: the Tools reference used to fetch a
        // scaffolded dictionary where ~91% of headings had EMPTY descriptions.
        let dto = dictionary_core(Some("4.1.1")).expect("4.1.1");
        let loca = dto.groups.iter().find(|g| g.code == "LOCA").expect("LOCA");
        assert!(!loca.contents.is_empty(), "the group needs its description");
        assert!(
            loca.headings
                .iter()
                .filter(|h| !h.description.is_empty())
                .count()
                > loca.headings.len() / 2,
            "most headings should carry a description"
        );
    }
}
