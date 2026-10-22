//! `lat validate --index` with a cert that vouches holds a buffer, not the file (#1131).
//!
//! The binary is the unit here, so the instrument is one child's own peak resident
//! set, read back from `wait4`. A counting allocator cannot see into another process,
//! and the peak over all of this process's children would include every other `lat`
//! a test had run.

#![cfg(unix)]

use std::ffi::OsStr;
use std::io::Read as _;
use std::path::Path;
use std::process::{Command, Stdio};

/// Run `lat`; return its exit code, its stderr, and its peak resident set in bytes.
// The child IS reaped, by `wait4` below: std's `wait` would reap it first and leave
// `wait4` nothing to report on.
#[allow(clippy::zombie_processes)]
fn lat_peak<S: AsRef<OsStr>>(args: &[S]) -> (i32, String, usize) {
    let mut child = Command::new(env!("CARGO_BIN_EXE_lat"))
        .args(args)
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .spawn()
        .expect("spawn lat");
    let mut err = String::new();
    child
        .stderr
        .take()
        .expect("piped")
        .read_to_string(&mut err)
        .expect("stderr");
    let pid = libc::pid_t::try_from(child.id()).expect("pid");
    let mut status = 0;
    // SAFETY: `rusage` is a plain C struct, valid all-zero, which `wait4` fills in.
    let mut usage: libc::rusage = unsafe { std::mem::zeroed() };
    // SAFETY: `pid` is this process's own child, not yet reaped (std only reaps on
    // `wait`, which is never called), and both out-pointers are live locals.
    let reaped = unsafe { libc::wait4(pid, &raw mut status, 0, &raw mut usage) };
    assert_eq!(reaped, pid, "wait4 failed");
    assert!(libc::WIFEXITED(status), "lat did not exit normally");
    // `ru_maxrss` is in bytes on macOS and in KiB on Linux.
    let unit = if cfg!(target_os = "macos") { 1 } else { 1024 };
    let peak = usize::try_from(usage.ru_maxrss).expect("rss") * unit;
    (libc::WEXITSTATUS(status), err, peak)
}

const FILE_MIB: usize = 32;

/// Error-clean AGS4 of `FILE_MIB` MiB, so it can be certified: the size is one long
/// `PROJ_NAME`, which keeps the engine runs short.
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
fn a_vouched_index_holds_a_buffer_not_the_file() {
    let dir = std::env::temp_dir().join(format!("lat_vouched_{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).expect("scratch");
    let ags = dir.join("big.ags");
    let idx = dir.join("big.ags.idx");
    std::fs::write(&ags, big_clean()).expect("write");
    let size = usize::try_from(std::fs::metadata(&ags).expect("stat").len()).expect("size");
    let (code, err, _) = lat_peak(&[OsStr::new("certify"), ags.as_os_str()]);
    assert_eq!(code, 0, "certify: {err}");
    assert!(
        Path::new(&idx).exists(),
        "certify wrote no {}",
        idx.display()
    );

    // The binary's own floor: what `lat` holds having read no file at all.
    let (_, _, idle) = lat_peak(&["rules"]);

    let (code, err, vouched) = lat_peak(&[
        OsStr::new("validate"),
        ags.as_os_str(),
        OsStr::new("--index"),
        idx.as_os_str(),
    ]);
    assert_eq!(code, 0, "{err}");
    assert!(
        err.contains("rule engine skipped"),
        "the cert should vouch: {err}"
    );

    // The positive control: the same file with no cert, so the engine runs and the
    // bytes must be held. If the instrument cannot see that, it cannot see the
    // absence of it either.
    let (code, err, engine) = lat_peak(&[OsStr::new("validate"), ags.as_os_str()]);
    assert_eq!(code, 0, "{err}");
    assert!(
        engine >= size,
        "the control should hold the whole file: {engine} < {size}"
    );

    assert!(
        vouched < idle + size / 4,
        "a vouched --index peaked at {vouched} bytes for a {size}-byte file \
         ({idle} with no file)"
    );
    let _ = std::fs::remove_dir_all(&dir);
}
