# virtua-grotesk figures

Figure sources for the "Virtua Grotesk: Powers-of-Two Design Grids for
Neural Networks" post, drawn with [designbot](https://github.com/eliheuer/designbot).
Assumes a sibling checkout of the font sources at `~/GH/repos/virtua-grotesk`.

```sh
cargo run --release --bin og     # share-card.png + public/og/virtua-grotesk.png
cargo run --release --bin figs   # fig-complete-two.png + fig-bolden-b.png
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
- `figs` — the two post comparison figures (`fig-complete-two`,
  `fig-bolden-b`) in the OG dimension-sheet language. Parses the
  font-garden-lab eval SVGs (`~/GH/repos/font-garden-lab/runs/night1/`),
  flips them to y-up, and re-renders each panel color-coded by role:
  gray = ground truth, green = the given input, red = the model output.
  Re-run after a fresh eval to refresh the model panels.

## TODO

- planned: `fig-grid-ladder` (the 2/4/8/…/256 measurement ladder against the
  vertical metrics), `fig-tokens` (glyph outline with numbered points next
  to its token stream), `fig-interp` (Regular→Bold interpolation sweep as
  the synthetic-data story).
