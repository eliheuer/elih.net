#!/usr/bin/env python3
"""Render the Gulzar cascade figure from the extracted cluster records."""

import json
import re
import sys
from pathlib import Path


WORD = "نستعليق"
WIDTH = 572
HEIGHT = 508
RASTER_SCALE = 4
MARGIN = 28
BACKGROUND = "#0c0c0c"
GREEN = "#2aa35f"
GRAY = "#6e6e6e"
RED = "#ef4444"
METRIC_GUIDE = "#66ee88"
HASUBI_FONT_URL = (
    Path(__file__).resolve().parents[2] / "public/fonts/HasubiMono-Regular.woff2"
).as_uri()

# Gulzar's master metrics from sources/Gulzar.glyphs. The compiled font uses
# separate hhea and typo overrides for line spacing.
UNITS_PER_EM = 1000
ASCENDER = 800
DESCENDER = -800
X_HEIGHT = 500
CAP_HEIGHT = 700

# The first glyph in a Gulzar cluster is often a detached dot. These are the
# main glyphs for the seven clusters in WORD, verified against glyphs.jsonl.
EXPECTED_BASE_GIDS = [75, 641, 703, 40, 486, 686, 557]

def path_bounds(path_d):
    values = [float(value) for value in re.findall(r"-?\d+\.?\d*", path_d)]
    xs = values[0::2]
    ys = values[1::2]
    return min(xs), min(ys), max(xs), max(ys)


def load_data(source_repo):
    extract = source_repo / "data" / "extract-gulzar"
    glyph_paths = {}
    with (extract / "glyphs.jsonl").open() as source:
        for line in source:
            record = json.loads(line)
            glyph_paths[record["gid"]] = record["path"] or ""

    clusters = []
    with (extract / "contexts.jsonl").open() as source:
        for line in source:
            record = json.loads(line)
            if record["word"] == WORD:
                clusters.append(record)
    clusters.sort(key=lambda record: record["index"])

    base_gids = []
    for index, cluster in enumerate(clusters):
        base = next(
            glyph
            for glyph in cluster["glyphs"]
            if glyph["gid"] == EXPECTED_BASE_GIDS[index]
        )
        cluster["base"] = base
        base_gids.append(base["gid"])

    if base_gids != EXPECTED_BASE_GIDS:
        raise RuntimeError(f"unexpected base glyphs: {base_gids}")

    return glyph_paths, clusters


def render(glyph_paths, clusters):
    placed = []
    bounds = []
    base_origins = []

    for cluster in clusters:
        cluster_paths = []
        for glyph in cluster["glyphs"]:
            path_d = glyph_paths[glyph["gid"]]
            dx = cluster["ox"] + glyph["dx"]
            dy = cluster["oy"] + glyph["dy"]
            x0, y0, x1, y1 = path_bounds(path_d)
            bounds.append((x0 + dx, y0 + dy, x1 + dx, y1 + dy))
            cluster_paths.append((path_d, dx, dy))
        placed.append(cluster_paths)

        base = cluster["base"]
        base_x = cluster["ox"] + base["dx"]
        base_y = cluster["oy"] + base["dy"]
        base_origins.append((base_x, base_y))

    x0 = min(box[0] for box in bounds)
    y0 = min(min(box[1] for box in bounds), DESCENDER)
    x1 = max(box[2] for box in bounds)
    y1 = max(max(box[3] for box in bounds), ASCENDER)
    scale = min(
        (WIDTH - 2 * MARGIN) / (x1 - x0),
        (HEIGHT - 2 * MARGIN) / (y1 - y0),
    )
    tx = (WIDTH - (x1 - x0) * scale) / 2 - x0 * scale
    ty = (HEIGHT - (y1 - y0) * scale) / 2 + y1 * scale

    def point(x, y):
        return tx + x * scale, ty - y * scale

    elements = [f'<rect width="{WIDTH}" height="{HEIGHT}" fill="{BACKGROUND}"/>']

    # Draw the five master metrics shown in Gulzar's Glyphs source.
    metrics = (
        (UNITS_PER_EM, "UPM  1000", "start"),
        (ASCENDER, "ascender  +800", "start"),
        (CAP_HEIGHT, "cap height  +700", "start"),
        (X_HEIGHT, "x-height  +500", "start"),
        (0, "baseline  0", "end"),
        (DESCENDER, "descender  -800", "end"),
    )
    for height, label, anchor in metrics:
        y = point(0, height)[1]
        label_x = MARGIN + 4 if anchor == "start" else WIDTH - MARGIN - 4
        elements.append(
            f'<line x1="{MARGIN}" y1="{y:.3f}" '
            f'x2="{WIDTH - MARGIN}" y2="{y:.3f}" '
            f'stroke="{METRIC_GUIDE}" stroke-opacity="0.5" '
            'stroke-width="0.8"/>'
        )
        elements.append(
            f'<text x="{label_x}" y="{y - 4:.3f}" '
            f'fill="{METRIC_GUIDE}" fill-opacity="0.7" '
            f'font-family="Hasubi Mono" font-size="8" text-anchor="{anchor}">'
            f'{label}</text>'
        )

    for index, cluster_paths in enumerate(placed):
        color = GREEN if index % 2 == 0 else GRAY
        for path_d, dx, dy in cluster_paths:
            elements.append(
                f'<path d="{path_d}" fill="{color}" '
                f'transform="translate({tx + dx * scale:.6f} '
                f'{ty - dy * scale:.6f}) scale({scale:.8f} {-scale:.8f})"/>'
            )

    points = [point(x, y) for x, y in base_origins]
    for (x1, y1), (x2, y2) in zip(points, points[1:]):
        dx = x2 - x1
        dy = y2 - y1
        length = (dx * dx + dy * dy) ** 0.5
        nx = -dy * 0.75 / length
        ny = dx * 0.75 / length
        corners = (
            (x1 + nx, y1 + ny),
            (x2 + nx, y2 + ny),
            (x2 - nx, y2 - ny),
            (x1 - nx, y1 - ny),
        )
        corner_string = " ".join(f"{x:.3f},{y:.3f}" for x, y in corners)
        elements.append(
            f'<polygon points="{corner_string}" fill="{RED}"/>'
        )
    for x, y in points:
        elements.append(f'<circle cx="{x:.3f}" cy="{y:.3f}" r="3.5" fill="{RED}"/>')

    return (
        f'<svg xmlns="http://www.w3.org/2000/svg" '
        f'width="{WIDTH * RASTER_SCALE}" height="{HEIGHT * RASTER_SCALE}" '
        f'viewBox="0 0 {WIDTH} {HEIGHT}" shape-rendering="crispEdges">\n'
        f'<style>@font-face {{ font-family: "Hasubi Mono"; src: url("{HASUBI_FONT_URL}") format("woff2"); }}</style>\n'
        + "\n".join(elements)
        + "\n</svg>\n"
    )


def main():
    if len(sys.argv) != 2:
        raise SystemExit("usage: render_cascade.py <output.svg>")

    repo = Path(__file__).resolve().parents[2]
    source_repo = repo.parent / "post-opentype"
    glyph_paths, clusters = load_data(source_repo)
    Path(sys.argv[1]).write_text(render(glyph_paths, clusters))


if __name__ == "__main__":
    main()
