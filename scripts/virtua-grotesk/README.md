# Virtua Grotesk figures

DesignBot sources for the “Virtua Grotesk: Powers-of-Two Design Grids for
Neural Networks” post. The renderer reads the font sources from the sibling
checkout at `~/GH/repos/virtua-grotesk`. Renders are previews by default and
replace the working-tree PNG used by the blogpost, so Astro shows every change
in context immediately. Git does not store a rendered revision unless it is
explicitly committed.

## Where to edit

The system has seven deliberately separate layers:

1. `src/inputs.rs` — pinned external fonts, UFOs, model runs, SVGs, and logs.
   Change an input here deliberately; generators never select the newest run.
2. `src/style.rs` — the visual editing surface.
   - `color`: context-free swatches only.
   - `line`: the primitive line-width scale.
   - `type_size`: the primitive type-size scale.
   - `role`: mappings from those primitives to jobs such as canvas background,
     grid levels, dimension text, Bézier handles, and curves.
3. `src/technical.rs` — the approved technical-drawing language established
   by `fig-system-no.png` and `fig-system-ho.png`. It owns the real source
   grid, metric furniture, glyph pen, point language, measurement caps, and
   measurement typography.
4. `src/lib.rs` — shared rendering mechanics. UFO loading, drawing helpers,
   dimensions, markers, labels, and collision-aware placement live here.
   The abstract figures share two primitives: `ValueBox` (the stroked,
   filled rectangle with a centered mono numeral used by ladder rungs, bit
   cells, and exactness cells; box strokes are `line::BOX`) and `Marker`
   (a single pen-stroked position dot). A figure states each primitive's
   proportions once, locally, and calls `draw`; the primitives own only
   the drawing, never the layout.
5. `src/bin/*.rs` — one binary per figure group. These files contain content,
   geometry, and layout decisions; they should not contain raw RGB values.
6. `src/content/blog/virtua-grotesk/*.png` — the live site-preview outputs.
   These may stay modified throughout a design pass without being committed.
7. `build/preview/` — optional isolated scratch renders. This directory is
   ignored by Git.

To change a hue everywhere, edit its base swatch in `style::color`. To change
which swatch a drawing job uses, edit the corresponding function in
`style::role`. For example, the Bézier handle mapping and the primary curve
mapping are separate from the neutral swatches they currently use.

The broad shared line and type roles retain their historical names (`PEN`,
`PEN_LIGHT`, `FRAME_TEXT`, and so on) as compatibility mappings. Their actual
values come from the primitive scales in `style.rs`.

## Technical drawing preset

Use `TechnicalStyle::section_three()` when a figure should look like the `no`
and `HO` drawings. Do not copy their constants or rebuild their primitives in
a figure binary. The binary should specify only the outlines, source-space
placement, fill colors, metrics, and deliberately placed measurements.

```rust
let technical = TechnicalStyle::section_three();
let frame = technical.frame(run, bottom, top);

technical.background_grid(&mut sheet, &frame, &glyphs, bottom, top);
technical.metric_system(
    &mut sheet,
    &frame,
    run,
    &sort_bounds,
    &[0.0, 576.0],
    &[-16.0, 592.0],
    top,
    bottom,
);
technical.glyph(&mut sheet, &outline, &frame, origin, fill);
technical.measurement(
    &mut sheet,
    &frame,
    origin,
    TechnicalMeasurement::points(0, (32.0, 288.0), (132.0, 288.0), 100),
);
```

Named modifiers document genuine semantic differences. For example,
`with_grid_level_points()` uses green and red to distinguish points on and off
the 8-unit grid while preserving every other section 03 convention. A
continuous multi-glyph composition can use `continuous_background_grid()` and
`metric_rules()` without creating a second visual system.

When art direction changes a shared primitive, edit `TechnicalStyle` and
rebuild its reference figures first. When art direction changes only one
composition, keep the source coordinates and layout adjustment in that
figure's binary.

## Color and export guardrails

These rules belong to this blog-figure crate for now. They are deliberately a
thin policy layer around DesignBot, not a change to the upstream DesignBot
renderer.

- Define coordinated saturated palettes in OKLCH through `oklch_srgb` in
  `src/style.rs`, not by giving several hues the same HSL saturation or pushing
  individual RGB channels to `255`. OKLCH chroma is a much better starting
  point for even visual intensity across hues.
- Use a shared chroma target for a palette, then review the result on its real
  background at full size and thumbnail size. Equal OKLCH chroma improves the
  baseline but cannot cancel the effects of area, surround, display, or human
  adaptation. Record any resulting hue-specific optical correction as a named
  delta beside the shared target; do not hide it in a replacement RGB value.
- Let `oklch_srgb` reduce chroma at constant lightness and hue when a requested
  color is outside sRGB. Do not clamp RGB channels independently; clipping one
  channel can make one hue look substantially louder than the rest.
- Write every generated PNG through `write_png` (normally via `Sheet::save`).
  It marks the pixels as sRGB and supplies the standard gAMA/cHRM fallbacks so
  browsers and social-media ingestion pipelines do not have to guess the
  source color space.
- Keep the OG palette separate from the generic illustration palette. A color
  correction for a large share card should not silently alter every figure in
  the article.

When these Rust sources seed social-media renderers in the upstream
`virtua-grotesk` repository, copy the color-management layer with them:

1. `oklch_srgb` and the palette comments from `src/style.rs`;
2. `write_png` and `tag_png_as_srgb` from `src/lib.rs`;
3. the `crc32fast` dependency from `Cargo.toml`;
4. the semantic palette mappings, rather than only the final RGB values.

That handoff keeps the upstream compositions visually related to the blog
figures while allowing their formats and art direction to evolve separately.

## Graphic composition guardrails

The inline figures and the OG image form one family. Keep these rules visible
in the source so a person can art-direct a difficult detail without reverse
engineering a layout engine:

- Render at `2520 × 1320` with a nominal `64 px` outer margin. Important forms
  should usually fill the frame; judge every render again at roughly one-third
  size, which is close to its width in the article and on many social cards.
- Give each image one dominant visual argument. Remove titles, legends,
  attribution, panel chrome, and explanatory prose when the article already
  supplies that context.
- Use the mid-gray field, one near-black drawing pen, opaque palette fills, and
  the shared point fills from `role::figure`. Avoid accidental extra colors in
  one-off helpers.
- Keep factual geometry factual. Glyph outlines, points, handles, metrics, and
  measurements come from the current Virtua Grotesk designspace; simplify the
  composition around them, not the source data itself.
- Use equal-size on-curve and off-curve markers. Distinguish the correction
  tier by fill value rather than by making those points smaller.
- Prefer explicit, local layout constants in each `src/bin/*.rs` composition.
  These figures are meant to be adjusted by hand; abstraction is useful only
  when it makes the visual relationship easier to understand and edit.
- Keep captions and alt text in the MDX synchronized when a composition's
  visible content changes. During parallel copy editing, hand that text update
  to the editor instead of modifying the MDX from the image-design pass.

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
| `proofs` | `fig-midpoint.png`, `fig-ladder.png`, `fig-bits.png` |
| `model` | `fig-model-bolden-n.png` |
| `system` | system sheets and `fig-semantic-grid.png` |
| `interp` | `fig-interp.png` |
| `interpn` | `fig-interp-outlines.png` |
| `curves` | `fig-losscurve.png` |

`card` is the older hero renderer. It can only write
`build/legacy-card/share-card.png`; `og` exclusively owns the current
share-card outputs.
