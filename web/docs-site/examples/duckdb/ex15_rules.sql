-- Inspect the bundled numbered-rule catalogue — the AGS4 validation rules the
-- library/CLI enforce. (The extension is read-only and does not RUN them; this
-- just lists them, incl. sub-rules like 2a/10a, with severity + fixability.)
-- expect-rows: 27
SELECT rule, title, severity, fixable FROM ags_rules() ORDER BY rule;
