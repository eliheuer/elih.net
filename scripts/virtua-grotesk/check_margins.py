#!/usr/bin/env python3
"""Margin linter for the Virtua Grotesk post figures.

The house frame (2520x1320 sheets): green rules 96px from the top and
bottom edges; frame text (title row above the header rule, caption row
below the footer rule) exactly GAP px off its rule; the content block
between the rules equidistant from both rules, and centered between the
side margins.

Measures real pixels, so it catches what the eye catches:

    scratchpad-venv/bin/python check_margins.py [figure.png ...]

With no arguments, lints every fig-*.png in the post directory.
"""

import sys
from pathlib import Path

from PIL import Image, ImageChops

POST = Path(__file__).resolve().parents[2] / "src/content/blog/virtua-grotesk"

BG = (16, 16, 16)
GAP = 24  # spec: text sits this far off its rule
TOL_TEXT = 5  # px tolerance on the text gap
TOL_V = 14  # px tolerance on content top-vs-bottom balance
TOL_H = 24  # px tolerance on content left-vs-right balance


def bbox(im, box):
    """bbox of non-background pixels inside box, in full-image coords."""
    region = im.crop(box)
    bg = Image.new("RGB", region.size, BG)
    b = ImageChops.difference(region, bg).getbbox()
    if b is None:
        return None
    return (b[0] + box[0], b[1] + box[1], b[2] + box[0], b[3] + box[1])


def lint(path):
    im = Image.open(path).convert("RGB")
    w, h = im.size
    if (w, h) != (2520, 1320):
        return [f"SKIP ({w}x{h}, not a 2520x1320 sheet)"]

    header_row = 112  # image row of the header rule (canvas y 1208)
    footer_row = 1208  # image row of the footer rule (canvas y 112)
    problems = []

    # outermost ink must sit MARGIN from every canvas edge
    ink = bbox(im, (0, 0, w, h))
    for name, gap in [("left", ink[0]), ("top", ink[1]),
                      ("right", w - ink[2]), ("bottom", h - ink[3])]:
        if abs(gap - 64) > 6:
            problems.append(f"outer {name} margin {gap} (spec 64)")

    # frame text: nearest ink to each rule from the outside
    top = bbox(im, (0, 0, w, header_row - 4))
    bot = bbox(im, (0, footer_row + 5, w, h))
    if top is None or bot is None:
        problems.append("missing frame text")
    else:
        gap_top = header_row - top[3]
        gap_bot = bot[1] - footer_row
        # descenders (commas etc.) may dip up to 9px below the text baseline
        if not (GAP - 9 <= gap_top <= GAP + TOL_TEXT):
            problems.append(f"title-to-rule gap {gap_top} (spec {GAP})")
        if not (GAP - 9 <= gap_bot <= GAP + TOL_TEXT):
            problems.append(f"caption-to-rule gap {gap_bot} (spec {GAP})")

    # content block between the rules
    c = bbox(im, (0, header_row + 5, w, footer_row - 4))
    if c is None:
        problems.append("no content between the rules")
    else:
        vt, vb = c[1] - header_row, footer_row - c[3]
        hl, hr = c[0], w - c[2]
        if abs(vt - vb) > TOL_V:
            problems.append(f"content off-center vertically: {vt} above, {vb} below")
        if abs(hl - hr) > TOL_H:
            problems.append(f"content off-center horizontally: {hl} left, {hr} right")

    return problems or ["ok"]


def main():
    targets = [Path(a) for a in sys.argv[1:]] or sorted(
        list(POST.glob("fig-*.png")) + [POST / "share-card.png"]
    )
    failed = False
    for p in targets:
        for msg in lint(p):
            if msg not in ("ok",) and not msg.startswith("SKIP"):
                failed = True
            print(f"{p.name:28s} {msg}")
    sys.exit(1 if failed else 0)


main()
