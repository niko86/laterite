# Support

## laterite is in beta

Beta is a statement about **how much real-world use laterite has had** — not about
how much testing it has had. Those are different things, and only one of them can be
fixed from inside the repo.

The engine is a clean-room implementation of the AGS4 numbered rules, cross-checked
against a real corpus, with every read surface asserted to report byte-identical
findings on each PR. What it has not had is your files. That is what beta is for, and
[the feedback loop](../feedback.md) is how it ends.

### What it means in practice

**The API can still change.** Beta is when the shape of things is cheap to change,
and a report that says "I wanted it to do X and it wouldn't" is a reason to change
it. That is the churn you should expect.

**What it runs on is not churning.** Platform and version floors are deliberately the
steadier promise of the two. Expect movement in the API; don't expect it underfoot.

**It is not a quality tier.** There is no "beta parts" and "stable parts" — one
engine compiles to every surface, and a rule behaves the same everywhere by test, not
by assertion. Beta applies to the whole project or none of it.

### What carries the label

| Surface | Install | In beta |
| --- | --- | :---: |
| **Python** | `pip install laterite` | ✅ |
| **Node** | `npm i laterite` | ✅ |
| **Browser** | `npm i @laterite/ags4-wasm` | ✅ |
| **DuckDB** | `INSTALL laterite_ags4 FROM community` | ✅ |
| **CLI** | the [`lat` binary](https://github.com/niko86/laterite/releases) | ✅ |
| **Rust** | `cargo add laterite` | — |

**The Rust crate is the one exception**, and the reason is completeness rather than
quality: it is not yet at parity with the other surfaces, and its API will change more
than theirs. It runs the same engine. It joins the claim when it is finished.

The engine crates the surfaces are built from are published to crates.io and are part
of the same source tree, so the same statement covers them — but they are machinery,
not a door, and you are not expected to depend on them directly.

### When beta ends

At **1.0.0**, which is a declaration in the same way beta is — not a bar the code
clears. The signal being watched is a change of character in what comes back: inbound
traffic turning from "this is broken" into "can it also do X". That gets read at a
review **twelve months after the beta announcement**, with an earlier checkpoint at
three months that 1.0 can be pulled forward on.

Silence ships 1.0 anyway. Zero adoption is information about reach, not about
readiness, and a version number held hostage to a user who never arrived is just 0.x
forever under another name.

When it happens, the whole product line moves together — the Python wheel, the Node
and browser packages, the DuckDB extension and the `lat` binary already share one
version number. The Rust crate keeps its own clock and is not carried along by it.
