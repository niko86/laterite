# Support

## laterite is in beta

Beta is a statement about **how much real-world use laterite has had**, not about
how much testing it has had. Those are different things, and only one of them can be
fixed from inside the repo.

The engine implements the AGS4 numbered rules from the published specification,
cross-checked against a real corpus, with every read surface asserted to report byte-identical
findings on each PR. What it has not had is your files. That is what beta is for, and
[the feedback loop](../feedback.md) is how it ends.

### What it means in practice

**The API can still change.** Beta is when the shape of things is cheap to change,
and a report that says "I wanted it to do X and it wouldn't" is a reason to change
it. That is the churn you should expect. [How versions move](#how-versions-move) is
the rule that bounds it, and tells you which upgrades are the ones to look at.

**What it runs on is not churning.** Platform and version floors are deliberately the
steadier promise of the two. Expect movement in the API; don't expect it underfoot.

**It is not a quality tier.** There is no "beta parts" and "stable parts": one
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
| **Rust** | `cargo add laterite` | ✅ |

The engine crates the surfaces are built from are published to crates.io and are part
of the same source tree, so the same statement covers them. But they are machinery,
not a door, and you are not expected to depend on them directly.

### How versions move

"Expect the API to change" is a warning, not a rule. This is the rule. The two
version lines above answer it differently, because they are two different promises.

**The product line (Python, Node, browser, DuckDB, `lat`, and the Rust crate) is one
number and one changelog.** While it is pre-1.0, a **breaking change takes the minor** and everything
else takes the patch. So a patch is safe to take, and a minor is the one to look at
before you take it. Which minors those were is the first thing in the changelog:
[an index of every breaking change](https://github.com/niko86/laterite/blob/main/CHANGELOG.md#breaking-changes),
one line each with the version it landed in, so the answer is a short list rather than
a read.

Whether an upgrade happens *to* you is your package manager's business, not ours. npm
and Cargo both treat a `0.x` minor as a wall once the requirement is written into your
manifest; `pip install laterite` has no equivalent and takes the newest release every
time. Pin it or lock it if you want that to be your decision.

**The engine crates make no version promise at all, and they are on their own line.**
Said above and worth saying as a rule: they are reshaped whenever the toolchain needs
it, so any release of one can move anything. Each says so on its own crates.io page.
Their version is **not** the product's either, so a number you see on crates.io will
not match the one in a release announcement. That is two clocks, not a mistake.

### When beta ends

At **1.0.0**, which is a declaration in the same way beta is, not a bar the code
clears. The signal being watched is a change of character in what comes back: inbound
traffic turning from "this is broken" into "can it also do X". That gets read at a
review **twelve months after the beta announcement**, with an earlier checkpoint at
three months that 1.0 can be pulled forward on.

Silence ships 1.0 anyway. Zero adoption is information about reach, not about
readiness, and a version number held hostage to a user who never arrived is just 0.x
forever under another name.

When it happens, the whole product line moves together: the Python wheel, the Node
and browser packages, the DuckDB extension, the `lat` binary and the Rust crate
already share one version number.
