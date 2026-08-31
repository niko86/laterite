//! `ParseOptions::locate_only` — the cert/index profile.
//!
//! The profile exists to skip work, so the only thing worth testing is that it
//! skips nothing that MATTERS: every answer `index_ags4_bytes` reads must be
//! identical to the full walk's, and every input the full walk rejects must
//! still be rejected. A faster locator that disagrees with the reader would
//! mint certs pointing at the wrong bytes while reporting success.

/// The locator profile must be a pure SUBSET of the full walk, never a
/// different answer. Everything `index_ags4_bytes` reads — every GROUP record
/// with its byte offset and line, the first-seen-wins order, the total size
/// and the source-true flag — has to come back byte-identical, or a cert built
/// from it points at the wrong place while claiming to be valid.
#[test]
fn locate_only_agrees_with_the_full_walk() {
    use laterite_ags4_parse::{ParseOptions, parse_bytes_opts};

    let cases: [&[u8]; 6] = [
        b"\"GROUP\",\"PROJ\"\r\n\"HEADING\",\"PROJ_ID\"\r\n\"DATA\",\"1\"\r\n",
        // A redeclared group: the locator must keep BOTH records, in order.
        b"\"GROUP\",\"LOCA\"\r\n\"DATA\",\"a\"\r\n\"GROUP\",\"LOCA\"\r\n\"DATA\",\"b\"\r\n",
        // Descriptor rows before any GROUP — skipped by the lenient walk.
        b"\"HEADING\",\"X\"\r\n\"GROUP\",\"PROJ\"\r\n",
        // Blank lines and a lone-LF terminator must not shift offsets.
        b"\"GROUP\",\"PROJ\"\n\n\"DATA\",\"1\"\n",
        // A BOM: byte 0 is still byte 0.
        b"\xef\xbb\xbf\"GROUP\",\"PROJ\"\r\n\"DATA\",\"1\"\r\n",
        // An unterminated final line.
        b"\"GROUP\",\"PROJ\"\r\n\"DATA\",\"1\"",
    ];

    for bytes in cases {
        let full = parse_bytes_opts(bytes, ParseOptions::lean()).expect("full walk");
        let located = parse_bytes_opts(
            bytes,
            ParseOptions {
                locate_only: true,
                ..ParseOptions::lean()
            },
        )
        .expect("locate walk");

        // The retains-nothing contract: the locator keeps neither the decoded
        // buffer nor the raw-line overlay — the whole reason the profile exists.
        assert!(located.text.is_empty(), "locate_only must retain no text");
        assert!(
            located.raw_lines.is_empty(),
            "locate_only must retain no raw lines"
        );
        assert_eq!(located.group_records, full.group_records, "records");
        assert_eq!(located.group_order, full.group_order, "order");
        assert_eq!(located.total_bytes, full.total_bytes, "total_bytes");
        assert_eq!(located.total_lines, full.total_lines, "total_lines");
        assert_eq!(located.has_bom, full.has_bom, "has_bom");
        assert_eq!(
            located.byte_offsets_source_true, full.byte_offsets_source_true,
            "source_true"
        );
        // Group identity + location match; only the row MODEL is absent.
        let full_keys: Vec<_> = full.groups.keys().collect();
        let loc_keys: Vec<_> = located.groups.keys().collect();
        assert_eq!(loc_keys, full_keys, "group keys");
        for (code, g) in &located.groups {
            assert_eq!(g.group_byte, full.groups[code].group_byte, "group_byte");
            assert_eq!(g.group_line, full.groups[code].group_line, "group_line");
            assert!(g.rows.is_empty() && g.headings.is_empty(), "model skipped");
        }
    }
}

/// Both profiles must reject the same inputs for the same reasons — a locator
/// that silently accepts a file the reader rejects is worse than a slow one.
#[test]
fn locate_only_rejects_what_the_full_walk_rejects() {
    use laterite_ags4_parse::{ParseError, ParseOptions, parse_bytes_opts};

    let locate = ParseOptions {
        locate_only: true,
        ..ParseOptions::lean()
    };

    // No GROUP rows at all.
    for bytes in [&b"\"DATA\",\"1\"\r\n"[..], &b""[..]] {
        assert!(matches!(
            parse_bytes_opts(bytes, locate).unwrap_err(),
            ParseError::NotAgs4(_)
        ));
    }
    // AGS3 markers are sniffed from the TAG, which the locator still reads.
    assert!(matches!(
        parse_bytes_opts(&b"\"**PROJ\"\r\n\"*ID\"\r\n"[..], locate).unwrap_err(),
        ParseError::UnsupportedEdition { .. }
    ));
    // Invalid UTF-8 under Reject.
    assert!(matches!(
        parse_bytes_opts(&b"\"GROUP\",\"PR\xffJ\"\r\n"[..], locate).unwrap_err(),
        ParseError::NotUtf8
    ));
}
