#!/usr/bin/env python3
"""Social export pipeline for the Virtua Grotesk post figures.

Reads index.mdx, finds every figure in post order, and produces a ready-
to-post kit in social-exports/ (gitignored; regenerate any time):

    social-exports/
      01-share-card.png        numbered in post order = a thread outline
      02-fig-system-no.png     pngquant-compressed (platforms recompress
      03-fig-system-ho.png     anyway; flat-color sheets palette-quantize
      ...
      alt-texts.md             beautifully) with each figure's alt text
                               from the MDX, for X / Mastodon / LinkedIn
                               alt fields

Masters are 2520x1320 = 1.91:1, exactly the X/LinkedIn card ratio, so
they post as-is. Run check_margins.py before exporting.

    python3 export_social.py

PHASE 4 PLACEHOLDER (post-publish promotion pass): square (2048x2048)
and vertical (1080x1920) NATIVE compositions per figure — see the note
at the bottom of src/lib.rs. This script will grow a --formats flag and
per-format subfolders; the numbering and alt-text extraction stay.
"""

import re
import shutil
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
POST = ROOT / "src/content/blog/virtua-grotesk"
OUT = ROOT / "social-exports"

PNGQUANT = Path.home() / "GH/repos/virtua-grotesk/.venv/bin/pngquant"


def main():
    mdx = (POST / "index.mdx").read_text()
    figures = re.findall(r"!\[(.*?)\]\(\./(.+?\.png)\)", mdx, flags=re.S)
    if not figures:
        sys.exit("no figures found in index.mdx")

    OUT.mkdir(exist_ok=True)
    for old in OUT.glob("*.png"):
        old.unlink()

    lines = ["# Virtua Grotesk post figures, in post order\n"]
    for i, (alt, name) in enumerate(figures, 1):
        src = POST / name
        stem = name.removesuffix(".png")
        dst = OUT / f"{i:02d}-{stem}.png"
        if PNGQUANT.exists():
            subprocess.run(
                [str(PNGQUANT), "--quality", "80-98", "--speed", "1",
                 "--force", "--output", str(dst), str(src)],
                check=True,
            )
        else:
            shutil.copy(src, dst)
        alt_clean = " ".join(alt.split())
        lines.append(f"## {dst.name}\n\n{alt_clean}\n")
        kb = dst.stat().st_size // 1024
        print(f"{dst.name:44s} {kb:5d} KB")

    (OUT / "alt-texts.md").write_text("\n".join(lines))
    print(f"\n{len(figures)} figures -> {OUT}")
    print("alt texts -> social-exports/alt-texts.md")


main()
