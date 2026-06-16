// Link Windows-specific libraries that bundled libduckdb references but
// duckdb-sys 1.4 doesn't link automatically.
//
// `Rstrtmgr` (Windows Restart Manager API) provides RmStartSession,
// RmEndSession, RmRegisterResources, RmGetList — used by libduckdb's
// file-lock diagnostics. Without this, link.exe fails with LNK1120
// "unresolved external symbol". Newer duckdb-sys releases handle this
// in their own build.rs; we add it locally for the version range we pin.

fn main() {
    if std::env::var_os("CARGO_CFG_WINDOWS").is_some() {
        println!("cargo:rustc-link-lib=Rstrtmgr");
    }
}
