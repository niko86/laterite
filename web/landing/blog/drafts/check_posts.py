# /// script
# requires-python = ">=3.12"
# dependencies = [
#     "laterite",
#     "laterite-quarry[rvo]",
#     "laterite-outcrop[laterite]>=0.1.1",
#     "marimo",
#     "maplibre[anywidget]",
#     "matplotlib",
#     "pygments",
#     "pyproj",
# ]
# ///
"""Run everything the posts show, on the public data they name, and say what changed.

    uv run check_posts.py

One command before a post goes out. It fetches the two datasets with
laterite-quarry (once; they are cached), runs every post's script and the `lat`
commands the opener quotes, runs the notebook, and redraws the figures and code
cards into a temporary folder to compare with the committed ones.

A line the drafts quote that no longer comes out is a failure: either the
draft is stale or something broke. An image that differs is only reported,
because fonts and library versions differ between machines.
"""

import shutil
import subprocess
import sys
import tempfile
from pathlib import Path

from laterite_quarry import cli as quarry
from laterite_quarry import store

HERE = Path(__file__).parent
FARRINGDON = ["bgs", "pull", "farringdon", "--bbox=-0.108,51.518,-0.102,51.522"]
FUGRO = ["rvo", "pull", "hkz-wfs3-bh"]

results: list[tuple[str, str, str]] = []  # (status, what, detail)


def note(status: str, what: str, detail: str = "") -> None:
    results.append((status, what, detail))
    print(f"{status:<5} {what}" + (f"  ({detail})" if detail else ""), flush=True)


def run(
    what: str,
    command: list[str],
    cwd: Path,
    expect: list[str],
    codes: tuple[int, ...] = (0,),
    forbid: str = "",
) -> None:
    """Run one command; pass if it exits as expected and prints every quoted line."""
    done = subprocess.run(command, cwd=cwd, capture_output=True, text=True, check=False)
    printed = done.stdout + done.stderr
    if done.returncode not in codes:
        last = printed.strip().splitlines()[-1:] or ["no output"]
        note("FAIL", what, f"exit {done.returncode}: {last[0][:150]}")
    elif forbid and forbid in printed:
        note("FAIL", what, forbid)
    else:
        missing = [line for line in expect if line not in printed]
        note(
            "FAIL" if missing else "ok",
            what,
            f"not printed: {missing[0]!r}" if missing else "",
        )


def same_images(what: str, fresh: Path) -> None:
    """Report, without failing, which redrawn images differ from the committed ones."""
    drawn = sorted(fresh.glob("post-*/*.png"))
    differ = [
        f"{image.parent.name}/{image.name}"
        for image in drawn
        if not (HERE / image.parent.name / image.name).exists()
        or image.read_bytes() != (HERE / image.parent.name / image.name).read_bytes()
    ]
    if differ:
        note("DIFF", what, f"{len(differ)} of {len(drawn)} differ: {', '.join(differ)}")
    else:
        note("ok", what, f"all {len(drawn)} identical to the committed images")


def main() -> int:
    for pull in (FARRINGDON, FUGRO):
        if quarry.main(pull) != 0:
            note("FAIL", "fetch " + " ".join(pull[:3]))
            return 1
    crossrail = (
        store.dataset_dir("bgs", "farringdon") / "cell000" / "202002061142130.ags"
    )
    fugro = store.dataset_dir("rvo", "hkz-wfs3-bh")
    python = sys.executable

    with tempfile.TemporaryDirectory() as scratch:
        work = Path(scratch)
        merged = work / "merged.ags"

        def script(name: str) -> str:
            return str(next(HERE.glob(f"post-*/{name}")))

        # Post 5 first: posts 2 and 3 read the file it writes.
        run(
            "post 5  merge_files.py",
            [python, script("merge_files.py"), str(fugro), str(merged)],
            work,
            [
                "LOCA    17 in    17 out",
                "SAMP   754 in   754 out",
                "17 errors, 3 warnings",
            ],
        )
        run(
            "post 5  type_clash.py",
            [python, script("type_clash.py")],
            work,
            [
                "promote: declared 2DP, read as [12.5, 8.5]",
                "changed: ['LOCA_FDEP: 2DP -> X']",
            ],
        )
        run(
            "post 1  one_borehole.py",
            [python, script("one_borehole.py"), str(crossrail), "F1"],
            work,
            ["GEOL    20 rows", "SAMP   100 rows"],
        )
        run(
            "post 2  typed_columns.py",
            [python, script("typed_columns.py"), str(merged)],
            work,
            ["10 of 53 results are high plasticity or above", "LNMC_MC is declared X"],
        )
        run(
            "post 3  sql_across_groups.py",
            [python, script("sql_across_groups.py"), str(merged)],
            work,
            ["53 of 53 results matched a stratum"],
        )
        run(
            "post 4  check.py",
            [python, script("check.py"), str(crossrail), str(work / "fixed.ags")],
            work,
            ["1488 errors, 13 warnings", "94 errors left"],
        )
        run(
            "post 4  clean_recipe.py",
            [
                python,
                script("clean_recipe.py"),
                str(crossrail),
                str(work / "cleaned.ags"),
            ],
            work,
            ["1. fix()                                  94", "13 warning(s) remain"],
        )

        lat = shutil.which("lat")
        if lat is None:
            note("SKIP", "post 0  lat commands", "no lat command in this environment")
        else:
            # `lat` exits 1 when it finds anything, which is the right answer here.
            run(
                "post 0  lat validate",
                [lat, "validate", str(crossrail)],
                work,
                ["1501 finding(s)"],
                (1,),
            )
            run(
                "post 0  lat fix",
                [lat, "fix", str(crossrail), "--fix-out", str(work / "lat_fixed.ags")],
                work,
                ["applied 1 fix(es) [normalize_crlf]", "107 finding(s) remain"],
                (1,),
            )
            run(
                "post 0  lat read --csv",
                [lat, "read", str(crossrail), "LOCA", "--csv"],
                work,
                ["F1,CP,"],
            )

        notebook = HERE / "extra-marimo-notebook" / "boreholes_on_a_map.py"
        export = [python, "-m", "marimo", "export", "html", str(notebook)]
        export += [
            "-o",
            str(work / "notebook.html"),
            "-f",
            "--",
            "--file",
            str(crossrail),
        ]
        # The export exits 0 even when a cell raised, so its message is the test.
        run(
            "notebook runs top to bottom",
            export,
            work,
            [],
            forbid="cells failed to execute",
        )

        fresh = work / "images"
        run(
            "figures redraw",
            [python, str(HERE / "figures.py"), str(crossrail), str(merged), str(fresh)],
            work,
            [],
        )
        run(
            "code cards redraw",
            [python, str(HERE / "code_cards.py"), str(fresh)],
            work,
            [],
        )
        same_images("images match the committed ones", fresh)

    failed = [r for r in results if r[0] == "FAIL"]
    print()
    print(
        f"{len(results) - len(failed)} of {len(results)} checks passed, {len(failed)} failed."
    )
    print("Not checked here, so check by hand before posting:")
    print("  - the notebook's street map draws, and clicking a borehole selects it")
    print("  - the wording of each LinkedIn post and article")
    print("  - that each image still says what its post says")
    return 1 if failed else 0


if __name__ == "__main__":
    sys.exit(main())
