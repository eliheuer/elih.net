"""vtracer-side machinery for the comparison figures.

Parses vtracer's SVG output into dense polylines, reconciles a Regular/Bold
pair into equal-length aligned rings (vtracer has no notion of a master set,
so this does for it what it cannot do itself — resample + cyclic alignment),
and reconstructs a smooth cubic point structure so both panels of a comparison
render in the same Runebender point language.

Needs the `vtracer` CLI on PATH (cargo install vtracer). The settings used for
the comparisons are vtracer's cleanest for glyph work:
    vtracer --input g.png --output g.svg --colormode bw --mode spline \
        -f 10 -c 60 -l 10 -s 135
"""
import math
import re
import subprocess
from pathlib import Path

# Contours larger than this (px) in both axes are the image-frame rectangle
# vtracer traces around the canvas; smaller than SPECKLE in both axes are
# stray artifacts. Both are dropped.
FRAME_MIN = 1150
SPECKLE_MAX = 150


def trace(png, svg):
    """Run the vtracer CLI at the comparison settings."""
    subprocess.run(
        ["vtracer", "--input", str(png), "--output", str(svg),
         "--colormode", "bw", "--mode", "spline",
         "-f", "10", "-c", "60", "-l", "10", "-s", "135"],
        check=True, capture_output=True,
    )


def native_on_curve(svg):
    """vtracer's own node count for the glyph (M/L/C commands, frame and
    speckles excluded) — the honest number for a point-count label."""
    count = 0
    for d in _glyph_path_data(svg):
        count += len(re.findall(r"[MLC]", d))
    return count


def parse_contours(svg):
    """Glyph contours as dense y-up polylines (curves flattened)."""
    out = []
    for d in _glyph_path_data(svg):
        pts = _flatten(d)
        if len(pts) >= 4:
            out.append(pts)
    return out


def _glyph_path_data(svg):
    for d in re.findall(r'<path[^>]*\bd="([^"]+)"', Path(svg).read_text()):
        nums = [float(x) for x in re.findall(r"-?\d+\.?\d*", d)]
        xs, ys = nums[0::2], nums[1::2]
        if not xs:
            continue
        w, h = max(xs) - min(xs), max(ys) - min(ys)
        if w > FRAME_MIN and h > FRAME_MIN:
            continue
        if w < SPECKLE_MAX and h < SPECKLE_MAX:
            continue
        yield d


def _flatten(d, steps=16):
    """Flatten an absolute M/L/C/Z path to a y-up polyline."""
    toks = re.findall(r"[MLCZ]|-?\d+\.?\d*", d)
    i = 0
    cur = (0.0, 0.0)
    start = None
    out = []
    while i < len(toks):
        c = toks[i]
        i += 1
        if c == "M":
            x, y = float(toks[i]), float(toks[i + 1])
            i += 2
            cur = (x, y)
            start = cur
            out.append((x, -y))
        elif c == "L":
            x, y = float(toks[i]), float(toks[i + 1])
            i += 2
            out.append((x, -y))
            cur = (x, y)
        elif c == "C":
            x1, y1, x2, y2, x, y = (float(toks[i + k]) for k in range(6))
            i += 6
            p0 = cur
            for s in range(1, steps + 1):
                t = s / steps
                mt = 1 - t
                bx = mt**3 * p0[0] + 3 * mt * mt * t * x1 + 3 * mt * t * t * x2 + t**3 * x
                by = mt**3 * p0[1] + 3 * mt * mt * t * y1 + 3 * mt * t * t * y2 + t**3 * y
                out.append((bx, -by))
            cur = (x, y)
        elif c == "Z":
            cur = start
    return out


def resample(poly, n):
    """Uniform arc-length resample of a closed polyline to n points."""
    P = list(poly)
    if P[0] != P[-1]:
        P = P + [P[0]]
    seg = [math.dist(P[k], P[k + 1]) for k in range(len(P) - 1)]
    total = sum(seg)
    step = total / n
    out = []
    k = 0
    acc = 0.0
    for j in range(n):
        target = j * step
        while k < len(seg) - 1 and acc + seg[k] < target:
            acc += seg[k]
            k += 1
        r = (target - acc) / seg[k] if seg[k] else 0
        out.append((P[k][0] + r * (P[k + 1][0] - P[k][0]),
                    P[k][1] + r * (P[k + 1][1] - P[k][1])))
    return out


def _anchor_top(poly):
    top = max(range(len(poly)), key=lambda k: poly[k][1])
    return poly[top:] + poly[:top]


def reconcile(reg, bold, n):
    """Match contours by area, resample both to n points, anchor at the top —
    the fair-as-possible correspondence for outlines that share no point set."""
    def area(p):
        return abs(sum(p[k][0] * p[(k + 1) % len(p)][1]
                       - p[(k + 1) % len(p)][0] * p[k][1]
                       for k in range(len(p)))) / 2
    reg = sorted(reg, key=area, reverse=True)
    bold = sorted(bold, key=area, reverse=True)
    return [(_anchor_top(resample(cr, n)), _anchor_top(resample(cb, n)))
            for cr, cb in zip(reg, bold)]


def _ang(v1, v2):
    a = math.atan2(v1[1], v1[0])
    b = math.atan2(v2[1], v2[0])
    d = abs(math.degrees(b - a)) % 360
    return d if d <= 180 else 360 - d


def to_cubic(poly, corner_deg=42, k=1 / 3.0):
    """Reconstruct a smooth cubic contour (img2bez point format) through a
    resampled polyline: on-curve points with smooth/corner flags plus two
    Catmull-Rom-tangent handles per segment — so vtracer's outline renders in
    the same point language as img2bez's."""
    n = len(poly)
    tang = []
    for i in range(n):
        a = poly[(i - 1) % n]
        c = poly[(i + 1) % n]
        tx, ty = c[0] - a[0], c[1] - a[1]
        L = math.hypot(tx, ty) or 1.0
        tang.append((tx / L, ty / L))
    out = []
    for i in range(n):
        b = poly[i]
        nb = poly[(i + 1) % n]
        a = poly[(i - 1) % n]
        smooth = _ang((b[0] - a[0], b[1] - a[1]),
                      (nb[0] - b[0], nb[1] - b[1])) < corner_deg
        out.append({"x": b[0], "y": b[1], "type": "curve", "smooth": smooth})
        d = math.hypot(nb[0] - b[0], nb[1] - b[1]) * k
        out.append({"x": b[0] + tang[i][0] * d, "y": b[1] + tang[i][1] * d})
        out.append({"x": nb[0] - tang[(i + 1) % n][0] * d,
                    "y": nb[1] - tang[(i + 1) % n][1] * d})
    return out
