//! A vouched check through [`check_reader`] holds a buffer, not the file (#1097).
//!
//! Its own test binary because it installs a counting global allocator, which
//! would see every other test's allocations if it shared a binary with them.

use std::alloc::{GlobalAlloc, Layout, System};
use std::sync::atomic::{AtomicUsize, Ordering};

use laterite_ags4_trust::{
    EditionInput, ReaderRequest, Sidecar, TierCoverage, ValidationStamp, check_reader, engine_id,
};
use laterite_ags4_validator::{CheckOptions, DictResolution, WorldScope};

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

/// A file of `FILE_MIB` MiB the byte index accepts: one group, a lot of rows.
fn big_file() -> Vec<u8> {
    let mut bytes = b"\"GROUP\",\"PROJ\"\r\n\"HEADING\",\"PROJ_ID\"\r\n\
                      \"UNIT\",\"\"\r\n\"TYPE\",\"ID\"\r\n"
        .to_vec();
    let row = b"\"DATA\",\"P1\"\r\n";
    while bytes.len() < FILE_MIB << 20 {
        bytes.extend_from_slice(row);
    }
    bytes
}

/// A cert for `bytes` that measured every tier and found errors and FYI clean, and
/// never measured warnings. So it vouches for an errors-only request, and a request
/// for warnings has to run the engine.
fn cert_for(bytes: &[u8]) -> Sidecar {
    let id = engine_id(None);
    let stamp = ValidationStamp {
        validator: id.validator,
        engine: id.fingerprint,
        compat: None,
        checked_at: "2026-01-01T00:00:00Z".to_string(),
        edition: EditionInput::Auto {
            resolved: "4.2".to_string(),
            resolution: DictResolution::ExactTranAgs,
        },
        encoding: "UTF-8".to_string(),
        custom_dict: None,
        errors: TierCoverage::Measured { count: 0 },
        warnings: TierCoverage::NotMeasured,
        fyi: TierCoverage::Measured { count: 0 },
    };
    Sidecar::assemble(bytes, stamp).expect("indexes")
}

#[test]
fn a_vouched_check_holds_a_buffer_not_the_file() {
    let bytes = big_file();
    let cert = cert_for(&bytes);
    let size = bytes.len();

    let (out, vouched_peak) = peak_during(|| {
        check_reader(ReaderRequest {
            file: std::io::Cursor::new(&bytes[..]),
            opts: &CheckOptions::default(),
            cert: Some(&cert),
            world: WorldScope::None,
            compat: None,
            named: true,
        })
    });
    let out = out.expect("checks");
    assert!(out.certified, "the cert vouches for an errors-only request");

    // The positive control: the same file, but a question the cert cannot answer, so
    // the engine runs and the bytes must be read whole. If the instrument cannot see
    // that, it cannot see the absence of it either.
    let mut warnings = CheckOptions::default();
    warnings.include_warnings = true;
    let (out, engine_peak) = peak_during(|| {
        check_reader(ReaderRequest {
            file: std::io::Cursor::new(&bytes[..]),
            opts: &warnings,
            cert: Some(&cert),
            world: WorldScope::None,
            compat: None,
            named: true,
        })
    });
    assert!(!out.expect("checks").certified);
    assert!(
        engine_peak >= size,
        "the control should hold the whole file: {engine_peak} < {size}"
    );

    assert!(
        vouched_peak < size / 16,
        "a vouched check held {vouched_peak} bytes for a {size}-byte file"
    );
}
