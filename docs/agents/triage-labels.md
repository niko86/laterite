# Triage Labels

The skills speak in terms of five canonical triage roles. This file maps those
roles to the actual label strings used in this repo's issue tracker.

| Label in mattpocock/skills | Label in our tracker | Meaning                                  |
| -------------------------- | -------------------- | ---------------------------------------- |
| `needs-triage`             | `needs-triage`       | Maintainer needs to evaluate this issue  |
| `needs-info`               | `needs-info`         | Waiting on reporter for more information |
| `ready-for-agent`          | `ready-for-agent`    | Fully specified, ready for an AFK agent  |
| `ready-for-human`          | `ready-for-human`    | Requires human implementation            |
| `wontfix`                  | `wontfix`            | Will not be actioned                     |

When a skill mentions a role (e.g. "apply the AFK-ready triage label"), use the
corresponding label string from this table.

Edit the right-hand column to match whatever vocabulary you actually use.

## Which of these exist on the repo today

All five. `wontfix` is one of GitHub's stock labels ("This will not be worked
on"); the other four were created for triage, each with the meaning in the table
above as its description. Don't create any of them again — `gh label create`
fails on a duplicate.

## Issues outside the state machine

`tracking` marks an issue that is not itself a ticket: an epic whose children
are the tickets, a standing ledger (the wiki small-writes ledger), or a
bot-maintained status issue (the nightly engine-release tracker). It takes no
category or state role, and a triage sweep should skip it rather than report it
as unlabeled.

Nothing else in the label set (`bug`, `enhancement`, `documentation`,
`question`, the Dependabot ecosystem labels, `no-changelog`) plays a triage role.
