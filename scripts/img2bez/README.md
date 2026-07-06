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
  lib.py             shared helpers: Runebender point language, outline
                     parsing, framing, animation assembly
  vtracer.py         vtracer-side machinery: run the CLI, parse its SVG,
                     reconcile a Regular/Bold pair, rebuild a cubic point
                     structure so both panels share one point language
  fig_g_compare.py   figure: vtracer vs img2bez side by side, morphing
                     Regular <-> Bold                        (section 03)
  fig_g_interp.py    figure: the img2bez G morphing alone    (unused today)
  inputs/            normalized source rasters (neo-grotesk G Regular/Bold,
                     autocropped to identical 1200px framing)
  g-masters.json     img2bez joint masters trace (geometry source of truth)
  build/             frame + SVG scratch (gitignored)
  requirements.txt   this post's Python deps
```

**Outputs land where the post consumes them**, not here: still images →
`src/content/blog/img2bez/*.png` (Astro-optimized); video →
`public/demos/img2bez/*.mp4`.

## Running

Uses the repo-root `.venv` (the eliheuer/drawbot-skia fork + Pillow), plus
`ffmpeg` on PATH; `fig_g_compare.py` also needs the `vtracer` CLI
(`cargo install vtracer`). From the repo root:

```sh
.venv/bin/python scripts/img2bez/fig_g_compare.py  # -> public/demos/img2bez/g-compare.mp4
.venv/bin/python scripts/img2bez/fig_g_interp.py   # -> public/demos/img2bez/g-interp.mp4
```

## Regenerating the trace snapshot

`g-masters.json` is a cached `img2bez masters` trace (the joint masters
pipeline), so tweaking a figure's *style* or *motion* needs no re-trace. To
re-trace from the raw rasters (needs the `img2bez` CLI and a designspace with
Regular+Bold masters for metrics):

```sh
img2bez masters <designspace> \
  --glyph G --unicode 0047 \
  --image Regular=scripts/img2bez/inputs/G-regular.png \
  --image Bold=scripts/img2bez/inputs/G-bold.png \
  --fit descender:cap --profile clean --format json > scripts/img2bez/g-masters.json
```

**Input hygiene matters:** the two rasters must share identical framing (same
canvas size, same ink placement). The committed inputs are autocropped to the
ink and centered on a 1200px square with an equal margin — mismatched framing
makes the tracer see the masters at different scales and degrades the
correspondence.

TODO (self-containment): the `masters` subcommand needs a designspace for
metrics. Right now that's borrowed from an external font repo. Ship a tiny stub
designspace + two metric-only master UFOs under `inputs/` so re-tracing depends
only on this repo.

## Conventions for new figures

- Add `fig_<name>.py`, import `lib`, keep raw inputs in `inputs/`.
- The point language (smooth = green circle, corner = orange square,
  off-curve = purple circle, dark fill + colored ring) mirrors Runebender-web
  and lives once in `lib.py`. Render every tool's output through
  `lib.draw_points` so comparisons differ only in where the points sit.
- The existing OG-card generator still lives at `scripts/og/img2bez/card.py`;
  folding it in here (so all of this post's generators share `lib.py`) is a
  reasonable next step.
