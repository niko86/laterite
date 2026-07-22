# Canonicalise a GitHub ruleset JSON so the drift check compares GOVERNANCE, not
# noise. Strip volatile / caller-dependent fields, and sort every collection
# whose order the API does not guarantee — so a real change (a required check
# added/removed, enforcement dropped, admin-bypass widened, the protected branch
# swapped) is the ONLY thing that can diff against the committed snapshot.
#
# Used identically to (a) generate .github/rulesets/main-protection.expected.json
# and (b) normalise the live ruleset in the nightly `ruleset-drift` job. Run with
# `jq -S -f normalize.jq` (the -S sorts object keys after this transform).
del(
  ._links,              # HATEOAS URLs, not config
  .created_at,          # timestamps
  .updated_at,
  .node_id,             # GraphQL node id, opaque
  .current_user_can_bypass  # depends on WHO reads it — differs local vs CI token
)
| .bypass_actors |= sort_by(.actor_type, .actor_id, .bypass_mode)
| .conditions.ref_name.include |= sort
| .conditions.ref_name.exclude |= sort
| .rules |= (
    sort_by(.type)
    | map(
        if .type == "required_status_checks"
        # the required-check list is the crux — sort it so ordering can't diff
        then .parameters.required_status_checks |= sort_by(.context)
        else .
        end
      )
  )
