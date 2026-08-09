#!/usr/bin/env python3
"""Compare Mir Emad Hassani's Al-Fatiha composition with Gulzar."""

import argparse
import os
import subprocess
import tempfile
import urllib.request
from pathlib import Path

from PIL import Image, ImageOps


SOURCE_URL = (
    "https://upload.wikimedia.org/wikipedia/commons/9/92/"
    "Nasta%27liq_calligraphy_style_-_Mir_Emad_Hassani_09.png"
)

LINES = [
    "بسم الله الرحمن الرحيم",
    "الحمد لله رب العالمين الرحمن الرحيم",
    "مالك يوم الدين اياك نعبد",
    "واياك نستعين اهدنا الصراط المستقيم",
    "صراط الذين انعمت عليهم غير المغضوب",
    "عليهم ولا الضالين",
]

WIDTH = 2800
HEIGHT = 1800
HALF_WIDTH = WIDTH // 2

BACKGROUND = "#0b0b0b"
TYPE_COLOR = "#c5c5c5"
RENDER_INK = "000000"


def download_source(path: Path) -> None:
    if path.exists():
        return
    path.parent.mkdir(parents=True, exist_ok=True)
    urllib.request.urlretrieve(SOURCE_URL, path)


def render_line(font_path: Path, text: str, output_path: Path) -> Image.Image:
    env = os.environ.copy()
    env["LC_ALL"] = "en_US.UTF-8"
    subprocess.run(
        [
            "hb-view",
            "--shapers=ot",
            "--direction=rtl",
            "--script=arab",
            "--language=ar",
            "--font-size=102",
            "--margin=8",
            "--background=transparent",
            f"--foreground={RENDER_INK}",
            "--output-format=png",
            f"--output-file={output_path}",
            str(font_path),
            text,
        ],
        check=True,
        env=env,
    )
    grayscale = Image.open(output_path).convert("L")
    alpha = ImageOps.invert(grayscale)
    line = Image.new("RGBA", grayscale.size, TYPE_COLOR)
    line.putalpha(alpha)
    return line


def render(source_path: Path, font_path: Path, output_path: Path) -> None:
    manuscript = Image.open(source_path).convert("RGB")

    canvas = Image.new("RGB", (WIDTH, HEIGHT), BACKGROUND)
    manuscript_fitted = ImageOps.fit(
        manuscript,
        (HALF_WIDTH, HEIGHT),
        method=Image.Resampling.LANCZOS,
        centering=(0.5, 0.5),
    )
    canvas.paste(manuscript_fitted, (0, 0))

    line_area_top = 70
    line_area_bottom = HEIGHT - 70
    line_slot = (line_area_bottom - line_area_top) / len(LINES)
    max_line_width = HALF_WIDTH - 150

    with tempfile.TemporaryDirectory() as temp_dir:
        temp_path = Path(temp_dir)
        for index, text in enumerate(LINES):
            line = render_line(font_path, text, temp_path / f"line-{index}.png")
            if line.width > max_line_width:
                scale = max_line_width / line.width
                line = line.resize(
                    (round(line.width * scale), round(line.height * scale)),
                    Image.Resampling.LANCZOS,
                )
            center_x = HALF_WIDTH + HALF_WIDTH // 2
            center_y = line_area_top + line_slot * (index + 0.5)
            canvas.paste(
                line,
                (round(center_x - line.width / 2), round(center_y - line.height / 2)),
                line,
            )

    output_path.parent.mkdir(parents=True, exist_ok=True)
    canvas.save(output_path, optimize=True)


def main() -> None:
    repo = Path(__file__).resolve().parents[2]
    parser = argparse.ArgumentParser()
    parser.add_argument(
        "--source",
        type=Path,
        default=Path("/tmp/mir-emad-al-fatiha.png"),
    )
    parser.add_argument(
        "--font",
        type=Path,
        default=repo.parent / "google-fonts/ofl/gulzar/Gulzar-Regular.ttf",
    )
    parser.add_argument(
        "--output",
        type=Path,
        default=repo
        / "src/content/blog/nastaliq-distilled/fatiha-gulzar-comparison.png",
    )
    args = parser.parse_args()

    download_source(args.source)
    if not args.font.exists():
        raise SystemExit(f"Gulzar font not found: {args.font}")
    render(args.source, args.font, args.output)


if __name__ == "__main__":
    main()
