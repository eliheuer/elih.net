#!/usr/bin/env python3
"""Compare Mishkin-Qalam's Greatest Name composition with Gulzar."""

import argparse
import json
import os
import subprocess
import tempfile
import urllib.request
import xml.etree.ElementTree as ET
from pathlib import Path


SOURCE_URL = (
    "https://upload.wikimedia.org/wikipedia/commons/a/a9/"
    "Arabic_letters_in_the_Greatest_Name.svg"
)
TEXT = "يا بهاء الأبهى"

WIDTH = 2400
HEIGHT = 1100
BACKGROUND = "#0b0b0b"

# Colors from the source diagram, indexed by character position in TEXT.
COLORS = {
    0: "#ff6600",  # ي
    1: "#9ade00",  # ا
    3: "#4e9a06",  # ب
    4: "#f1caff",  # ه
    5: "#ccff42",  # ا
    6: "#ba00ff",  # ء
    8: "#ff4141",  # ا
    9: "#ffc022",  # ل
    10: "#19aeff",  # أ
    11: "#009100",  # ب
    12: "#f1caff",  # ه
    13: "#0084c8",  # ى
}

SVG_NS = "http://www.w3.org/2000/svg"
INKSCAPE_NS = "http://www.inkscape.org/namespaces/inkscape"

ET.register_namespace("", SVG_NS)
ET.register_namespace("xlink", "http://www.w3.org/1999/xlink")
ET.register_namespace("inkscape", INKSCAPE_NS)


def download_source(path: Path) -> None:
    if path.exists():
        return
    path.parent.mkdir(parents=True, exist_ok=True)
    urllib.request.urlretrieve(SOURCE_URL, path)


def run_harfbuzz(font_path: Path, svg_path: Path) -> tuple[ET.Element, list[int]]:
    env = os.environ.copy()
    env["LC_ALL"] = "en_US.UTF-8"
    common = [
        "--shapers=ot",
        "--direction=rtl",
        "--script=arab",
        "--language=ar",
    ]

    subprocess.run(
        [
            "hb-view",
            *common,
            "--font-size=400",
            "--margin=30",
            "--background=transparent",
            "--foreground=ffffff",
            "--output-format=svg",
            f"--output-file={svg_path}",
            str(font_path),
            TEXT,
        ],
        check=True,
        env=env,
    )
    shaped = subprocess.run(
        [
            "hb-shape",
            *common,
            "--output-format=json",
            str(font_path),
            TEXT,
        ],
        check=True,
        capture_output=True,
        text=True,
        env=env,
    )
    clusters = [glyph["cl"] for glyph in json.loads(shaped.stdout)]
    return ET.parse(svg_path).getroot(), clusters


def colored_gulzar(font_path: Path, temp_dir: Path) -> ET.Element:
    root, clusters = run_harfbuzz(font_path, temp_dir / "gulzar.svg")

    for rect in list(root.findall(f"{{{SVG_NS}}}rect")):
        root.remove(rect)

    uses = root.findall(f".//{{{SVG_NS}}}use")
    if len(uses) != len(clusters):
        raise RuntimeError(
            f"HarfBuzz returned {len(clusters)} glyphs but drew {len(uses)}"
        )
    for use, cluster in zip(uses, clusters):
        use.set("fill", COLORS.get(cluster, "#c5c5c5"))

    _, _, view_width, view_height = map(float, root.get("viewBox").split())
    scale = min(1100 / view_width, 1000 / view_height)
    x = 1240 + (1100 - view_width * scale) / 2
    y = 50 + (1000 - view_height * scale) / 2
    wrapper = ET.Element(
        f"{{{SVG_NS}}}g",
        {"transform": f"matrix({scale:g} 0 0 {scale:g} {x:g} {y:g})"},
    )
    for child in list(root):
        wrapper.append(child)
    return wrapper


def arabic_source(source_path: Path) -> ET.Element:
    source_root = ET.parse(source_path).getroot()
    arabic = next(
        group
        for group in source_root.findall(f"{{{SVG_NS}}}g")
        if group.get(f"{{{INKSCAPE_NS}}}label") == "arabic"
    )

    view_x, view_y, view_width, view_height = 7, 30, 1400, 940
    scale = min(1100 / view_width, 1000 / view_height)
    x = 30 + (1100 - view_width * scale) / 2 - view_x * scale
    y = 50 + (1000 - view_height * scale) / 2 - view_y * scale
    wrapper = ET.Element(
        f"{{{SVG_NS}}}g",
        {"transform": f"matrix({scale:g} 0 0 {scale:g} {x:g} {y:g})"},
    )
    wrapper.append(arabic)
    return wrapper


def render(source_path: Path, font_path: Path, output_path: Path) -> None:
    with tempfile.TemporaryDirectory() as directory:
        temp_dir = Path(directory)
        root = ET.Element(
            f"{{{SVG_NS}}}svg",
            {
                "width": str(WIDTH),
                "height": str(HEIGHT),
                "viewBox": f"0 0 {WIDTH} {HEIGHT}",
            },
        )
        ET.SubElement(
            root,
            f"{{{SVG_NS}}}rect",
            {
                "width": str(WIDTH),
                "height": str(HEIGHT),
                "fill": BACKGROUND,
            },
        )
        root.append(arabic_source(source_path))
        root.append(colored_gulzar(font_path, temp_dir))

        composed_svg = temp_dir / "greatest-name-comparison.svg"
        ET.ElementTree(root).write(
            composed_svg,
            encoding="utf-8",
            xml_declaration=True,
        )
        output_path.parent.mkdir(parents=True, exist_ok=True)
        subprocess.run(
            [
                "magick",
                "-background",
                BACKGROUND,
                str(composed_svg),
                "-strip",
                str(output_path),
            ],
            check=True,
        )


def main() -> None:
    repo = Path(__file__).resolve().parents[2]
    parser = argparse.ArgumentParser()
    parser.add_argument(
        "--source",
        type=Path,
        default=Path("/tmp/Arabic_letters_in_the_Greatest_Name.svg"),
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
        / "src/content/blog/nastaliq-distilled/greatest-name-comparison.png",
    )
    args = parser.parse_args()

    download_source(args.source)
    if not args.font.exists():
        raise SystemExit(f"Gulzar font not found: {args.font}")
    render(args.source, args.font, args.output)


if __name__ == "__main__":
    main()
