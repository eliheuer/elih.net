#!/usr/bin/env python3
"""Margin linter for the Virtua Grotesk post figures (frameless spec).

Every sheet is 2520x1320 with the outermost ink exactly MARGIN (64) from
all four canvas edges; content fills the zone. Measures real pixels:

    scratchpad-venv/bin/python check_margins.py [figure.png ...]
"""

import sys
from pathlib import Path

from PIL import Image, ImageChops

POST = Path(__file__).resolve().parents[2] / "src/content/blog/virtua-grotesk"
BG = (16, 16, 16)
MARGIN = 64
TOL = 8


def lint(path):
    im = Image.open(path).convert("RGB")
    w, h = im.size
    if (w, h) != (2520, 1320):
        return [f"SKIP ({w}x{h}, not a 2520x1320 sheet)"]
    bg = Image.new("RGB", im.size, BG)
    b = ImageChops.difference(im, bg).getbbox()
    if b is None:
        return ["empty sheet"]
    problems = []
    for name, gap in [("left", b[0]), ("top", b[1]),
                      ("right", w - b[2]), ("bottom", h - b[3])]:
        if abs(gap - MARGIN) > TOL:
            problems.append(f"outer {name} margin {gap} (spec {MARGIN})")
    return problems or ["ok"]


def main():
    targets = [Path(a) for a in sys.argv[1:]] or sorted(
        list(POST.glob("fig-*.png")) + [POST / "share-card.png"]
    )
    failed = False
    for p in targets:
        for msg in lint(p):
            if msg != "ok" and not msg.startswith("SKIP"):
                failed = True
            print(f"{p.name:28s} {msg}")
    sys.exit(1 if failed else 0)


main()
