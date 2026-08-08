#!/usr/bin/env python3
"""Render the Gulzar and NeuralType word comparison as SVG."""

import json
import re
import subprocess
import sys
from pathlib import Path


WORDS = ["بسم", "الرحمن", "الرحيم", "نور", "نستعليق"]
WIDTH = 1800
HEIGHT = 720
OUTER_MARGIN = 60
CELL_PADDING = 20
ROW_HEIGHT = 240
ROW_CENTERS = (220, 500)


def translate(path_d, dx, dy):
    tokens = re.findall(r"[MLQCZ]|-?\d+\.?\d*(?:e-?\d+)?", path_d)
    output = []
    index = 0
    while index < len(tokens):
        command = tokens[index]
        index += 1
        value_count = {"M": 2, "L": 2, "Q": 4, "C": 6, "Z": 0}[command]
        output.append(command)
        for offset in range(0, value_count, 2):
            x = float(tokens[index + offset]) + dx
            y = float(tokens[index + offset + 1]) + dy
            output.append(f"{x:.1f} {y:.1f}")
        index += value_count
    return " ".join(output)


def scale_path(path_d, x_scale, y_scale):
    tokens = re.findall(r"[MLQCZ]|-?\d+\.?\d*(?:e-?\d+)?", path_d)
    output = []
    index = 0
    while index < len(tokens):
        command = tokens[index]
        index += 1
        value_count = {"M": 2, "L": 2, "Q": 4, "C": 6, "Z": 0}[command]
        output.append(command)
        for offset in range(0, value_count, 2):
            x = float(tokens[index + offset]) * x_scale
            y = float(tokens[index + offset + 1]) * y_scale
            output.append(f"{x:.3f} {y:.3f}")
        index += value_count
    return " ".join(output)


def bounds(path_d):
    values = [float(value) for value in re.findall(r"-?\d+\.?\d*(?:e-?\d+)?", path_d)]
    xs = values[0::2]
    ys = values[1::2]
    return min(xs), min(ys), max(xs), max(ys)


def load_teacher_words(source_repo):
    extract = source_repo / "data" / "extract-gulzar"
    glyphs = {}
    with (extract / "glyphs.jsonl").open() as source:
        for line in source:
            record = json.loads(line)
            glyphs[record["gid"]] = record["path"] or ""

    contexts = {}
    with (extract / "contexts.jsonl").open() as source:
        for line in source:
            record = json.loads(line)
            contexts.setdefault(record["word"], []).append(record)
    for records in contexts.values():
        records.sort(key=lambda record: record["index"])

    result = {}
    for word in WORDS:
        paths = []
        for record in contexts[word]:
            for glyph in record["glyphs"]:
                path_d = glyphs.get(glyph["gid"], "")
                if path_d:
                    paths.append(
                        translate(
                            path_d,
                            glyph["dx"] + record["ox"],
                            glyph["dy"] + record["oy"],
                        )
                    )
        result[word] = " ".join(paths)
    return result


def load_model_words(source_repo, model_path):
    distill = source_repo / "target" / "release" / "distill"
    result = {}
    for word in WORDS:
        process = subprocess.run(
            [distill, "wordjson", model_path, word],
            check=True,
            capture_output=True,
            text=True,
        )
        record = json.loads(process.stdout.strip().splitlines()[-1])
        font_units_per_pixel = record["upm"] / record["em_px"]
        result[word] = scale_path(
            record["d"], font_units_per_pixel, -font_units_per_pixel
        )
    return result


def centered_path(path_d, center_x, center_y, scale, color):
    x0, y0, x1, y1 = bounds(path_d)
    path_center_x = (x0 + x1) / 2
    path_center_y = (y0 + y1) / 2
    return (
        f'<g transform="translate({center_x:.2f} {center_y:.2f}) '
        f'scale({scale:.6f} {-scale:.6f}) '
        f'translate({-path_center_x:.2f} {-path_center_y:.2f})">'
        f'<path d="{path_d}" fill="{color}"/></g>'
    )


def render(teacher, model):
    column_width = (WIDTH - 2 * OUTER_MARGIN) / len(WORDS)
    max_width = column_width - 2 * CELL_PADDING
    max_height = ROW_HEIGHT - 2 * CELL_PADDING
    layouts = []

    for index, word in enumerate(WORDS):
        teacher_bounds = bounds(teacher[word])
        model_bounds = bounds(model[word])
        widths = [box[2] - box[0] for box in (teacher_bounds, model_bounds)]
        heights = [box[3] - box[1] for box in (teacher_bounds, model_bounds)]
        scale = min(max_width / max(widths), max_height / max(heights))
        center_x = OUTER_MARGIN + column_width * (index + 0.5)
        layouts.append((word, center_x, scale, max(widths)))

    left_edge = layouts[0][1] - layouts[0][3] * layouts[0][2] / 2
    right_edge = layouts[-1][1] + layouts[-1][3] * layouts[-1][2] / 2
    horizontal_shift = (WIDTH - left_edge - right_edge) / 2

    paths = []
    for word, center_x, scale, _ in layouts:
        center_x += horizontal_shift
        paths.append(
            centered_path(teacher[word], center_x, ROW_CENTERS[0], scale, "#6e6e6e")
        )
        paths.append(
            centered_path(model[word], center_x, ROW_CENTERS[1], scale, "#2aa35f")
        )

    return (
        f'<svg xmlns="http://www.w3.org/2000/svg" width="{WIDTH}" height="{HEIGHT}" '
        f'viewBox="0 0 {WIDTH} {HEIGHT}">\n'
        '<rect width="100%" height="100%" fill="#0c0c0c"/>\n'
        + "\n".join(paths)
        + "\n</svg>\n"
    )


def main():
    if len(sys.argv) != 2:
        raise SystemExit("usage: render_teacher_model.py <output.svg>")

    repo = Path(__file__).resolve().parents[2]
    source_repo = repo.parent / "post-opentype"
    model_path = repo / "public" / "demos" / "neuraltype" / "gulzar.ntf"
    teacher = load_teacher_words(source_repo)
    model = load_model_words(source_repo, model_path)
    Path(sys.argv[1]).write_text(render(teacher, model))


if __name__ == "__main__":
    main()
