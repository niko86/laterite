#!/usr/bin/env python3
"""Measure mimalloc v2 vs v3 vs the system allocator, on one machine.

## The question this answers, and the one it does not

`#294`/`#301` pinned mimalloc **v2** on all three native artifacts because v3
co-resident with another mimalloc corrupts memory. `#448` records that upstream
fixed the mechanism in mimalloc 3.4.4 — but the mechanism is **macOS-specific**
(v3 defaulted to Apple's fixed TLS slots 108/109), while the pin is *not*
platform-gated. So Linux and Windows carry a macOS fix, and nobody has measured
what it costs them.

That is the question here: **on this machine, does v3 beat v2 by enough to be
worth narrowing the pin to `cfg(target_os = "macos")`?**

It is a WITHIN-platform comparison — same box, same fixture, three binaries —
which is why it needs no macOS baseline. Comparing Linux-v3 against macOS-v2
would measure the machines, not the allocator.

## Why the system arm exists

It is the **positive control**, and without it a null result is worthless. The
perf ledger records mimalloc-vs-system as a real, measured win when the
allocator landed (`ags-wiki/concepts/perf-campaign.md`, ledger row 10). So if
this harness cannot separate `system` from `v2`, it cannot resolve allocator
effects at all, and "v2 and v3 look the same" means the instrument is broken —
not that the allocators are equivalent.

## Why the numbers are conservative

Each rep spawns `lat`, so process start-up sits inside every timing. That
overhead is a constant shared by all arms, so it **shrinks** the measured
relative gap rather than inflating it: a delta that clears the noise floor here
is at least that large at the parse level. Good bias for a "is it worth
unpinning" decision — this instrument under-claims.

## The noise floor, and why a gap has to clear it twice over

Criterion drifts run-to-run on the dev machine, and whether a runner is steadier
is itself unmeasured. So the arm order is **A/B/A** — v2, v3, v2 again — and the
noise floor is the WORSE of two independent estimates: that A/B/A drift, and the
widest within-arm spread. Drift alone misses a box that is jittery inside a
single arm but happens to land both v2 passes in the same place.

A gap then has to clear **2x** the floor to be called a result. Clearing it by a
hair is not one: an earlier revision of this script reported 1.5% as "measurably
faster" against a 1.0% floor, which is the exact over-claim it exists to stop.
The verdict is computed here rather than left to the reader.

Usage (repo root):

    uv run --no-sync python tools/bench/allocator_ab.py
    uv run --no-sync python tools/bench/allocator_ab.py --reps 25 --size 25MB
"""

from __future__ import annotations

import argparse
import hashlib
import io
import os
import platform
import shutil
import statistics
import subprocess
import sys
import time
from dataclasses import dataclass, field
from pathlib import Path

REPO = Path(__file__).resolve().parent.parent.parent
CRATES = REPO / "rust-packages"
CLI_MANIFEST = CRATES / "laterite-cli" / "Cargo.toml"
CLI_MAIN = CRATES / "laterite-cli" / "src" / "main.rs"
EXE = ".exe" if sys.platform == "win32" else ""

#: The shipped line, and what each arm rewrites it to. Matched EXACTLY — a
#: near-miss must fail loudly rather than leave the arm silently unpatched,
#: which would report "v3" while measuring v2.
PIN_V2 = 'mimalloc = { version = "0.1", features = ["v2"] }'
PIN_V3 = 'mimalloc = { version = "0.1" }'  # libmimalloc-sys defaults to v3
ALLOC_ATTR = (
    "#[global_allocator]\nstatic GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;"
)
ALLOC_OFF = "// [allocator_ab] system allocator arm — attribute removed for this build"


@dataclass
class Arm:
    """One built `lat`, and the readings taken from it."""

    label: str
    binary: Path | None = None
    digest: str = ""
    times: list[float] = field(default_factory=list)

    def report(self) -> tuple[float, float]:
        """min, median — in that order, because min is the least noisy statistic
        for a CPU-bound run and the median is the sanity check on it."""
        return min(self.times), statistics.median(self.times)


def run(cmd: list[str], cwd: Path) -> None:
    print(f"  $ {' '.join(cmd)}", flush=True)
    subprocess.run(cmd, cwd=cwd, check=True)


def patch(path: Path, old: str, new: str) -> None:
    """Rewrite `old` to `new`, failing if `old` was not there.

    The failure mode this guards is the quiet one: a manifest that did not
    change still builds, still runs, and reports a number under the wrong
    label."""
    text = path.read_text(encoding="utf-8")
    if old not in text:
        raise SystemExit(
            f"allocator_ab: {path.relative_to(REPO)} does not contain the expected "
            f"text — the source moved under this script:\n  {old!r}"
        )
    path.write_text(text.replace(old, new, 1), encoding="utf-8")


def build_arm(label: str, out_dir: Path) -> Arm:
    """Patch → build → stash the binary → restore. Restores on any failure."""
    manifest = CLI_MANIFEST.read_text(encoding="utf-8")
    main_rs = CLI_MAIN.read_text(encoding="utf-8")
    try:
        if label == "v3":
            patch(CLI_MANIFEST, PIN_V2, PIN_V3)
        elif label == "system":
            patch(CLI_MAIN, ALLOC_ATTR, ALLOC_OFF)
        elif label != "v2":
            raise SystemExit(f"allocator_ab: unknown arm {label!r}")

        print(f"\n=== building arm: {label} ===", flush=True)
        run(["cargo", "build", "--release", "-p", "laterite-cli"], cwd=CRATES)

        src = CRATES / "target" / "release" / f"lat{EXE}"
        dst = out_dir / f"lat-{label}{EXE}"
        shutil.copy2(src, dst)
        digest = hashlib.sha256(dst.read_bytes()).hexdigest()[:16]
        print(f"  built {dst.name}  sha256:{digest}", flush=True)
        return Arm(label=label, binary=dst, digest=digest)
    finally:
        CLI_MANIFEST.write_text(manifest, encoding="utf-8")
        CLI_MAIN.write_text(main_rs, encoding="utf-8")


def ensure_fixture(size: str, out: Path) -> Path:
    """`forge scale` synthesises a valid AGS4 file to a target size, reproducibly
    from a seed — so no real delivery is involved and any machine gets the same
    bytes. Same generator `tools/gen-bench-fixtures.sh` uses."""
    if out.exists():
        print(f"fixture present: {out}", flush=True)
        return out
    forge = CRATES / "target" / "release" / f"laterite-ags4-forge{EXE}"
    if not forge.exists():
        run(["cargo", "build", "--release", "-p", "laterite-ags4-forge"], cwd=CRATES)
    out.parent.mkdir(parents=True, exist_ok=True)
    run(
        [
            str(forge),
            "scale",
            "--size",
            size,
            "--scaffold",
            "wide",
            "--seed",
            "0",
            "--out",
            str(out),
        ],
        cwd=REPO,
    )
    return out


def time_arm(arm: Arm, fixture: Path, reps: int) -> None:
    """One warm-up (page cache, and the first run of any binary is not like the
    rest), then `reps` timed runs of the same command."""
    assert arm.binary is not None
    cmd = [str(arm.binary), "validate", str(fixture), "--no-warnings"]
    subprocess.run(cmd, capture_output=True)
    for _ in range(reps):
        t0 = time.perf_counter()
        subprocess.run(cmd, capture_output=True)
        arm.times.append((time.perf_counter() - t0) * 1000.0)


def state_the_toolchain() -> None:
    """A reading is only attributable if the output says what produced it.

    This lives here rather than in the workflow because the workflow cannot say
    it portably: the Linux legs have bash and the self-hosted Windows VM has
    neither bash nor pwsh on PATH (`release.yml` records both facts). Anything
    the harness can say about itself, it should."""
    print(f"platform: {platform.platform()}  python {platform.python_version()}")
    for tool in ("rustc", "cargo"):
        try:
            out = subprocess.run(
                [tool, "--version"], capture_output=True, text=True, check=True
            )
            print(f"{tool}: {out.stdout.strip()}")
        except (OSError, subprocess.CalledProcessError) as e:
            raise SystemExit(f"allocator_ab: {tool} is not usable here: {e}") from e


def emit(text: str, log: Path) -> None:
    """The file for the artifact, the job summary on a runner, and stdout last.

    Done here rather than by piping through `tee` and `sed` in the workflow for
    the same portability reason as the banner above: that pipeline needs a shell
    both platforms do not share.

    **Durable first, fragile last** — and that order is the fix for a real
    failure, not a preference. The first Windows run measured everything
    correctly and then died in `print`: the console there is cp1252, which has
    no U+2212 MINUS SIGN, and the table header carried one. Because printing
    came first, a cosmetic encoding fault also took out `bench.log` and left the
    artifact step with nothing to upload — a completed measurement lost to its
    own presentation layer. Both writes specify their encoding, so only stdout
    was ever at risk; it now goes last, after the results are already on disk.
    """
    log.write_text(text, encoding="utf-8")
    summary = os.environ.get("GITHUB_STEP_SUMMARY")
    if summary:
        with Path(summary).open("a", encoding="utf-8") as fh:
            fh.write(text + "\n")
    print(text, flush=True)


def main() -> int:
    # The Windows console is cp1252 and this file's prose is not. Reconfiguring
    # the stream is what keeps a typographic character in a comment or a verdict
    # line from failing a run that has already done all of its work — the class
    # of fault, not just the one character that fired it.
    # `isinstance` rather than `hasattr`: the duck-typed check narrows to
    # `object`, which `ty` correctly refuses to call a method on.
    if isinstance(sys.stdout, io.TextIOWrapper):
        sys.stdout.reconfigure(encoding="utf-8")

    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument("--reps", type=int, default=15, help="timed runs per arm")
    ap.add_argument("--size", default="25MB", help="forge scale target size")
    ap.add_argument(
        "--out-dir",
        type=Path,
        default=REPO / "output" / "allocator-ab",
        help="where the built binaries and the fixture go (gitignored)",
    )
    args = ap.parse_args()

    args.out_dir.mkdir(parents=True, exist_ok=True)
    state_the_toolchain()
    fixture = ensure_fixture(args.size, args.out_dir / f"scale-{args.size}.ags")

    built = {label: build_arm(label, args.out_dir) for label in ("v2", "v3", "system")}

    # A build that did not actually change is the failure this catches: if two
    # arms are byte-identical, the feature swap did not reach the compiler and
    # every number below is the same binary measured three times.
    digests = {a.label: a.digest for a in built.values()}
    if len(set(digests.values())) != len(digests):
        raise SystemExit(f"allocator_ab: arms are not distinct binaries — {digests}")

    # A/B/A: the second v2 pass is the drift probe, and it has to be a SEPARATE
    # reading rather than a re-use of the first, so order it explicitly.
    aba = Arm(label="v2 (again)", binary=built["v2"].binary, digest=built["v2"].digest)
    order = [built["v2"], built["v3"], aba, built["system"]]

    print(f"\n=== timing: {args.reps} reps/arm on {fixture.name} ===", flush=True)
    for arm in order:
        time_arm(arm, fixture, args.reps)
        lo, med = arm.report()
        print(f"  {arm.label:<12} min {lo:8.1f} ms   median {med:8.1f} ms", flush=True)

    v2_lo = min(built["v2"].times)
    v3_lo = min(built["v3"].times)
    aba_lo = min(aba.times)
    sys_lo = min(built["system"].times)

    # ONE sign convention for the whole report: every percentage is measured
    # against v2, and NEGATIVE MEANS FASTER. Two ways of saying the same delta —
    # a table column relative to v2 and a prose line phrased as "v2 → v3" —
    # print the same number with opposite signs, which is how a reader ends up
    # concluding the reverse of the result.
    def vs_v2(value: float) -> float:
        return (value - v2_lo) / v2_lo * 100.0

    drift = abs(aba_lo - v2_lo)
    delta = abs(v3_lo - v2_lo)
    control = abs(sys_lo - v2_lo)

    # Two independent noise estimates, and the pessimistic one wins. The A/B/A
    # drift catches slow wander between arms; the within-arm spread catches a
    # box that is jittery inside a single arm, which drift alone can miss when
    # both v2 passes happen to land in the same place.
    spread = max(statistics.median(a.times) - min(a.times) for a in order)
    noise = max(drift, spread)

    # A delta that clears the noise by a hair is not a result. Requiring 2x is
    # arbitrary in the way every threshold is, but the alternative is worse: an
    # earlier revision of this script called 1.5% "measurably faster" against a
    # 1.0% noise floor, which is precisely the over-claim it exists to stop.
    MARGIN = 2.0

    lines = [
        "",
        "| arm | min (ms) | median (ms) | vs v2 (negative = faster) |",
        "|---|---:|---:|---:|",
    ]
    for arm in order:
        lo, med = arm.report()
        rel = "—" if arm.label.startswith("v2") else f"{vs_v2(lo):+.1f}%"
        lines.append(f"| `{arm.label}` | {lo:.1f} | {med:.1f} | {rel} |")
    lines += [
        "",
        f"- **noise floor: {noise:.1f} ms** ({noise / v2_lo * 100:.1f}%) — the worse of "
        f"the A/B/A drift ({drift:.1f} ms) and the widest within-arm spread "
        f"({spread:.1f} ms). A gap must clear **{MARGIN:g}x** this to count.",
        f"- **v3 vs v2** — {'faster' if v3_lo < v2_lo else 'slower'} by {delta:.1f} ms "
        f"({abs(vs_v2(v3_lo)):.1f}%).",
        f"- **control, system vs v2** — {'faster' if sys_lo < v2_lo else 'slower'} by "
        f"{control:.1f} ms ({abs(vs_v2(sys_lo)):.1f}%). mimalloc must come out ahead "
        f"here, or the harness is not resolving allocators at all.",
        "",
    ]

    if sys_lo < v2_lo or control < MARGIN * noise:
        lines.append(
            "**VERDICT: the instrument proved nothing.** The system allocator did not "
            "come out behind mimalloc by more than this run's own drift, so the harness "
            "cannot resolve allocator effects at this scale. The v2/v3 comparison above "
            "it is meaningless — more reps, a quieter box, or a larger rung, before "
            "reading anything into it."
        )
    elif delta < MARGIN * noise:
        lines.append(
            "**VERDICT: v2 and v3 are indistinguishable here.** The control separated, "
            "so the instrument works; the v2/v3 gap does not clear the noise floor by "
            "the required margin. On the evidence of this run, narrowing the pin to "
            "macOS would buy this platform nothing measurable."
        )
    else:
        faster = "v3" if v3_lo < v2_lo else "v2"
        lines.append(
            f"**VERDICT: {faster} is measurably faster**, by more than this run's drift. "
            "Remember the spawn overhead makes this conservative — the parse-level gap "
            "is at least this large. Worth taking to #448."
        )

    emit("\n".join(lines), args.out_dir / "bench.log")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
