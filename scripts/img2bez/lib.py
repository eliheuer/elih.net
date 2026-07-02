"""Shared drawbot-skia helpers for the img2bez post's figures.

Post-local on purpose: this file is the single source of truth for *this
post's* look (palette, point markers, framing) and nothing outside
scripts/img2bez/ imports it. A future post that uses a different tool (e.g. the
designbot Rust renderer) or a different style gets its own scripts/<post>/
workspace and does not touch this.

Env: the repo-root .venv (has the eliheuer/drawbot-skia fork + Pillow).
Run figures from the repo root, e.g.:
    .venv/bin/python scripts/img2bez/fig_g_interp.py
"""
import json
import math
import subprocess
from pathlib import Path

from drawbot_skia.drawing import Drawing

# ── palette (Runebender-web point language) ───────────────────────────────
BG = (0.047, 0.047, 0.047)
OUTLINE = (0.90, 0.90, 0.90)
HANDLE = (0.42, 0.42, 0.46)          # handle lines (muted gray)
GREEN = (0.09, 0.72, 0.44)           # smooth on-curve  (#18b86f)
ORANGE = (1.00, 0.60, 0.06)          # corner on-curve  (#ff980f)
PURPLE = (0.55, 0.42, 1.00)          # off-curve        (#8c6cff)
POINT_INNER = (0.094, 0.094, 0.094)  # dark point fill  (#181818)


# ── outline model (img2bez `masters --format json` output) ────────────────
def load_masters(json_path):
    """Return {master_name: [contour, ...]}, each contour a list of point dicts
    {x, y, type?, smooth?} (no `type` == off-curve control)."""
    d = json.loads(Path(json_path).read_text())
    return {m["name"]: [c["points"] for c in m["outline"]["contours"]]
            for m in d["masters"]}


def lerp(a, b, t):
    return a + (b - a) * t


def interp_contours(reg, bold, t):
    """Linearly interpolate two compatible master outlines at parameter t."""
    out = []
    for cr, cb in zip(reg, bold):
        pts = []
        for a, b in zip(cr, cb):
            p = {"x": lerp(a["x"], b["x"], t), "y": lerp(a["y"], b["y"], t)}
            if a.get("type"):
                p["type"] = a["type"]
                p["smooth"] = a.get("smooth", False)
            pts.append(p)
        out.append(pts)
    return out


def union_bbox(contour_sets):
    xs, ys = [], []
    for contours in contour_sets:
        for c in contours:
            for p in c:
                xs.append(p["x"]); ys.append(p["y"])
    return min(xs), min(ys), max(xs), max(ys)


def frame_transform(bbox, size, margin=0.16):
    """Return (X, Y) mapping font units into a square `size` canvas (y-up),
    centering `bbox` with `margin` fractional padding."""
    minx, miny, maxx, maxy = bbox
    scale = (size * (1 - 2 * margin)) / max(maxx - minx, maxy - miny)
    cx, cy = (minx + maxx) / 2, (miny + maxy) / 2
    X = lambda x: size / 2 + (x - cx) * scale
    Y = lambda y: size / 2 + (y - cy) * scale
    return X, Y


# ── drawing ───────────────────────────────────────────────────────────────
def _first_oncurve(pts):
    for i, p in enumerate(pts):
        if p.get("type"):
            return i
    return 0


def draw_contour(db, pts, X, Y):
    """Append one closed contour (UFO point-pen order) to the current path."""
    n = len(pts)
    s = _first_oncurve(pts)
    seq = [pts[(s + k) % n] for k in range(n)]
    x0, y0 = X(seq[0]["x"]), Y(seq[0]["y"])
    db.moveTo((x0, y0))
    ctrl = []
    for p in seq[1:]:
        if p.get("type"):
            if len(ctrl) == 2:
                db.curveTo((X(ctrl[0]["x"]), Y(ctrl[0]["y"])),
                           (X(ctrl[1]["x"]), Y(ctrl[1]["y"])),
                           (X(p["x"]), Y(p["y"])))
            else:
                db.lineTo((X(p["x"]), Y(p["y"])))
            ctrl = []
        else:
            ctrl.append(p)
    if len(ctrl) == 2:
        db.curveTo((X(ctrl[0]["x"]), Y(ctrl[0]["y"])),
                   (X(ctrl[1]["x"]), Y(ctrl[1]["y"])), (x0, y0))
    db.closePath()


def draw_points(db, pts, X, Y, r=4.2, sw=3.6):
    """Runebender-web point language: smooth on-curve = green circle, corner =
    orange square, off-curve = purple circle; each a dark-filled shape with a
    colored ring, joined by gray handle lines. Identical for any outline, so the
    only thing that differs between traces is where the points sit."""
    n = len(pts)
    # handle lines
    db.stroke(*HANDLE); db.strokeWidth(1.2); db.fill(None)
    for i, p in enumerate(pts):
        if not p.get("type"):
            for j in (i - 1, i + 1):
                q = pts[j % n]
                if q.get("type"):
                    db.newPath()
                    db.moveTo((X(p["x"]), Y(p["y"])))
                    db.lineTo((X(q["x"]), Y(q["y"])))
                    db.drawPath()
    # points: dark fill + colored ring; circle (smooth/off-curve) or square (corner)
    for p in pts:
        cx, cy = X(p["x"]), Y(p["y"])
        db.fill(*POINT_INNER); db.strokeWidth(sw)
        if not p.get("type"):
            db.stroke(*PURPLE); db.oval(cx - r, cy - r, 2 * r, 2 * r)
        elif p.get("smooth"):
            db.stroke(*GREEN); db.oval(cx - r, cy - r, 2 * r, 2 * r)
        else:
            db.stroke(*ORANGE); db.rect(cx - r, cy - r, 2 * r, 2 * r)


def render_frame(contours, path, X, Y, size=640, fill_alpha=0.10,
                 stroke_w=2.0):
    """Render one frame. X, Y is a fixed transform (from frame_transform) so a
    whole animation shares one framing and the glyph never drifts."""
    db = Drawing()
    db.size(size, size)
    db.fill(*BG); db.rect(0, 0, size, size)
    # subtle fill
    db.fill(*OUTLINE, fill_alpha); db.stroke(None); db.newPath()
    for pts in contours:
        draw_contour(db, pts, X, Y)
    db.drawPath()
    # outline stroke
    db.fill(None); db.stroke(*OUTLINE); db.strokeWidth(stroke_w); db.newPath()
    for pts in contours:
        draw_contour(db, pts, X, Y)
    db.drawPath()
    # markers
    for pts in contours:
        draw_points(db, pts, X, Y)
    db.saveImage(str(path))


# ── animation ──────────────────────────────────────────────────────────────
def ease_pingpong(i, n):
    """Cosine ping-pong 0->1->0 over n frames; eases in/out, loops seamlessly."""
    return 0.5 - 0.5 * math.cos(2 * math.pi * i / n)


def frames_to_mp4(frames_glob_dir, out_path, fps=24):
    Path(out_path).parent.mkdir(parents=True, exist_ok=True)
    subprocess.run(
        ["ffmpeg", "-y", "-framerate", str(fps),
         "-i", str(Path(frames_glob_dir) / "f%03d.png"),
         "-vf", "format=yuv420p", "-c:v", "libx264", "-crf", "20",
         "-movflags", "+faststart", str(out_path), "-loglevel", "error"],
        check=True,
    )
