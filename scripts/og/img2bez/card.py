#!/usr/bin/env python3
"""Render the img2bez post's share card / hero at 2400x1260 (2x of 1200x630).

The story in one image: a low-resolution, pixelated raster 'a' (the kind of
input img2bez traces) sitting under the clean vector trace it produces — white
outline, green on-curve points, purple off-curve handles. Dark mode, inside a
frame that matches the page's code blocks and the interactive demo.

One script per post (mirrors public/og/<postId>.png). Self-contained: it traces
the source raster with the `img2bez` CLI, so it needs no pre-existing .ufo.

Drawn with the eliheuer/drawbot-skia fork. Run from the repo root with the
local venv:  .venv/bin/python scripts/og/img2bez/card.py
"""
import subprocess
import tempfile
import xml.etree.ElementTree as ET
from pathlib import Path

import drawbot_skia.drawbot as db
from PIL import Image

W, H = 2400, 1260
CELL = 60  # raster pixel block size (the "low resolution")
LINE_WIDTH = 8  # one stroke weight for the outline, handles, and point rings

# colours, 0-1 floats. Dark-mode editor palette (matches the demo island).
BG = (0.047, 0.047, 0.047)
PIXEL_MAX = (0.34, 0.34, 0.34)  # brightest raster block (darkest source pixel)
OUTLINE = (0.90, 0.90, 0.90)
HANDLE = (0.42, 0.42, 0.42)
GREEN = (0.40, 0.933, 0.533)  # on-curve
PURPLE = (0.545, 0.424, 1.0)  # off-curve

HERE = Path(__file__).resolve().parent  # scripts/og/img2bez/
REPO = HERE.parents[2]
SRC = HERE / "source.png"  # the raster this card traces, kept next to the script
OG = REPO / "public/og/img2bez.png"
HERO = REPO / "src/content/blog/img2bez/share-card.png"


def trace_glif(src: Path) -> Path:
    """Trace `src` with the img2bez CLI and return the path to the .glif."""
    out = Path(tempfile.mkdtemp(prefix="og_card_")) / "a.ufo"
    try:
        subprocess.run(
            ["img2bez", "--input", str(src), "--output", str(out), "--name", "a"],
            check=True, capture_output=True, text=True,
        )
    except FileNotFoundError:
        raise SystemExit("error: `img2bez` not on PATH — cargo install --git "
                         "https://github.com/eliheuer/img2bez")
    except subprocess.CalledProcessError as e:
        raise SystemExit(f"error: img2bez trace failed:\n{e.stderr}")
    return out / "glyphs" / "a.glif"


def contours(glif: Path):
    root = ET.parse(glif).getroot()
    out = []
    for c in root.iter("contour"):
        pts = [(float(p.get("x")), float(p.get("y")), p.get("type") is not None)
               for p in c.findall("point")]
        if pts:
            out.append(pts)
    return out


def source_glyph_box(img):
    """Tight bbox (px) of the glyph in the source raster, plus its bg value."""
    g = img.convert("L")
    w, h = g.size
    px = g.load()
    bg = (px[0, 0] + px[w - 1, 0] + px[0, h - 1] + px[w - 1, h - 1]) / 4
    minx, miny, maxx, maxy = w, h, 0, 0
    for y in range(h):
        for x in range(w):
            if abs(px[x, y] - bg) > 40:
                minx, miny = min(minx, x), min(miny, y)
                maxx, maxy = max(maxx, x), max(maxy, y)
    return minx, miny, maxx + 1, maxy + 1, bg


def main():
    glif = trace_glif(SRC)
    cs = contours(glif)

    db.newDrawing()
    db.newPage(W, H)
    db.fill(*BG)
    db.rect(0, 0, W, H)
    # No baked-in frame: the page gives this image the same CSS border + radius
    # as code blocks and the demo island, so a drawn frame would double it.

    # ── the glyph box (drawbot is y-up, origin bottom-left) ──
    # Size the glyph by WIDTH so it fills the wide card (a real zoom-in).
    # Centre it HORIZONTALLY, but anchor the TOP: the arch stays fully visible
    # with TOP_MARGIN above it and the foot crops off the bottom edge.
    WIDTH_FRAC = 0.66  # glyph width as a fraction of the card width
    TOP_MARGIN = 150
    img = Image.open(SRC)
    sx0, sy0, sx1, sy1, bgval = source_glyph_box(img)
    box_w = W * WIDTH_FRAC
    box_h = box_w * (sy1 - sy0) / (sx1 - sx0)
    box_y = H - TOP_MARGIN - box_h   # top-anchored; foot spills below the bottom edge

    # on-curve bbox (the visible outline; off-curve handles spill past it)
    on = [p for c in cs for p in c if p[2]]
    fminx, fmaxx = min(p[0] for p in on), max(p[0] for p in on)
    fminy, fmaxy = min(p[1] for p in on), max(p[1] for p in on)

    # Centre HORIZONTALLY on the *visible* part only. The cropped foot stretches
    # the bbox to the bottom-right, so centring the whole bbox would shove the
    # visible bowl/arch left; centre on points that land inside the frame.
    fy_lo = fminy + (0 - box_y) * (fmaxy - fminy) / box_h
    fy_hi = fminy + (H - box_y) * (fmaxy - fminy) / box_h
    vis = [p for p in on if fy_lo <= p[1] <= fy_hi] or on
    vcx = (min(p[0] for p in vis) + max(p[0] for p in vis)) / 2
    box_x = W / 2 - (vcx - fminx) / (fmaxx - fminx) * box_w

    # ── pixelated raster underlay: crop to the glyph, sample into CELL blocks ──
    cols = max(1, round(box_w / CELL))
    rows = max(1, round(box_h / CELL))
    low = img.crop((sx0, sy0, sx1, sy1)).convert("L").resize((cols, rows), Image.BILINEAR)
    lpx = low.load()
    cw, ch = box_w / cols, box_h / rows
    db.stroke(None)
    for j in range(rows):
        for i in range(cols):
            ink = max(0.0, (bgval - lpx[i, j]) / max(1.0, bgval))  # 0 bg .. 1 glyph
            if ink < 0.06:
                continue
            g = BG[0] + (PIXEL_MAX[0] - BG[0]) * ink
            db.fill(g, g, g)
            cy = box_y + box_h - (j + 1) * ch  # j=0 is the top row of the crop
            db.rect(box_x + i * cw, cy, cw + 0.5, ch + 0.5)

    # ── vector trace on top: fit the ON-CURVE outline to the box (computed
    # above); off-curve handles spill past it like an editor view. ──
    def tx(x, y):
        return (box_x + (x - fminx) / (fmaxx - fminx) * box_w,
                box_y + (y - fminy) / (fmaxy - fminy) * box_h)

    # outline
    db.fill(None)
    db.stroke(*OUTLINE)
    db.strokeWidth(LINE_WIDTH)
    db.lineJoin("round")
    for pts in cs:
        n = len(pts)
        start = next(i for i, p in enumerate(pts) if p[2])
        seq = [pts[(start + k) % n] for k in range(n)]
        path = db.BezierPath()
        path.moveTo(tx(seq[0][0], seq[0][1]))
        i = 1
        while i <= n:
            p = seq[i % n]
            if p[2]:
                path.lineTo(tx(p[0], p[1]))
                i += 1
            else:
                c1, c2, e = seq[i % n], seq[(i + 1) % n], seq[(i + 2) % n]
                path.curveTo(tx(c1[0], c1[1]), tx(c2[0], c2[1]), tx(e[0], e[1]))
                i += 3
        path.closePath()
        db.drawPath(path)

    # handle lines
    db.stroke(*HANDLE)
    db.strokeWidth(LINE_WIDTH)
    for pts in cs:
        n = len(pts)
        for i in range(n):
            if not pts[i][2]:
                continue
            for j in ((i - 1) % n, (i + 1) % n):
                if not pts[j][2]:
                    p = db.BezierPath()
                    p.moveTo(tx(pts[i][0], pts[i][1]))
                    p.lineTo(tx(pts[j][0], pts[j][1]))
                    db.drawPath(p)

    # points (hollow rings: bg fill + coloured stroke)
    db.strokeWidth(LINE_WIDTH)
    for pts in cs:
        for p in pts:
            x, y = tx(p[0], p[1])
            r = 24 if p[2] else 20
            db.fill(*BG)
            db.stroke(*(GREEN if p[2] else PURPLE))
            db.oval(x - r, y - r, 2 * r, 2 * r)

    OG.parent.mkdir(parents=True, exist_ok=True)
    db.saveImage(str(OG))
    db.endDrawing()
    # mirror into the content dir for the in-post hero (Astro image())
    Image.open(OG).save(HERO)
    print("saved", OG.relative_to(REPO), "and", HERO.relative_to(REPO))


if __name__ == "__main__":
    main()
