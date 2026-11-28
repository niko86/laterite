# /// script
# requires-python = ">=3.12"
# dependencies = ["matplotlib", "pygments"]
# ///
"""Draw each LinkedIn post's code as an image, because LinkedIn shows code as plain text.

    uv run code_cards.py [output-folder]

Reads the fenced blocks out of every `post-*/linkedin.md` and writes them, in
order, to `code.png` beside it: code highlighted, output as a quieter panel.
The post then goes out as text with two images, the card and the figure.
"""

import re
import sys
from pathlib import Path

import matplotlib as mpl

mpl.use("Agg")

import matplotlib.pyplot as plt
from matplotlib.patches import FancyBboxPatch
from pygments import lex
from pygments.lexers import BashLexer, PythonLexer, SqlLexer
from pygments.token import Comment, Keyword, Name, Number, Operator, String, Token

HERE = Path(__file__).parent
WIDTH_PX, DPI = 1080, 100  # LinkedIn shows a 1080-wide image at full width
MARGIN_PX, PAD_PX, GAP_PX = 48, 30, 26
PAGE, PANEL, QUIET = "#0d1117", "#161b22", "#11161d"
INK, MUTED = "#e6edf3", "#8b949e"
# One colour per kind of token, checked from the most specific kind to the least.
COLOURS = [
    (Comment, MUTED),
    (String, "#a5d6ff"),
    (Number, "#79c0ff"),
    (Keyword, "#ff7b72"),
    (Name.Function, "#d2a8ff"),
    (Name.Builtin, "#79c0ff"),
    (Operator, "#ff7b72"),
    (Token, INK),
]
MONO = "DejaVu Sans Mono"
LEXERS = {"python": PythonLexer, "bash": BashLexer, "sql": SqlLexer}


def advance() -> float:
    """Width of one character of the font as a share of its size, measured not assumed."""
    from matplotlib.font_manager import FontProperties
    from matplotlib.textpath import TextPath

    font = FontProperties(family=MONO)
    # The difference between two runs is one character's advance, free of side bearings.
    short = TextPath((0, 0), "M" * 100, size=100, prop=font).get_extents().x1
    long = TextPath((0, 0), "M" * 101, size=100, prop=font).get_extents().x1
    return (long - short) / 100


ADVANCE = advance()
LINE = 1.5  # line height, as a share of the font size
FENCE = re.compile(r"^```(\w*)\n(.*?)^```$", re.DOTALL | re.MULTILINE)


def blocks(markdown: str) -> list[tuple[str, list[str]]]:
    """The fenced blocks of a post as (language, lines), in the order they appear."""
    body = markdown.split("\n---\n", 1)[-1]  # the notes above the rule are not the post
    return [(lang, text.rstrip("\n").split("\n")) for lang, text in FENCE.findall(body)]


def colour_of(kind: object) -> str:
    return next(colour for parent, colour in COLOURS if kind in parent)


def spans(lang: str, lines: list[str]) -> list[list[tuple[str, str]]]:
    """Each line as (text, colour) runs. Output, which has no language, is one quiet run."""
    if lang not in LEXERS:
        return [[(line, "#c9d1d9")] for line in lines]
    lexer = LEXERS[lang]()
    out: list[list[tuple[str, str]]] = [[]]
    for kind, text in lex("\n".join(lines), lexer):
        for n, piece in enumerate(text.split("\n")):
            if n:
                out.append([])
            if piece:
                out[-1].append((piece, colour_of(kind)))
    return out[: len(lines)]


def card(title: str, found: list[tuple[str, list[str]]], target: Path) -> None:
    widest = max(48, *(len(line) for _lang, lines in found for line in lines))
    inner_px = WIDTH_PX - 2 * MARGIN_PX - 2 * PAD_PX
    size_pt = inner_px * 72 / DPI / (widest * ADVANCE)
    line_px = size_pt * LINE * DPI / 72
    head_px = 96
    heights = [len(lines) * line_px + 2 * PAD_PX for _lang, lines in found]
    height_px = head_px + sum(heights) + GAP_PX * (len(found) - 1) + MARGIN_PX

    fig = plt.figure(figsize=(WIDTH_PX / DPI, height_px / DPI), dpi=DPI, facecolor=PAGE)
    ax = fig.add_axes((0, 0, 1, 1))
    ax.set(xlim=(0, WIDTH_PX), ylim=(height_px, 0))
    ax.axis("off")
    ax.text(MARGIN_PX, 58, title, color=INK, fontsize=19, weight="bold", va="center")
    ax.text(
        WIDTH_PX - MARGIN_PX,
        58,
        "laterite",
        color=MUTED,
        fontsize=15,
        va="center",
        ha="right",
    )

    top = head_px
    char_px = size_pt * ADVANCE * DPI / 72
    for (lang, lines), block_px in zip(found, heights, strict=True):
        ax.add_patch(
            FancyBboxPatch(
                (MARGIN_PX, top),
                WIDTH_PX - 2 * MARGIN_PX,
                block_px,
                boxstyle="round,pad=0,rounding_size=14",
                facecolor=PANEL if lang in LEXERS else QUIET,
                edgecolor="#30363d",
                linewidth=1,
            )
        )
        for row, runs in enumerate(spans(lang, lines)):
            y = top + PAD_PX + (row + 0.5) * line_px
            column = 0
            for text, colour in runs:
                ax.text(
                    MARGIN_PX + PAD_PX + column * char_px, y, text,
                    color=colour, fontsize=size_pt, family=MONO, va="center",
                )  # fmt: skip
                column += len(text)
        top += block_px + GAP_PX
    fig.savefig(target, dpi=DPI, facecolor=PAGE)
    plt.close(fig)


def main(out: str | None = None) -> None:
    skipped = []
    for post in sorted(HERE.glob("post-*/linkedin.md")):
        text = post.read_text(encoding="utf-8")
        found = blocks(text)
        if not found:
            skipped.append(post.parent.name)
            continue
        # The card is headed with the article's title, which is the post's subject.
        article = post.with_name("article.md").read_text(encoding="utf-8")
        title = article.split("\n", 1)[0].removeprefix("# ").strip()
        target = Path(out) / post.parent.name if out else post.parent
        target.mkdir(parents=True, exist_ok=True)
        card(title, found, target / "code.png")
        print(f"{post.parent.name}/code.png  {len(found)} block(s)")
    # A post with no code gets no card; say so rather than pass over it.
    print(f"no code blocks, no card: {', '.join(skipped) or 'none'}")


if __name__ == "__main__":
    main(*sys.argv[1:2])
