#!/usr/bin/env python3
"""Render the neural-network explainer figures for Nasta'liq Distilled."""

from __future__ import annotations

import json
import math
from pathlib import Path

from PIL import Image, ImageChops, ImageDraw, ImageFilter, ImageFont


WIDTH = 2400
HEIGHT = 1260
BG = "#0b0b0b"
PANEL = "#111111"
PANEL_ALT = "#181818"
BORDER = "#383838"
GRID = "#303030"
TEXT = "#c8c8c8"
MUTED = "#858585"
GREEN = "#2aa35f"
GREEN_LIGHT = "#42c978"
RED = "#ff4d4d"
FONT_PATH = "/System/Library/Fonts/SFNS.ttf"
FONT_BOLD_PATH = "/System/Library/Fonts/SFNS.ttf"
ARABIC_FONT_PATH = "/System/Library/Fonts/Supplemental/Arial Unicode.ttf"


def font(size: int, bold: bool = False) -> ImageFont.FreeTypeFont:
    return ImageFont.truetype(FONT_BOLD_PATH if bold else FONT_PATH, size)


def arabic_font(size: int) -> ImageFont.FreeTypeFont:
    return ImageFont.truetype(ARABIC_FONT_PATH, size)


def canvas(height: int = HEIGHT) -> tuple[Image.Image, ImageDraw.ImageDraw]:
    image = Image.new("RGB", (WIDTH, height), BG)
    return image, ImageDraw.Draw(image)


def text_center(draw: ImageDraw.ImageDraw, xy: tuple[float, float], value: str, size: int,
                color: str = TEXT, bold: bool = False) -> None:
    draw.text(xy, value, font=font(size, bold), fill=color, anchor="mm")


def text_left(draw: ImageDraw.ImageDraw, xy: tuple[float, float], value: str, size: int,
              color: str = TEXT, bold: bool = False) -> None:
    draw.text(xy, value, font=font(size, bold), fill=color, anchor="lm")


def box(draw: ImageDraw.ImageDraw, bounds: tuple[int, int, int, int], radius: int = 24,
        fill: str = PANEL, outline: str = BORDER, width: int = 3) -> None:
    draw.rounded_rectangle(bounds, radius=radius, fill=fill, outline=outline, width=width)


def arrow(draw: ImageDraw.ImageDraw, start: tuple[float, float], end: tuple[float, float],
          color: str = MUTED, width: int = 6, head: int = 18) -> None:
    draw.line((start, end), fill=color, width=width)
    angle = math.atan2(end[1] - start[1], end[0] - start[0])
    left = (
        end[0] - head * math.cos(angle - math.pi / 6),
        end[1] - head * math.sin(angle - math.pi / 6),
    )
    right = (
        end[0] - head * math.cos(angle + math.pi / 6),
        end[1] - head * math.sin(angle + math.pi / 6),
    )
    draw.polygon((end, left, right), fill=color)


def letter_row(draw: ImageDraw.ImageDraw, x: int, y: int, tile: int = 96,
               gap: int = 16, small: bool = False) -> tuple[int, int, int, int]:
    letters = ["ن", "س", "ت", "ع", "ل"]
    total = len(letters) * tile + (len(letters) - 1) * gap
    for index, letter in enumerate(letters):
        x0 = x + index * (tile + gap)
        active = index == 2
        draw.rounded_rectangle(
            (x0, y, x0 + tile, y + tile),
            radius=14,
            fill="#173323" if active else PANEL_ALT,
            outline=GREEN if active else BORDER,
            width=3,
        )
        draw.text(
            (x0 + tile / 2, y + tile / 2 - (2 if small else 4)),
            letter,
            font=arabic_font(round(tile * (0.5 if small else 0.62))),
            fill=GREEN_LIGHT if active else TEXT,
            anchor="mm",
        )
    return x, y, x + total, y + tile


def dot_code(draw: ImageDraw.ImageDraw, x: int, y: int, cols: int = 16,
             rows: int = 16, spacing: int = 15, radius: int = 4) -> None:
    for row in range(rows):
        for col in range(cols):
            value = math.sin((row + 1) * 1.7 + (col + 1) * 2.3)
            color = GREEN if value > 0.48 else ("#666666" if value > -0.2 else "#303030")
            cx = x + col * spacing
            cy = y + row * spacing
            draw.ellipse((cx - radius, cy - radius, cx + radius, cy + radius), fill=color)


def mini_grid(draw: ImageDraw.ImageDraw, bounds: tuple[int, int, int, int], cells: int = 6,
              accent: bool = False) -> None:
    x0, y0, x1, y1 = bounds
    draw.rectangle(bounds, fill=PANEL_ALT, outline=GREEN if accent else BORDER, width=3)
    for i in range(1, cells):
        x = x0 + (x1 - x0) * i / cells
        y = y0 + (y1 - y0) * i / cells
        draw.line((x, y0, x, y1), fill=GRID, width=2)
        draw.line((x0, y, x1, y), fill=GRID, width=2)


def load_field() -> Image.Image:
    repo = Path(__file__).resolve().parents[2]
    fields_dir = repo.parent / "post-opentype" / "data" / "fields-gulzar-64"
    meta = json.loads((fields_dir / "fields-meta.json").read_text())
    w, h = int(meta["w"]), int(meta["h"])
    shape = 6
    raw = (fields_dir / "fields.bin").read_bytes()[shape * w * h:(shape + 1) * w * h]
    image = Image.new("L", (w, h))
    mapped = []
    for value in raw:
        if value < 128:
            mapped.append(round(18 + 72 * value / 128))
        else:
            mapped.append(round(132 + 88 * (value - 128) / 79))
    image.putdata(mapped)
    return image


def paste_field(base: Image.Image, field: Image.Image, bounds: tuple[int, int, int, int],
                nearest: bool = True, border: bool = True) -> None:
    x0, y0, x1, y1 = bounds
    resized = field.resize(
        (x1 - x0, y1 - y0),
        Image.Resampling.NEAREST if nearest else Image.Resampling.BICUBIC,
    ).convert("RGB")
    base.paste(resized, (x0, y0))
    if border:
        ImageDraw.Draw(base).rectangle(bounds, outline=BORDER, width=3)


def architecture(out: Path, field: Image.Image) -> None:
    image, draw = canvas(1800)
    text_center(draw, (WIDTH / 2, 100), "One small network, two outputs", 68, bold=True)
    text_center(draw, (WIDTH / 2, 175), "The same weights are reused for every letter.", 44, MUTED)

    stages = [(80, 245, 1160, 955), (1240, 245, 2320, 955),
              (80, 1025, 1160, 1735), (1240, 1025, 2320, 1735)]
    for stage in stages:
        box(draw, stage)

    text_center(draw, (620, 315), "1. Read nearby letters", 54, bold=True)
    letter_row(draw, 200, 405, tile=140, gap=35)
    text_center(draw, (620, 610), "Each letter is replaced by", 43, MUTED)
    text_center(draw, (620, 668), "24 learned numbers.", 43, TEXT)
    for i in range(5):
        x = 245 + i * 175
        for j in range(9):
            color = GREEN if (i * 3 + j * 5) % 7 < 2 else "#595959"
            draw.ellipse((x + j * 12, 755, x + j * 12 + 8, 763), fill=color)
    text_center(draw, (620, 850), "The middle letter is the shape being drawn.", 42, MUTED)

    text_center(draw, (1780, 315), "2. Combine them", 54, bold=True)
    dot_code(draw, 1615, 405, spacing=22, radius=6)
    text_center(draw, (1780, 800), "256 numbers", 50, GREEN_LIGHT, bold=True)
    text_center(draw, (1780, 860), "describe the shape in this context.", 42, MUTED)

    text_center(draw, (620, 1095), "3. Grow the field", 54, bold=True)
    sizes = [(86, 120), (116, 160), (154, 214), (202, 280), (270, 375)]
    start_x = 115
    baseline = 1500
    for index, (w, h) in enumerate(sizes):
        x0 = start_x + index * 190
        y0 = baseline - h
        mini_grid(draw, (x0, y0, x0 + w, baseline), cells=5, accent=index == len(sizes) - 1)
        if index < len(sizes) - 1:
            arrow(draw, (x0 + w + 8, baseline - h / 2),
                  (start_x + (index + 1) * 190 - 8, baseline - sizes[index + 1][1] / 2),
                  width=4, head=12)
    text_center(draw, (620, 1600), "Five layers double the size and add finer detail.", 42, MUTED)

    text_center(draw, (1780, 1095), "4. Draw and place", 54, bold=True)
    paste_field(image, field, (1400, 1165, 1720, 1617))
    draw.ellipse((1920, 1375, 1946, 1401), fill=RED)
    arrow(draw, (1933, 1388), (2165, 1235), GREEN, width=10, head=28)
    text_center(draw, (1560, 1660), "shape field", 39, TEXT)
    text_center(draw, (2050, 1660), "x/y position", 39, TEXT)

    arrow(draw, (1170, 600), (1230, 600), GREEN, width=8, head=24)
    arrow(draw, (1170, 1380), (1230, 1380), GREEN, width=8, head=24)
    arrow(draw, (1780, 970), (620, 1010), GREEN, width=7, head=22)

    image.save(out)


def training(out: Path, field: Image.Image) -> None:
    image, draw = canvas(1800)
    text_center(draw, (WIDTH / 2, 155), "Training is repeated correction", 68, bold=True)
    text_center(draw, (WIDTH / 2, 225), "Gulzar supplies the answer. The network learns to match it.", 44, MUTED)

    target = field
    prediction = ImageChops.offset(field.filter(ImageFilter.GaussianBlur(2.2)), 4, -3)
    difference = ImageChops.difference(target, prediction)
    heat = Image.new("RGB", difference.size, BG)
    diff_pixels = list(difference.get_flattened_data())
    heat.putdata([(min(255, 25 + v * 5), 24, 24) if v > 2 else (16, 16, 16) for v in diff_pixels])

    panels = [(80, 245, 1160, 955), (1240, 245, 2320, 955),
              (80, 1025, 1160, 1735), (1240, 1025, 2320, 1735)]
    for panel in panels:
        box(draw, panel)

    text_center(draw, (620, 315), "1. Gulzar target", 54, bold=True)
    paste_field(image, target, (245, 385, 620, 914))
    text_left(draw, (690, 590), "The correct field", 42, MUTED)
    text_left(draw, (690, 650), "and position come", 42, MUTED)
    text_left(draw, (690, 710), "from shaping.", 42, MUTED)

    text_center(draw, (1780, 315), "2. Model prediction", 54, bold=True)
    paste_field(image, prediction, (1405, 385, 1780, 914), nearest=False)
    text_left(draw, (1850, 610), "The first", 42, MUTED)
    text_left(draw, (1850, 670), "predictions", 42, MUTED)
    text_left(draw, (1850, 730), "are wrong.", 42, MUTED)

    text_center(draw, (620, 1095), "3. Measure the error", 54, bold=True)
    paste_field(image, heat.convert("L"), (245, 1165, 620, 1694), nearest=False)
    # Restore a red heat map after paste_field's grayscale conversion.
    image.paste(heat.resize((375, 529), Image.Resampling.BICUBIC), (245, 1165))
    draw.rectangle((245, 1165, 620, 1694), outline=BORDER, width=3)
    text_left(draw, (690, 1370), "Brighter cells", 42, MUTED)
    text_left(draw, (690, 1430), "differ more.", 42, MUTED)
    text_left(draw, (690, 1510), "Placement error", 42, MUTED)
    text_left(draw, (690, 1570), "is included.", 42, MUTED)

    text_center(draw, (1780, 1095), "4. Adjust the weights", 54, bold=True)
    nodes = [(1510, 1250), (2050, 1250), (1400, 1415), (1780, 1415), (2160, 1415),
             (1510, 1580), (2050, 1580)]
    edges = [(0, 2), (0, 3), (1, 3), (1, 4), (2, 5), (3, 5), (3, 6), (4, 6)]
    for a, b in edges:
        draw.line((nodes[a], nodes[b]), fill="#505050", width=7)
    for index, (x, y) in enumerate(nodes):
        color = GREEN if index in (0, 3, 6) else "#777777"
        draw.ellipse((x - 20, y - 20, x + 20, y + 20), fill=color)
    text_center(draw, (1780, 1670), "Reduce the error, then repeat.", 42, MUTED)

    arrow(draw, (1170, 600), (1230, 600), GREEN, width=8, head=24)
    arrow(draw, (1170, 1380), (1230, 1380), GREEN, width=8, head=24)
    arrow(draw, (1780, 970), (620, 1010), GREEN, width=7, head=22)

    image.save(out)


def future(out: Path, field: Image.Image) -> None:
    image, draw = canvas()
    text_center(draw, (WIDTH / 2, 115), "The architecture can change", 62, bold=True)
    text_center(draw, (WIDTH / 2, 180), "The current model is a baseline, not a limit of NeuralType.", 38, MUTED)

    left = (90, 235, 1160, 1120)
    right = (1240, 235, 2310, 1120)
    box(draw, left)
    box(draw, right)
    text_center(draw, (625, 300), "Current distilled model", 52, bold=True)
    text_center(draw, (1775, 300), "Possible native model", 52, bold=True)

    letter_row(draw, 345, 390, tile=96, gap=14)
    text_center(draw, (625, 535), "Nearby letters", 40, MUTED)
    arrow(draw, (625, 575), (625, 650), GREEN, width=6, head=18)
    box(draw, (425, 670, 825, 770), radius=18, fill=PANEL_ALT)
    text_center(draw, (625, 720), "fixed shape code", 44, TEXT)
    arrow(draw, (625, 790), (625, 850), GREEN, width=6, head=18)
    paste_field(image, field, (520, 865, 730, 1085))
    text_left(draw, (755, 935), "one fixed grid", 38, MUTED)
    text_left(draw, (755, 985), "for each shape", 38, MUTED)

    # Whole-line context: twelve compact letter blocks connected to a shared encoder.
    start_x = 1348
    for i in range(12):
        x0 = start_x + i * 71
        active = i in (2, 5, 8, 10)
        draw.rounded_rectangle((x0, 390, x0 + 52, 455), radius=10,
                               fill="#173323" if active else PANEL_ALT,
                               outline=GREEN if active else BORDER, width=2)
    for i in range(12):
        x = start_x + i * 71 + 26
        draw.line((x, 468, 1775, 650), fill="#315641" if i in (2, 5, 8, 10) else "#333333", width=3)
    draw.rounded_rectangle((1575, 482, 1975, 542), radius=14, fill=PANEL)
    text_center(draw, (1775, 512), "Whole line or page", 40, MUTED)
    box(draw, (1515, 650, 2035, 760), radius=18, fill=PANEL_ALT, outline=GREEN)
    text_center(draw, (1775, 705), "shared composition model", 44, TEXT)
    arrow(draw, (1775, 780), (1775, 845), GREEN, width=6, head=18)

    draw.rectangle((1500, 865, 1800, 1070), fill=PANEL_ALT, outline=BORDER, width=3)
    for i in range(1, 8):
        x = 1500 + i * 300 / 8
        draw.line((x, 865, x, 1070), fill=GRID, width=2)
    for i in range(1, 6):
        y = 865 + i * 205 / 6
        draw.line((1500, y, 1800, y), fill=GRID, width=2)
    curve = [(1530, 1015), (1560, 985), (1590, 945), (1620, 910),
             (1650, 900), (1680, 925), (1710, 960), (1740, 972), (1770, 940)]
    draw.line(curve, fill=GREEN, width=7, joint="curve")
    for i, (x, y) in enumerate(curve):
        draw.ellipse((x - 7, y - 7, x + 7, y + 7), fill=RED if i in (0, 4, 8) else GREEN_LIGHT)
    text_left(draw, (1840, 925), "query any x/y point", 38, MUTED)
    text_left(draw, (1840, 975), "at any resolution", 38, MUTED)
    text_left(draw, (1840, 1025), "across the composition", 38, MUTED)

    image.save(out)


def main() -> None:
    repo = Path(__file__).resolve().parents[2]
    out_dir = repo / "src" / "content" / "blog" / "nastaliq-distilled"
    field = load_field()
    architecture(out_dir / "model-architecture.png", field)
    training(out_dir / "training-loop.png", field)
    future(out_dir / "future-architecture.png", field)


if __name__ == "__main__":
    main()
