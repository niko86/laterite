"""`--no-opt` reaches exactly one wasm build: the full engine in `ts-lint` (#1115).

`ts-lint` builds every wasm package only so eslint's type-aware rules can read
the generated `.d.ts`; the web app imports its types from the full engine. That
build's `wasm-opt` pass is the slow part of the job and its output is thrown
away, so there the full engine is built unoptimized. The `.d.ts` does not depend
on the pass — wasm-bindgen writes it before `wasm-opt` runs — which was checked
once by building both and comparing the files.

Every other wasm build must stay optimized, and it is easy to break that from
the side nobody is looking at: `--no-opt` added to the shared `build:wasm-full`
script would ride into the deploy and e2e builds, and added to `build:wasm`
would make the tier-1 ceiling in `tools/release/check-wasm-tier1.mjs` measure a
binary the app never ships. So this resolves each workflow's wasm builds the way
npm would — following `npm run` into `web/package.json`, appending `-- args` to
the script text — and checks where `--no-opt` lands.

A `--no-opt` after wasm-pack's own `--` would go to cargo, not wasm-pack, and
fail the build, so only wasm-pack's half of the command counts.

WHAT THIS CANNOT SEE: it reads `run:` steps in `.github/workflows/*.yml`, and
only two command shapes in them — `wasm-pack build …` and `npm run <script>`.
A wasm build spelled any other way (a shell script, `npx wasm-pack`) is not
followed. The count of builds it resolved is printed on every run.
"""

from __future__ import annotations

import json
import re
import shlex
from pathlib import Path
from typing import Any

import pytest
import yaml

REPO = Path(__file__).resolve().parents[1]
WORKFLOWS = REPO / ".github" / "workflows"
WEB_SCRIPTS: dict[str, str] = json.loads(
    (REPO / "web" / "package.json").read_text(encoding="utf-8")
)["scripts"]

NPM_RUN = re.compile(r"^npm run (\S+)(?: -- (.*))?$")


def _expand(command: str, depth: int = 0) -> list[str]:
    """The wasm-pack invocations one shell command amounts to, as npm runs it."""
    assert depth < 10, f"npm script recursion too deep at {command!r}"
    out: list[str] = []
    for part in (p.strip() for p in command.split("&&")):
        if m := NPM_RUN.match(part):
            name, extra = m.group(1), m.group(2)
            if name not in WEB_SCRIPTS:
                continue
            script = WEB_SCRIPTS[name] + (f" {extra}" if extra else "")
            out += _expand(script, depth + 1)
        elif part.startswith("wasm-pack build"):
            out.append(part)
    return out


def _builds(run: str) -> list[str]:
    text = run.replace("\\\n", " ")
    return [b for line in text.splitlines() for b in _expand(line.strip())]


def _wasm_pack_args(build: str) -> list[str]:
    """wasm-pack's own options — everything before the cargo passthrough `--`."""
    args = shlex.split(build)
    return args[: args.index("--")] if "--" in args else args


def _out_dir(build: str) -> str:
    args = _wasm_pack_args(build)
    return args[args.index("--out-dir") + 1] if "--out-dir" in args else ""


def _all_builds() -> list[tuple[str, str, str, str]]:
    """(workflow, job, step name, wasm-pack invocation) for every wasm build."""
    rows = []
    for path in sorted(WORKFLOWS.glob("*.yml")):
        wf: dict[str, Any] = yaml.safe_load(path.read_text(encoding="utf-8"))
        for job_name, job in (wf.get("jobs") or {}).items():
            for step in job.get("steps", []):
                if isinstance(step, dict) and "run" in step:
                    name = step.get("name", "?")
                    rows.extend(
                        (path.name, job_name, name, b) for b in _builds(step["run"])
                    )
    return rows


BUILDS = _all_builds()


def _is_ts_lint_full(row: tuple[str, str, str, str]) -> bool:
    wf, job, _, build = row
    return wf == "ci.yml" and job == "ts-lint" and _out_dir(build).endswith("wasm-full")


def test_ts_lint_builds_the_full_engine_unoptimized(
    capsys: pytest.CaptureFixture[str],
) -> None:
    with capsys.disabled():
        print(f"\n[wasm-opt scope] resolved {len(BUILDS)} wasm-pack builds")
    full = [r for r in BUILDS if _is_ts_lint_full(r)]
    assert full, "ts-lint no longer builds the full engine; eslint needs its .d.ts"
    for _, _, step, build in full:
        assert "--no-opt" in _wasm_pack_args(build), (
            f"ts-lint step {step!r} builds the full engine with wasm-opt; only its "
            f".d.ts is used there, so it should pass --no-opt: {build}"
        )


def test_ts_lint_still_builds_every_engine_the_lint_reads() -> None:
    dirs = {
        _out_dir(b).rsplit("/", 1)[-1]
        for wf, job, _, b in BUILDS
        if wf == "ci.yml" and job == "ts-lint"
    }
    assert {"wasm", "wasm-full", "wasm-tokenizer"} <= dirs, dirs


@pytest.mark.parametrize(
    "row",
    [r for r in BUILDS if not _is_ts_lint_full(r)],
    ids=lambda r: f"{r[0]}:{r[1]}:{_out_dir(r[3]).rsplit('/', 1)[-1] or 'pkg'}",
)
def test_every_other_wasm_build_stays_optimized(
    row: tuple[str, str, str, str],
) -> None:
    wf, job, step, build = row
    assert "--no-opt" not in shlex.split(build), (
        f"{wf} job {job!r} step {step!r} skips wasm-opt; only ts-lint's full "
        f"engine may — deploy, e2e, the release and both size ceilings need the "
        f"optimized artifact: {build}"
    )
