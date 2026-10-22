//! A vouched `validate(path).index(..)` holds a buffer, not the file (#1131).
//!
//! Its own test binary because it installs a counting global allocator, which
//! would see every other test's allocations if it shared a binary with them.

use std::alloc::{GlobalAlloc, Layout, System};
use std::fs;
use std::sync::atomic::{AtomicUsize, Ordering};

use laterite::ags4;

/// The system allocator, keeping the live total and its high-water mark.
struct Counting;

static LIVE: AtomicUsize = AtomicUsize::new(0);
static PEAK: AtomicUsize = AtomicUsize::new(0);

// SAFETY: every call is forwarded unchanged to `System`; the counters only observe.
unsafe impl GlobalAlloc for Counting {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        // SAFETY: the caller's contract for `alloc` is passed straight through.
        let p = unsafe { System.alloc(layout) };
        if !p.is_null() {
            let now = LIVE.fetch_add(layout.size(), Ordering::SeqCst) + layout.size();
            PEAK.fetch_max(now, Ordering::SeqCst);
        }
        p
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        // SAFETY: as above.
        unsafe { System.dealloc(ptr, layout) };
        LIVE.fetch_sub(layout.size(), Ordering::SeqCst);
    }
}

#[global_allocator]
static ALLOC: Counting = Counting;

/// The most memory `f` had live at once beyond what was live when it started.
fn peak_during<T>(f: impl FnOnce() -> T) -> (T, usize) {
    let base = LIVE.load(Ordering::SeqCst);
    PEAK.store(base, Ordering::SeqCst);
    let out = f();
    (out, PEAK.load(Ordering::SeqCst) - base)
}

const FILE_MIB: usize = 8;

/// Error-clean AGS4 of `FILE_MIB` MiB, so it can be certified: the size is one
/// long `PROJ_NAME`, which keeps the mint's engine run short.
fn big_clean() -> String {
    let name = "A".repeat(FILE_MIB << 20);
    [
        "\"GROUP\",\"PROJ\"",
        "\"HEADING\",\"PROJ_ID\",\"PROJ_NAME\"",
        "\"UNIT\",\"\",\"\"",
        "\"TYPE\",\"ID\",\"X\"",
        &format!("\"DATA\",\"P1\",\"{name}\""),
        "\"GROUP\",\"TRAN\"",
        "\"HEADING\",\"TRAN_ISNO\",\"TRAN_DATE\",\"TRAN_PROD\",\"TRAN_STAT\",\"TRAN_AGS\",\"TRAN_RECV\"",
        "\"UNIT\",\"\",\"yyyy-mm-dd\",\"\",\"\",\"\",\"\"",
        "\"TYPE\",\"X\",\"DT\",\"X\",\"X\",\"X\",\"X\"",
        "\"DATA\",\"1\",\"2026-08-05\",\"Producer\",\"Draft\",\"4.1.1\",\"Recipient\"",
        "\"GROUP\",\"UNIT\"",
        "\"HEADING\",\"UNIT_UNIT\",\"UNIT_DESC\"",
        "\"UNIT\",\"\",\"\"",
        "\"TYPE\",\"X\",\"X\"",
        "\"DATA\",\"yyyy-mm-dd\",\"Date\"",
        "\"GROUP\",\"TYPE\"",
        "\"HEADING\",\"TYPE_TYPE\",\"TYPE_DESC\"",
        "\"UNIT\",\"\",\"\"",
        "\"TYPE\",\"X\",\"X\"",
        "\"DATA\",\"ID\",\"Unique identifier\"",
        "\"DATA\",\"X\",\"Text\"",
        "\"DATA\",\"DT\",\"Date\"",
        "",
    ]
    .join("\r\n")
}

#[test]
fn a_vouched_validate_holds_a_buffer_not_the_file() {
    let dir = std::env::temp_dir().join(format!("laterite-vouched-{}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();
    let ags = dir.join("big.ags");
    fs::write(&ags, big_clean()).unwrap();
    let size = usize::try_from(fs::metadata(&ags).unwrap().len()).unwrap();
    let cert = ags4::read(&ags)
        .run()
        .unwrap()
        .certify()
        .to_bytes()
        .unwrap();

    let (report, vouched_peak) =
        peak_during(|| ags4::validate(&ags).index_bytes(cert.clone()).run());
    assert!(report.unwrap().certified(), "the cert should vouch");

    // The positive control: the same file, but an edition the cert was not judged
    // under, so the engine runs and the bytes must be read whole. If the instrument
    // cannot see that, it cannot see the absence of it either.
    let (report, engine_peak) = peak_during(|| {
        ags4::validate(&ags)
            .index_bytes(cert.clone())
            .edition("4.1")
            .run()
    });
    assert!(!report.unwrap().certified());
    assert!(
        engine_peak >= size,
        "the control should hold the whole file: {engine_peak} < {size}"
    );

    assert!(
        vouched_peak < size / 16,
        "a vouched validate held {vouched_peak} bytes for a {size}-byte file"
    );
    let _ = fs::remove_dir_all(&dir);
}
