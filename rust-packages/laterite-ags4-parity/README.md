# laterite-ags4-parity

The comparison between **laterite**'s AGS4 validator and
[python-ags4](https://gitlab.com/ags-data-format-wg/ags-python-library): one
verdict model for "did the two engines agree on this file", and the bridge that
asks python-ags4.

```rust
use std::collections::BTreeSet;

use laterite_ags4_parity::{Parity, RustResult, classify};

fn main() {
    let rules: BTreeSet<String> = ["AGS Format Rule 8".to_string()].into_iter().collect();

    // Both engines raised the same rule: the files agree.
    let rust = RustResult::Rules(rules.clone());
    assert_eq!(classify(&rust, &Ok(rules)), Parity::Agree);

    // Only laterite raised it: a divergence, named by side.
    let verdict = classify(&rust, &Ok(BTreeSet::new()));
    assert!(matches!(verdict, Parity::RustOnlyRules { .. }));
}
```

The verdict is about **which rules fired**, never how many times: the two
validators split one defect across rules differently, so a count comparison
would report divergences that are not there. Where they differ by design, the
verdict says `KnownDivergence` and names the catalogued observation rather than
reporting it as a disagreement.

`PyOracle` runs python-ags4 through a wrapper script under `uv run`. The caller
passes the script and the uv project whose environment runs it; this crate
assumes no repository layout. `check` runs both engines over files or a
directory and renders the result through
[`laterite-cliutil`](https://crates.io/crates/laterite-cliutil)'s report
contract.

<!-- BEGIN GENERATED: availability — DO NOT EDIT BY HAND. Regenerate: uv run --no-project python tools/gen_crate_graph.py -->
> **Engine crate, not a door.** `laterite-ags4-parity` is machinery inside the laterite
> toolchain, reshaped whenever the toolchain needs it. The Rust door is
> [`laterite`](https://crates.io/crates/laterite); depend on this one directly
> only if that suits you, and expect it to move.

## Install it

```bash
cargo add laterite-ags4-parity
```

This crate versions independently of the engine.
<!-- END GENERATED: availability -->
