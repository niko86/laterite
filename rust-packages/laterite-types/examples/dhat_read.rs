//! Heap-allocation profile of the typed read path.
//!
//! The criterion benches TIME the stages (`parse_bytes`, `build_record_batch`);
//! this attributes the allocations *inside* them, so an alloc-bound stage (worth
//! attacking — cut the allocations) can be told apart from a bandwidth/compute
//! bound one (a wall — the allocations are already few). Pure Rust (`dhat`), no
//! external profiler, deterministic per fixture.
//!
//! Run (release + arrow), one stage per invocation so each profile is isolated:
//!
//! ```text
//! cargo run --release --features arrow --example dhat_read -- <fixture> build
//! cargo run --release --features arrow --example dhat_read -- <fixture> parse
//! ```
//!
//! Each run prints a Total / at-t-gmax summary to stderr and writes
//! `dhat-heap.json` (viewable at `nnethercote.github.io/dh_view/dh_view.html`).
//! dhat instruments every allocation, so the WALL TIME of this run is meaningless
//! — only the block counts and byte totals are the measurement.

use std::hint::black_box;

use laterite_types::arrow_cols::build_record_batch;

#[global_allocator]
static ALLOC: dhat::Alloc = dhat::Alloc;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let path = args
        .get(1)
        .cloned()
        .unwrap_or_else(|| "../../output/bench-fixtures/large.ags".to_string());
    let stage = args.get(2).map_or("build", String::as_str);
    let bytes = std::fs::read(&path).unwrap_or_else(|e| panic!("read {path}: {e}"));
    eprintln!(
        "dhat: fixture {path} ({} bytes), stage={stage}",
        bytes.len()
    );

    // `parse` profiles the parse leaf's allocations; anything else (default
    // `build`) parses first unprofiled, then profiles only the typed build.
    if stage == "parse" {
        let _profiler = dhat::Profiler::new_heap();
        let pf = laterite_ags4_parse::parse_bytes(&bytes, encoding_rs::UTF_8).expect("parses");
        black_box(&pf);
    } else {
        let pf = laterite_ags4_parse::parse_bytes(&bytes, encoding_rs::UTF_8).expect("parses");
        let _profiler = dhat::Profiler::new_heap();
        for code in &pf.group_order {
            let Some(grp) = pf.groups.get(code) else {
                continue;
            };
            let batch =
                build_record_batch(&grp.headings, &grp.types, grp.rows.len(), |col, row| {
                    grp.cell(col, row)
                })
                .expect("builds");
            black_box(batch);
        }
    }
}
