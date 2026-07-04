-- After a clean validate, certify_ags mints <path>.ags.idx beside the file —
-- the same certificate Python/Node mint via .certify() and the CLI via --emit-index.
-- expect-rows: 1
SELECT certified, groups, errors FROM certify_ags('examples/sample_site.ags');
