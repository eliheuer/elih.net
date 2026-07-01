# img2bez post — figure sources

Generative sources for the illustrations in `src/content/blog/img2bez/`. This
directory is **self-contained and post-local**: its `lib.py` defines *this
post's* look and nothing outside `scripts/img2bez/` imports it. A future post
with a different style — or a different tool, e.g. the **designbot** Rust
renderer replacing drawbot-skia — gets its own `scripts/<post>/` workspace with
its own toolchain. The site itself is not tied to Python; each post declares its
own (here, `requirements.txt`).

## Layout

```
scripts/img2bez/
  lib.py             shared helpers: palette, outline parsing, framing, anim
  fig_g_interp.py    figure: G interpolating Regular <-> Bold  (section 03)
  inputs/            raw source rasters (ultimate source)
  g-masters.json     cached img2bez trace (geometry source of truth for figures)
  build/             frame scratch (gitignored)
  requirements.txt   this post's Python deps
```

**Outputs land where the post consumes them**, not here: still images →
`src/content/blog/img2bez/*.png` (Astro-optimized); video →
`public/demos/img2bez/*.mp4`.

## Running

Uses the repo-root `.venv` (has the eliheuer/drawbot-skia fork + Pillow) and
needs `ffmpeg` on PATH. From the repo root:

```sh
.venv/bin/python scripts/img2bez/fig_g_interp.py   # -> public/demos/img2bez/g-interp.mp4
```

## Regenerating the trace snapshot

`g-masters.json` is a cached `img2bez` trace, so tweaking a figure's *style* or
*motion* needs no re-trace. To re-trace from the raw rasters (needs the
`img2bez` CLI):

```sh
img2bez masters <designspace with Regular+Bold masters> \
  --glyph G --unicode 0047 \
  --image Regular=scripts/img2bez/inputs/G-regular.png \
  --image Bold=scripts/img2bez/inputs/G-bold.png \
  --fit descender:cap --format json > scripts/img2bez/g-masters.json
```

TODO (self-containment): the `masters` subcommand needs a designspace for
metrics. Right now that's borrowed from an external font repo. Ship a tiny stub
designspace + two metric-only master UFOs under `inputs/` so re-tracing depends
only on this repo.

## Conventions for new figures

- Add `fig_<name>.py`, import `lib`, keep raw inputs in `inputs/`.
- `color=True` in a figure switches point markers to the Runebender language
  (green smooth, orange corner, purple off-curve); `color=False` is the mono
  look. The palette lives once in `lib.py`.
- The existing OG-card generator still lives at `scripts/og/img2bez/card.py`;
  folding it in here (so all of this post's generators share `lib.py`) is a
  reasonable next step.
