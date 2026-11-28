"""Draw the image each post carries, into that post's folder.

    uv run --with laterite --with "laterite-outcrop[laterite]" figures.py \
        <crossrail.ags> <fugro-merged.ags>

The first file is the BGS Crossrail export posts 1 and 4 use; the second is
the seventeen Fugro files merged by post 5's script.
"""

import sys
from pathlib import Path

import laterite
import matplotlib as mpl

mpl.use("Agg")

import matplotlib.pyplot as plt
import outcrop
import polars as pl
from outcrop import ags as plots

HERE = Path(__file__).parent
INK, MUTED, BLUE, RULE = "#1f2328", "#6b7280", "#2f6fb0", "#c9ced6"

# What each rule is about, in a few words, for the post 4 chart.
ABOUT = {
    "Rule 2a": "line endings",
    "Rule 7": "heading order",
    "Rule 8": "dates not as declared",
    "Rule 9": "heading not in dictionary",
    "Rule 10a": "duplicate keys",
    "Rule 10b": "required field empty",
    "Rule 16": "code not in ABBR",
    "Rule 19a": "heading too long",
    "Rule 19b": "heading name form",
}

# Errors left after each step of post 4's clean_recipe.py, as it prints them.
STEPS = [
    (".fix()", 94),
    ("duplicates", 75),
    ("date unit", 51),
    ("unused units", 32),
    ("TRAN", 31),
    ("codes, units", 24),
    ("rename, declare", 1),
    ("stale DICT row", 0),
]

IN_STRATUM = """
    SELECT l.LLPL_LL, l.LLPL_PI, g.GEOL_STAT AS unit
    FROM LLPL l
    JOIN GEOL g
      ON g.LOCA_ID = l.LOCA_ID
     AND l.SAMP_TOP >= g.GEOL_TOP AND l.SAMP_TOP < g.GEOL_BASE
    ORDER BY unit
"""


# Each stratum with the principal soil its legend code stands for.
PRINCIPAL_SOIL = """
    SELECT g.LOCA_ID, g.GEOL_TOP, g.GEOL_BASE,
           CASE WHEN a.ABBR_DESC LIKE '%CLAY' THEN 'Clay'
                WHEN a.ABBR_DESC LIKE '%SAND' THEN 'Sand'
                ELSE 'Other or not given' END AS soil
    FROM GEOL g
    LEFT JOIN ABBR a
      ON a.ABBR_HDNG = 'GEOL_LEG' AND a.ABBR_CODE = g.GEOL_LEG
"""
SOILS = {"Sand": "#e0a530", "Clay": "#2f6fb0", "Other or not given": "#9aa1ab"}


def save(fig: plt.Figure, post: str, name: str, dpi: int = 140) -> None:
    fig.savefig(HERE / post / name, dpi=dpi, bbox_inches="tight")
    plt.close(fig)


def bare(ax: plt.Axes) -> None:
    """Strip an axes to its marks: these charts label every bar directly."""
    for side in ("top", "right", "left"):
        ax.spines[side].set_visible(False)
    ax.spines["bottom"].set_color(RULE)
    ax.tick_params(length=0, colors=INK)


def post_1(crossrail: laterite.Ags4File) -> None:
    fig = plots.borehole_log(crossrail, "F1")
    fig.savefig(HERE / "post-1-one-borehole" / "log.png", dpi=110)
    plt.close(fig)


def post_2(fugro: laterite.Ags4File) -> None:
    title = "Hollandse Kust (zuid) site III: 53 Atterberg results"
    ax = plots.plasticity_chart(fugro, title=title)
    save(ax.figure, "post-2-typed-columns", "plasticity.png")


def post_3(fugro: laterite.Ags4File) -> None:
    frame = pl.DataFrame(fugro.sql(IN_STRATUM))
    title = "The same 53 results, by soil unit"
    ax = outcrop.plasticity_chart(frame, by="unit", title=title)
    save(ax.figure, "post-3-sql-across-groups", "by_unit.png")


def post_4(crossrail: laterite.Ags4File) -> None:
    report = crossrail.validate().report
    counts = sorted(
        (
            (rule.replace("AGS Format ", ""), len(rows))
            for rule, rows in report.by_rule().items()
            if "Warning" not in rule
        ),
        key=lambda pair: pair[1],
    )
    fig, ax = plt.subplots(figsize=(7.5, 4.2))
    labels = [f"{rule}  {ABOUT.get(rule, '')}" for rule, _ in counts]
    ax.barh(labels, [n for _, n in counts], color=BLUE, height=0.62)
    ax.set_xscale("log")
    ax.set_xticks([])
    ax.minorticks_off()
    for row, (_, n) in enumerate(counts):
        ax.text(n * 1.12, row, f"{n:,}", va="center", color=INK, fontsize=10)
    bare(ax)
    ax.spines["bottom"].set_visible(False)
    title = f"{report.errors:,} errors in one file, by rule (log scale)"
    ax.set_title(title, loc="left", color=INK, fontsize=12)
    save(fig, "post-4-clean-a-real-file", "by_rule.png")

    fig, ax = plt.subplots(figsize=(7.5, 4.2))
    at = range(len(STEPS))
    colours = [BLUE] + [MUTED] * (len(STEPS) - 1)
    ax.bar(at, [n for _, n in STEPS], color=colours, width=0.62)
    for x, (_, n) in zip(at, STEPS, strict=True):
        ax.text(x, n + 1.5, str(n), ha="center", color=INK, fontsize=10)
    ax.set_xticks(at)
    ax.set_xticklabels([s for s, _ in STEPS], rotation=30, ha="right", fontsize=9)
    ax.set_yticks([])
    bare(ax)
    title = f"Errors left after each step ({report.errors:,} as sent)"
    ax.set_title(title, loc="left", color=INK, fontsize=12)
    save(fig, "post-4-clean-a-real-file", "steps.png")


def post_5(fugro: laterite.Ags4File) -> None:
    """Every borehole in the merged file, side by side on one depth scale."""
    strata = pl.DataFrame(fugro.sql(PRINCIPAL_SOIL))
    holes = sorted(strata["LOCA_ID"].unique().to_list())
    column = {hole: x for x, hole in enumerate(holes)}
    fig, ax = plt.subplots(figsize=(11, 5.6))
    for soil, colour in SOILS.items():
        rows = strata.filter(pl.col("soil") == soil)
        ax.bar(
            [column[hole] for hole in rows["LOCA_ID"]],
            rows["GEOL_BASE"] - rows["GEOL_TOP"],
            bottom=rows["GEOL_TOP"],
            width=0.5,
            color=colour,
            edgecolor="white",
            linewidth=0.6,
            label=soil,
        )
    ax.invert_yaxis()
    ax.set_ylabel("Depth (m)", color=INK)
    ax.set_xticks(range(len(holes)))
    names = [hole.removeprefix("HKZ3-") for hole in holes]
    ax.set_xticklabels(names, rotation=45, ha="left", fontsize=9)
    ax.xaxis.tick_top()
    bare(ax)
    ax.spines["bottom"].set_visible(False)
    ax.grid(axis="y", color=RULE, linewidth=0.5)
    ax.set_axisbelow(True)
    ax.legend(
        frameon=False, ncols=len(SOILS), loc="lower left", bbox_to_anchor=(0, -0.1)
    )
    title = f"One merged file: {len(holes)} boreholes, {strata.height} strata"
    ax.set_title(title, loc="left", color=INK, fontsize=12, pad=14)
    save(fig, "post-5-merge", "site.png")


def main(crossrail_path: str, fugro_path: str) -> None:
    crossrail = laterite.read(crossrail_path)
    fugro = laterite.read(fugro_path)
    post_1(crossrail)
    post_2(fugro)
    post_3(fugro)
    post_4(crossrail)
    post_5(fugro)


if __name__ == "__main__":
    main(sys.argv[1], sys.argv[2])
