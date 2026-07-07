# virtua-grotesk figures

Figure sources for the "Virtua Grotesk: Powers-of-Two Design Grids for
Neural Networks" post, drawn with [designbot](https://github.com/eliheuer/designbot).
Assumes a sibling checkout of the font sources at `~/GH/repos/virtua-grotesk`.

```sh
cargo run --release --bin card   # share-card.png + public/og/virtua-grotesk.png
```

## Figures

- `card` — hero/OG: R a 2 outlines over the powers-of-two grid, Runebender
  point palette (smooth = green, corner = orange, off-curve = purple),
  baseline in orange.

## TODO

- `fig-complete-two` and `fig-bolden-b` in the post are placeholder renders
  from the font-garden-lab eval harness — redraw with designbot in the
  post's dark visual language (grid + metrics + labeled panels).
- planned: `fig-grid-ladder` (the 2/4/8/…/256 measurement ladder against the
  vertical metrics), `fig-tokens` (glyph outline with numbered points next
  to its token stream), `fig-interp` (Regular→Bold interpolation sweep as
  the synthetic-data story).
