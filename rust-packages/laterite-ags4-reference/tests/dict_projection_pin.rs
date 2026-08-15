//! Pin what the compiled dictionary tables PROJECT, not just what the JSON says.
//!
//! `tests/test_dictionary_faithful.py` already guards the input: it re-runs
//! `tools/gen_dictionary.py` and asserts the committed `ags_dictionary.json`
//! matches the five official `.ags` files. Nothing guarded the **projection** —
//! the per-edition tables `build.rs` emits from that union and every rule module
//! reads. A change to the codegen could alter what an edition believes about a
//! heading's TYPE or a group's parent with both the JSON gate and the whole test
//! suite still green, because the suite exercises rules, not the dictionary's
//! full surface.
//!
//! That gap matters the moment the representation changes. The tables are five
//! complete `phf` projections of a union whose editions overlap almost entirely,
//! and repacking them (one union + a presence mask, interned field indices) is
//! worth ~45% of the wasm binary — but it is only safe if "the projection is
//! unchanged" is something a machine asserts rather than something a reviewer
//! believes after reading a diff of generated code.
//!
//! So: hash every observable the bundled arm exposes, per edition, and pin it.
//! A repack that preserves behaviour reproduces these exactly; one that does not
//! names the edition and the table it broke.
//!
//! Set `DICT_PIN_DUMP=<path>` to write the full pre-hash text instead of just
//! comparing digests — two dumps diff line-by-line, which is how you find *which*
//! heading moved rather than only that one did.

use std::fmt::Write as _;

use laterite_ags4_reference::dict::{DictVersion, Dictionary};

/// FNV-1a, 64-bit. Deliberately not SHA-256: this detects accidental change in
/// a build-time projection, not tampering, and a leaf crate whose selling point
/// is a two-dependency graph should not grow a third for a test digest. It is
/// fully specified and stable across toolchains, which `DefaultHasher` is not —
/// `SipHash`'s output is explicitly not guaranteed stable across Rust releases, so
/// a pin built on it would rot into a false failure on a compiler bump.
fn fnv1a(s: &str) -> u64 {
    let mut h: u64 = 0xcbf2_9ce4_8422_2325;
    for b in s.as_bytes() {
        h ^= u64::from(*b);
        h = h.wrapping_mul(0x1000_0000_01b3);
    }
    h
}

/// Every observable of one edition's bundled tables, in a deterministic order.
///
/// Sorted at every level: `phf` iteration order is an implementation detail of
/// the hash function and would make the digest depend on the very thing being
/// changed.
fn project(v: DictVersion) -> String {
    let d = Dictionary::bundled(v);
    let mut out = String::new();

    writeln!(out, "edition\t{}", v.as_str()).unwrap();
    writeln!(out, "tran_ags\t{}", d.tran_ags()).unwrap();

    let mut codes: Vec<&str> = d.group_codes().collect();
    codes.sort_unstable();
    writeln!(out, "groups\t{}", codes.len()).unwrap();

    for code in &codes {
        let g = d.group(code).unwrap_or_else(|| {
            panic!(
                "{}: group {code} in group_codes but not group()",
                v.as_str()
            )
        });
        writeln!(out, "G\t{code}\t{}\t{}", g.parent, g.desc).unwrap();

        // Heading ORDER is Rule 7's canonical order — a separate table from the
        // heading map, and one a repack could plausibly disturb. Pinned as
        // emitted, NOT sorted.
        let order = d.group_headings(code);
        writeln!(out, "O\t{code}\t{}", order.join(",")).unwrap();

        for h in order.iter() {
            match d.heading(code, h) {
                Some(e) => writeln!(
                    out,
                    "H\t{code}\t{h}\t{}\t{}\t{}\t{}",
                    e.ags_type, e.unit, e.status, e.desc
                )
                .unwrap(),
                // Not an assertion failure: pin the absence, so a repack that
                // starts or stops resolving it shows up as a diff either way.
                None => writeln!(out, "H\t{code}\t{h}\t<ABSENT>").unwrap(),
            }
        }
    }

    // The ABBR pick-list, reached through the public accessors so the pin covers
    // the door callers actually use.
    let mut abbr_hdngs: Vec<&str> = codes
        .iter()
        .flat_map(|c| d.group_headings(c).into_owned())
        .collect();
    abbr_hdngs.sort_unstable();
    abbr_hdngs.dedup();

    for hdng in &abbr_hdngs {
        let mut cs = d.abbr_codes(hdng);
        if cs.is_empty() {
            continue;
        }
        cs.sort_unstable();
        for c in cs {
            writeln!(
                out,
                "A\t{hdng}\t{c}\t{}",
                d.abbr_desc(hdng, c).unwrap_or("<NONE>")
            )
            .unwrap();
        }
    }

    out
}

/// The pinned digests. Regenerate ONLY when the dictionary data itself changes
/// (a new edition, a corrected `.ags` source) — never to make a refactor pass.
/// A repack that moves one of these has changed what an edition believes, which
/// is the entire thing this file exists to stop.
const PINNED: &[(&str, u64, usize)] = &[
    ("4.0.3", 0xf693_8b9b_c6f6_1fae, 172_169),
    ("4.0.4", 0xd63a_ece4_7983_da0b, 172_778),
    ("4.1", 0xa646_fa02_3b27_7303, 326_580),
    ("4.1.1", 0x157f_d32e_c01a_5196, 326_396),
    ("4.2", 0x2789_4454_6d5e_d299, 378_184),
];

#[test]
fn every_edition_projects_exactly_what_it_did() {
    if let Ok(path) = std::env::var("DICT_PIN_DUMP") {
        let mut all = String::new();
        for v in DictVersion::ALL {
            all.push_str(&project(*v));
        }
        std::fs::write(&path, all).expect("write dump");
        eprintln!("DICT_PIN_DUMP written to {path} — comparing digests anyway");
    }

    let mut actual: Vec<(String, u64, usize)> = Vec::new();
    for v in DictVersion::ALL {
        let text = project(*v);
        actual.push((v.as_str().to_string(), fnv1a(&text), text.len()));
    }

    let rendered: Vec<String> = actual
        .iter()
        .map(|(e, h, n)| format!("    (\"{e}\", 0x{h:016x}, {n}),"))
        .collect();

    assert_eq!(
        actual.len(),
        PINNED.len(),
        "edition count changed: {} projected, {} pinned.\nPINNED should be:\n{}",
        actual.len(),
        PINNED.len(),
        rendered.join("\n")
    );

    let mut broken = Vec::new();
    for ((ed, hash, len), (ped, phash, plen)) in actual.iter().zip(PINNED) {
        assert_eq!(
            ed, ped,
            "edition order changed: {ed} where {ped} was pinned"
        );
        if hash != phash || len != plen {
            broken.push(format!(
                "  {ed}: projected {hash:#018x} ({len} bytes), pinned {phash:#018x} ({plen} bytes)"
            ));
        }
    }

    assert!(
        broken.is_empty(),
        "the compiled dictionary projection changed for {} edition(s):\n{}\n\n\
         If the DATA changed (new edition, corrected source), update PINNED to:\n{}\n\n\
         If this is a refactor, it is not behaviour-preserving. Re-run with \
         DICT_PIN_DUMP=/tmp/after.txt on each side and diff the two dumps to find \
         which heading moved.",
        broken.len(),
        broken.join("\n"),
        rendered.join("\n")
    );
}
