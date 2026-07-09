# virtua-grotesk figures

Figure sources for the "Virtua Grotesk: Powers-of-Two Design Grids for
Neural Networks" post, drawn with [designbot](https://github.com/eliheuer/designbot).
Assumes a sibling checkout of the font sources at `~/GH/repos/virtua-grotesk`.

```sh
cargo run --release --bin og     # share-card.png + public/og/virtua-grotesk.png
cargo run --release --bin card   # older hero: R a 2 outlines with point markers
```

## Figures

- `og` — hero/OG: "Grid" as a Replica-style dimension sheet in dark mode with
  the runebender-web palette — red glyphs on the 16-unit design grid, blue
  vertical-metric lines with values, advance widths and hatched side bearings
  in staggered dimension rows. Labels in Geist Mono
  (`~/GH/repos/google-fonts/ofl/geistmono`).
- `card` — previous hero: R a 2 outlines over the powers-of-two grid,
  Runebender point palette (smooth = green, corner = orange, off-curve =
  purple), baseline in orange.

## TODO

- `fig-complete-two` and `fig-bolden-b` in the post are placeholder renders
  from the font-garden-lab eval harness — redraw with designbot in the
  post's dark visual language (grid + metrics + labeled panels).
- planned: `fig-grid-ladder` (the 2/4/8/…/256 measurement ladder against the
  vertical metrics), `fig-tokens` (glyph outline with numbered points next
  to its token stream), `fig-interp` (Regular→Bold interpolation sweep as
  the synthetic-data story).
