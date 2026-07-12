-- Recipe: deepest sample + its plasticity per borehole. A three-group walk
-- (LOCA -> SAMP -> LLPL) joined on the content keys _parent_id/_id; arg_max
-- reads llpl_pi from the deepest sample's plasticity test in one pass.
-- expect-rows: 14
SELECT l.loca_id,
       max(s.samp_top)                AS deepest,
       arg_max(t.llpl_pi, s.samp_top) AS pi_at_deepest
FROM read_ags('examples/sample_site.ags', 'LOCA') l
JOIN read_ags('examples/sample_site.ags', 'SAMP') s ON s._parent_id = l._id
JOIN read_ags('examples/sample_site.ags', 'LLPL') t ON t._parent_id = s._id
GROUP BY l.loca_id ORDER BY l.loca_id;
