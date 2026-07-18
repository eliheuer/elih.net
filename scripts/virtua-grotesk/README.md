# Virtua Grotesk figures

DesignBot sources for the “Virtua Grotesk: Powers-of-Two Design Grids for
Neural Networks” post. The renderer reads the font sources from the sibling
checkout at `~/GH/repos/virtua-grotesk`. Renders are previews by default and
stay under the ignored `build/preview/` directory. Publishing to the blogpost
directory is always explicit.

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
5. `build/preview/` — disposable renders and comparison images. This directory
   is ignored by Git.
6. `src/content/blog/virtua-grotesk/*.png` — approved published outputs. Do not
   edit these by hand; edit the generator and publish one chosen render.

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
cargo run --release --bin interpn                # ignored preview
cargo run --release --bin interpn -- --publish   # replace published PNG
```

Use the default preview command while editing. Inspect the file printed by the
renderer under `build/preview/blog/`, and use `--publish` only after approving
it. This prevents exploratory PNG revisions from entering Git history.

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
