# Virtua Grotesk figures

DesignBot sources for the “Virtua Grotesk: Powers-of-Two Design Grids for
Neural Networks” post. The renderer reads the font sources from the sibling
checkout at `~/GH/repos/virtua-grotesk`. Renders are previews by default and
replace the working-tree PNG used by the blogpost, so Astro shows every change
in context immediately. Git does not store a rendered revision unless it is
explicitly committed.

## Where to edit

The system has six deliberately separate layers:

1. `src/inputs.rs` — pinned external fonts, UFOs, model runs, SVGs, and logs.
   Change an input here deliberately; generators never select the newest run.
2. `src/style.rs` — the visual editing surface.
   - `color`: context-free swatches only.
   - `line`: the primitive line-width scale.
   - `type_size`: the primitive type-size scale.
   - `role`: mappings from those primitives to jobs such as canvas background,
     grid levels, dimension text, Bézier handles, and curves.
3. `src/lib.rs` — shared rendering mechanics. UFO loading, drawing helpers,
   dimensions, markers, labels, and collision-aware placement live here.
4. `src/bin/*.rs` — one binary per figure group. These files contain content,
   geometry, and layout decisions; they should not contain raw RGB values.
5. `src/content/blog/virtua-grotesk/*.png` — the live site-preview outputs.
   These may stay modified throughout a design pass without being committed.
6. `build/preview/` — optional isolated scratch renders. This directory is
   ignored by Git.

To change a hue everywhere, edit its base swatch in `style::color`. To change
which swatch a drawing job uses, edit the corresponding function in
`style::role`. For example, the Bézier handle mapping and the primary curve
mapping are separate from the neutral swatches they currently use.

The broad shared line and type roles retain their historical names (`PEN`,
`PEN_LIGHT`, `FRAME_TEXT`, and so on) as compatibility mappings. Their actual
values come from the primitive scales in `style.rs`.

## Verify and rebuild

From `scripts/virtua-grotesk`:

```sh
cargo check --bins
cargo run --release --bin interpn               # update the PNG in the post
cargo run --release --bin interpn -- --scratch  # ignored standalone render
```

Run the site alongside the generator:

```sh
pnpm dev
# open http://localhost:4321/blog/virtua-grotesk/
```

The default generator command updates the image already referenced by the
post, and Astro refreshes it in place. Leave these PNGs uncommitted while
iterating. Commit the generator freely, then commit each approved PNG once at
the end of the design pass. Use `--scratch` only when you want a render that
does not appear in the post.

If intermediate checkpoints are valuable, keep them as local commits and
squash them before pushing. This gives us recovery without making history
rewrites part of the normal blog workflow.

Before committing a large batch of published assets, audit repository growth
from the repository root:

```sh
pnpm repo:size
```

The audit reports current file sizes and the paths accumulating the most data
across `main`. It fails if the compressed `main` bundle exceeds 100 MiB; set
`REPO_BUNDLE_BUDGET_MB` to test a different budget.

Active figure groups:

| Binary | Outputs |
| --- | --- |
| `og` | `share-card.png`, `public/og/virtua-grotesk.png` |
| `figs` | `fig-complete-r.png`, `fig-bolden-g.png` |
| `optical` | `fig-optical-correction.png` |
| `grids` | `fig-grid-labels.png` |
| `proofs` | `fig-fractions.png`, `fig-midpoint.png`, `fig-ladder.png`, `fig-bits.png` |
| `model` | `fig-model-review.png`, `fig-model-bolden-n.png` |
| `system` | system sheets and `fig-semantic-grid.png` |
| `interp` | `fig-interp.png` |
| `interpn` | `fig-interp-outlines.png` |
| `curves` | `fig-losscurve.png` |

`card` is the older hero renderer. It can only write
`build/legacy-card/share-card.png`; `og` exclusively owns the current
share-card outputs.
