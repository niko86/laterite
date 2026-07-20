# Build the Rust `lat` AGS4 CLI in release mode.
#
# Windows sibling of build-rust.sh. `lat` is the shipped standalone binary —
# the small, fast AGS4 validator/reader (validate, read, fix, diff, merge,
# certify, pack/lock, …). The Python `laterite` wheel is the primary library
# surface; this script is for the CLI on its own.
#
# Run from repo root:
#     .\tools\build-rust.ps1
#
# Output:
#     dist\lat.exe                          the AGS4 CLI binary
#     rust-packages\target\release\         cargo build cache (gitignored)
#
# Requires: cargo (Rust toolchain, install via `rustup`). The CLI is DuckDB-free,
# so a cold build is a couple of minutes; incremental builds are ~10 s.

$ErrorActionPreference = "Stop"

Push-Location rust-packages
try {
    # Run cargo bare (not `2>&1 |`); cargo writes progress to stderr and
    # ErrorActionPreference + pipeline-redirect makes that fatal.
    #
    # `-p laterite-ags4-check` builds the crate that carries the `lat` binary.
    $prev_ea = $ErrorActionPreference
    $ErrorActionPreference = "Continue"
    cargo build --release -p laterite-ags4-check
    $cargo_exit = $LASTEXITCODE
    $ErrorActionPreference = $prev_ea
    if ($cargo_exit -ne 0) {
        throw "cargo build failed (exit $cargo_exit)"
    }
}
finally {
    Pop-Location
}

$src = "rust-packages\target\release\lat.exe"
$dst = "dist\lat.exe"
if (-not (Test-Path $src)) {
    Write-Error "Build reported success but $src is missing."
    exit 1
}
New-Item -ItemType Directory -Force dist | Out-Null
Copy-Item $src $dst -Force

$size_mb = [math]::Round((Get-Item $dst).Length / 1MB, 2)
Write-Host ""
Write-Host "Built $dst ($size_mb MB)"
Write-Host "Smoke test:  $dst --version"
