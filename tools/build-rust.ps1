# Build the Rust read-side `ags5db` CLI in release mode.
#
# Why this script exists: the Rust binary is the small-and-fast counterpart
# to the Python CLI (`ags5db-py`, installed via `uv tool install ./packages/ags5-db`).
# It ships at ~25-30 MB and starts in <100 ms, replacing the PyInstaller
# bundle (117.8 MB / 7.6 s cold) for the read-side commands.
#
# Run from repo root:
#     .\tools\build-rust.ps1
#
# Output:
#     dist\ags5db.exe                       the canonical read-side binary
#     rust-packages\target\release\         cargo build cache (gitignored)
#
# Requires: cargo (Rust toolchain). Installed via `rustup`. The first
# build compiles bundled libduckdb from source and takes ~5-10 min;
# incremental builds are ~10 s.

$ErrorActionPreference = "Stop"

Push-Location rust-packages
try {
    # Run cargo bare (not `2>&1 |`); cargo writes progress to stderr and
    # ErrorActionPreference + pipeline-redirect makes that fatal. See the
    # same workaround in build-exe-pyapp.ps1.
    #
    # `--bin ags5db` builds only the binary crate. The sibling
    # `ags4-validator` placeholder is part of the workspace but not
    # needed for shipping the CLI.
    $prev_ea = $ErrorActionPreference
    $ErrorActionPreference = "Continue"
    cargo build --release --bin ags5db
    $cargo_exit = $LASTEXITCODE
    $ErrorActionPreference = $prev_ea
    if ($cargo_exit -ne 0) {
        throw "cargo build failed (exit $cargo_exit)"
    }
}
finally {
    Pop-Location
}

$src = "rust-packages\target\release\ags5db.exe"
$dst = "dist\ags5db.exe"
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
