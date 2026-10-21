# laterite-cliutil

The terminal presentation the **laterite** command-line tools share, so they
look and behave like one tool: the table grid, the progress spinner, the colour
gate, coloured JSON, atomic file writes, and the report contract every command
document renders through.

```rust
use laterite_cliutil::{colour_enabled, styled_table};

fn main() {
    // Colour follows NO_COLOR and whether stdout is a terminal.
    let colour = colour_enabled(false);
    let table = styled_table(
        &["Rule", "Findings"],
        vec![vec!["AGS Format Rule 8".into(), "3".into()]],
        colour,
    );
    println!("{table}");
}
```

The output contract is the same in every tool: results go to stdout and
progress to stderr, and a piped run gets newline-delimited JSON with no flag
asked for. The `report` module carries it, as a `Report` trait each command
document implements, with table, JSON and ndjson renderings of one value.

`clap` is optional: the `clap` feature derives `ValueEnum` on `OutputMode` for
tools that parse their arguments with it, and a tool that does not never links
it.

<!-- BEGIN GENERATED: availability — DO NOT EDIT BY HAND. Regenerate: uv run --no-project python tools/gen_crate_graph.py -->
> **Engine crate, not a door.** `laterite-cliutil` is machinery inside the laterite
> toolchain, reshaped whenever the toolchain needs it. The Rust door is
> [`laterite`](https://crates.io/crates/laterite); depend on this one directly
> only if that suits you, and expect it to move.

## Install it

```bash
cargo add laterite-cliutil
```

This crate versions independently of the engine.
<!-- END GENERATED: availability -->
