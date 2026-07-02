#!/usr/bin/env python3
"""Figure: vtracer vs img2bez, side by side, morphing Regular <-> Bold.

Section 03 of the img2bez post. Both panels trace the same Helvetica G pair
(inputs/G-{regular,bold}.png) and render in the same Runebender point
language; the difference on screen is purely where the points sit. Left:
vtracer at its cleanest settings — a smooth silhouette, but a couple hundred
points that follow the pixels, so this script has to reconcile its two traces
itself (resample + top anchor) just to morph them at all. Right: img2bez's
joint masters trace (g-masters.json) — the structure is decided once across
the set, so the masters interpolate by construction.

Needs the `vtracer` CLI on PATH (cargo install vtracer); its SVGs are traced
into build/ on first run. Regenerate g-masters.json per README.md.

Run from the repo root:
    .venv/bin/python scripts/img2bez/fig_g_compare.py
"""
import shutil
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
import lib
import vtracer as vt
from drawbot_skia.drawing import Drawing

HERE = Path(__file__).resolve().parent
REPO = HERE.parents[1]
BUILD = HERE / "build" / "g-compare"
OUT = REPO / "public" / "demos" / "img2bez" / "g-compare.mp4"

W, H, BOX = 1300, 748, 600
PANEL_Y = 66
TOP_LABEL_Y = 702
BOT_LABEL_Y = 26
N_FRAMES, FPS = 84, 24


def panel_tx(bbox, px, box=BOX, y0=PANEL_Y, margin=0.15):
    """Height-normalized framing, so both panels' glyphs share one cap size."""
    minx, miny, maxx, maxy = bbox
    sc = (box * (1 - 2 * margin)) / (maxy - miny)
    cx, cy = (minx + maxx) / 2, (miny + maxy) / 2
    return (lambda x: px + box / 2 + (x - cx) * sc,
            lambda y: y0 + box / 2 + (y - cy) * sc)


def label(db, txt, cx, y, size=26, alpha=1.0):
    db.fill(*lib.OUTLINE, alpha)
    db.stroke(None)
    db.font("Menlo")
    db.fontSize(size)
    db.text(txt, (cx, y), align="center")


def count_points(contours):
    return sum(len(c) for c in contours)


def draw_fill_outline(db, contours, X, Y):
    db.fill(*lib.OUTLINE, 0.09)
    db.stroke(None)
    db.newPath()
    for pts in contours:
        lib.draw_contour(db, pts, X, Y)
    db.drawPath()
    db.fill(None)
    db.stroke(*lib.OUTLINE)
    db.strokeWidth(2.0)
    db.newPath()
    for pts in contours:
        lib.draw_contour(db, pts, X, Y)
    db.drawPath()


def main():
    # img2bez side: the joint masters trace snapshot.
    mi = lib.load_masters(HERE / "g-masters.json")
    reg_i, bold_i = mi["Regular"], mi["Bold"]
    bbox_i = lib.union_bbox([reg_i, bold_i])

    # vtracer side: trace the same inputs (cached in build/), then reconcile
    # its two point sets at vtracer's own native density.
    BUILD.mkdir(parents=True, exist_ok=True)
    svgs = {}
    for w in ("regular", "bold"):
        svg = BUILD / f"G-{w}.svg"
        if not svg.exists():
            vt.trace(HERE / "inputs" / f"G-{w}.png", svg)
        svgs[w] = svg
    density = vt.native_on_curve(svgs["regular"])
    pairs = vt.reconcile(vt.parse_contours(svgs["regular"]),
                         vt.parse_contours(svgs["bold"]), n=density)

    def vt_cubic(t):
        polys = [[(pr[k][0] + (pb[k][0] - pr[k][0]) * t,
                   pr[k][1] + (pb[k][1] - pr[k][1]) * t)
                  for k in range(len(pr))] for pr, pb in pairs]
        return [vt.to_cubic(poly) for poly in polys]

    bbox_v = lib.union_bbox([vt_cubic(0.0), vt_cubic(1.0)])

    def render(t, path):
        db = Drawing()
        db.size(W, H)
        db.fill(*lib.BG)
        db.rect(0, 0, W, H)
        db.stroke(*lib.OUTLINE, 0.15)
        db.strokeWidth(1)
        db.newPath()
        db.moveTo((W / 2, 48))
        db.lineTo((W / 2, 690))
        db.drawPath()

        Xv, Yv = panel_tx(bbox_v, 0)
        cv = vt_cubic(t)
        draw_fill_outline(db, cv, Xv, Yv)
        for pts in cv:
            lib.draw_points(db, pts, Xv, Yv)
        label(db, "vtracer", W * 0.25, TOP_LABEL_Y)
        label(db, f"Point Count: {count_points(cv)}", W * 0.25, BOT_LABEL_Y,
              size=20, alpha=0.7)

        Xi, Yi = panel_tx(bbox_i, W / 2)
        ci = lib.interp_contours(reg_i, bold_i, t)
        draw_fill_outline(db, ci, Xi, Yi)
        for pts in ci:
            lib.draw_points(db, pts, Xi, Yi)
        label(db, "img2bez", W * 0.75, TOP_LABEL_Y)
        label(db, f"Point Count: {count_points(ci)}", W * 0.75, BOT_LABEL_Y,
              size=20, alpha=0.7)

        db.saveImage(str(path))

    frames = BUILD / "frames"
    if frames.exists():
        shutil.rmtree(frames)
    frames.mkdir()
    for i in range(N_FRAMES):
        render(lib.ease_pingpong(i, N_FRAMES), frames / f"f{i:03d}.png")
    lib.frames_to_mp4(frames, OUT, fps=FPS)
    print(f"wrote {OUT.relative_to(REPO)} ({OUT.stat().st_size // 1024} KB, "
          f"{N_FRAMES} frames, vtracer density {density})")


if __name__ == "__main__":
    main()
