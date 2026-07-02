#!/usr/bin/env python3
"""Figure: a capital G's traced point structure interpolating Regular <-> Bold.

Section 03 of the img2bez post. Both masters were traced by img2bez from
inputs/G-{regular,bold}.png and reconciled into one shared 57-point structure;
this animates a cosine ping-pong between them (eases in/out, loops seamlessly).

Source of truth for the geometry is g-masters.json (the cached img2bez trace).
Regenerate that snapshot from the raw rasters with:

    img2bez masters <a designspace with Regular+Bold masters> \\
      --glyph G --unicode 0047 \\
      --image Regular=scripts/img2bez/inputs/G-regular.png \\
      --image Bold=scripts/img2bez/inputs/G-bold.png \\
      --fit descender:cap --format json > scripts/img2bez/g-masters.json

(The `masters` subcommand needs a designspace for metrics; see README.md for the
plan to ship a tiny stub designspace here so this figure is fully self-contained.)

Run from the repo root:
    .venv/bin/python scripts/img2bez/fig_g_interp.py
"""
import shutil
from pathlib import Path

import lib

HERE = Path(__file__).resolve().parent
REPO = HERE.parents[1]
FRAMES = HERE / "build" / "g-interp"
OUT = REPO / "public" / "demos" / "img2bez" / "g-interp.mp4"

SIZE = 640
N = 72          # frames; 3s loop at 24fps
FPS = 24


def main():
    masters = lib.load_masters(HERE / "g-masters.json")
    reg, bold = masters["Regular"], masters["Bold"]
    X, Y = lib.frame_transform(lib.union_bbox([reg, bold]), SIZE)

    if FRAMES.exists():
        shutil.rmtree(FRAMES)
    FRAMES.mkdir(parents=True)

    for i in range(N):
        t = lib.ease_pingpong(i, N)
        contours = lib.interp_contours(reg, bold, t)
        lib.render_frame(contours, FRAMES / f"f{i:03d}.png", X, Y, size=SIZE)

    lib.frames_to_mp4(FRAMES, OUT, fps=FPS)
    print(f"wrote {OUT.relative_to(REPO)} ({OUT.stat().st_size // 1024} KB, {N} frames)")


if __name__ == "__main__":
    main()
