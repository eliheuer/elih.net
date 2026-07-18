//! OG / share card for the Virtua Grotesk post: the word "Grid" at 1:1
//! (one font unit = one canvas pixel at scale 1), on the family frame with
//! the full point language — green points on the 8-unit machine grid, red
//! points off 8 on the 2-unit human grid. Hero fill (fill_strong), the
//! design grid, vertical metrics, and the advance/sidebearing dimension
//! zone below the baseline.
//!
//! REBUILD after editing (from this directory):
//!     cargo run --release --bin og
//!
//! Writes BOTH outputs:
//!     ../../src/content/blog/virtua-grotesk/share-card.png   (post hero)
//!     ../../public/og/virtua-grotesk.png                     (og:image)

use designbot::prelude::{Color, TextAlign};
use designbot_render::Renderer;
use std::path::{Path, PathBuf};
use virtua_grotesk_figures::*;

const GLYPHS: &[&str] = &["G", "r", "i", "d"];
const TITLE: &str = "Virtua Grotesk: Grid Systems as Datasets";

// EDIT HERE: composition controls. Measurements remain close to the baseline
// so the outlines can use more of the canvas without losing drawing detail.
const SCALE: f64 = 1.18;
const SHOW_BACKGROUND_GRID: bool = false;
const STRUCTURE_GRID_UNIT: f64 = 8.0;
const MAJOR_GRID_UNIT: f64 = 32.0;
const TOP_OVERSHOOT: f64 = 784.0;
const MEASUREMENT_ROW_BOTTOM: f64 = 180.0;
const MEASUREMENT_ROW_GAP: f64 = 36.0;

fn ink(o: &Outline) -> (f64, f64) {
    let mut lo = f64::INFINITY;
    let mut hi = f64::NEG_INFINITY;
    for (x, _, _) in &o.points {
        lo = lo.min(*x);
        hi = hi.max(*x);
    }
    (lo, hi)
}

fn mix_color(start: Color, end: Color, t: f64) -> Color {
    let channel = |a: u8, b: u8| (a as f64 + (b as f64 - a as f64) * t).round() as u8;
    Color::rgb(
        channel(start.r, end.r),
        channel(start.g, end.g),
        channel(start.b, end.b),
    )
}

fn default_source_from_designspace(designspace_path: &Path) -> PathBuf {
    let document = norad::designspace::DesignSpaceDocument::load(designspace_path)
        .unwrap_or_else(|error| panic!("load {}: {error}", designspace_path.display()));
    let source = document
        .sources
        .iter()
        .find(|source| {
            source.location.iter().all(|dimension| {
                let Some(axis) = document
                    .axes
                    .iter()
                    .find(|axis| axis.name == dimension.name)
                else {
                    return false;
                };
                let value = dimension.xvalue.or(dimension.uservalue);
                value.is_some_and(|value| (value - axis.default).abs() < 0.001)
            })
        })
        .expect("designspace must contain a source at the default location");
    designspace_path
        .parent()
        .expect("designspace must have a parent directory")
        .join(&source.filename)
}

/// The real source lattice: every 8 font units is visible, with every fourth
/// line emphasized to retain the more legible 32-unit rhythm.
fn draw_edge_grid(sheet: &mut Sheet, f: &Frame) {
    for (unit, color, width) in [
        (STRUCTURE_GRID_UNIT, role::og::grid_minor(), line::HAIRLINE),
        (MAJOR_GRID_UNIT, role::og::grid_major(), line::FINE),
    ] {
        let step = unit * f.s;
        sheet.ctx.no_fill().stroke(color).stroke_width(width);

        let mut x = f.x(0.0).rem_euclid(step);
        while x < W {
            sheet.ctx.line(x, 0.0, x, H);
            x += step;
        }

        let mut y = f.y(0.0).rem_euclid(step);
        while y < H {
            sheet.ctx.line(0.0, y, W, y);
            y += step;
        }
    }
}

fn assert_structure_grid_alignment(outlines: &[Outline]) {
    let mut origin = 0.0;
    for outline in outlines {
        for (x, y, _) in &outline.points {
            if on8(*x, *y) {
                let grid_x = (origin + x) / STRUCTURE_GRID_UNIT;
                let grid_y = y / STRUCTURE_GRID_UNIT;
                assert!((grid_x - grid_x.round()).abs() < 1e-9);
                assert!((grid_y - grid_y.round()).abs() < 1e-9);
            }
        }
        origin += outline.width;
    }
}

/// Draw vertical metrics only inside the outer advance boundaries, so no
/// horizontal strokes protrude beyond the blue specimen box.
fn metric_lines_inside_run(sheet: &mut Sheet, f: &Frame, run: f64, solid: &[f64], dashed: &[f64]) {
    let x0 = f.x(0.0);
    let x1 = f.x(run);
    let ctx = &mut sheet.ctx;
    ctx.no_fill()
        .stroke(role::og::construction())
        .stroke_width(PEN);
    ctx.line_dash(&[10.0, 10.0]);
    for &uy in dashed {
        ctx.line(x0, f.y(uy), x1, f.y(uy));
    }
    ctx.line_dash(&[]);
    for &uy in solid {
        ctx.line(x0, f.y(uy), x1, f.y(uy));
    }
}

fn main() {
    let mono_path = inputs::geist_mono();
    let virtua_path = inputs::virtua_regular_font();
    let designspace_path = inputs::virtua_designspace();
    let source_ufo = default_source_from_designspace(&designspace_path);
    let glyphs_dir = source_ufo.join("glyphs");
    assert!(
        glyphs_dir.is_dir(),
        "Virtua Grotesk source directory does not exist: {}",
        glyphs_dir.display()
    );
    println!(
        "using {} default source outlines from {}",
        designspace_path.display(),
        source_ufo.display()
    );
    assert!(
        virtua_path.is_file(),
        "Virtua Grotesk Regular font does not exist: {}; build the font first",
        virtua_path.display()
    );

    let mut renderer = Renderer::new(W as u32, H as u32);
    let mono = load_family(&mut renderer, mono_path.to_str().unwrap());
    let virtua = load_family(&mut renderer, virtua_path.to_str().unwrap());
    let mut sheet = new_sheet(&renderer, &mono);
    sheet.ctx.background(role::og::background());

    let outlines: Vec<Outline> = GLYPHS
        .iter()
        .map(|g| load_outline(&glyphs_dir, g))
        .collect();
    let run: f64 = outlines.iter().map(|o| o.width).sum();

    let side_margin = (W - run * SCALE) / 2.0;
    let f = Frame {
        s: SCALE,
        x0: side_margin,
        baseline: H - side_margin - TOP_OVERSHOOT * SCALE,
    };
    assert_structure_grid_alignment(&outlines);

    if SHOW_BACKGROUND_GRID {
        draw_edge_grid(&mut sheet, &f);
    }
    metric_lines_inside_run(&mut sheet, &f, run, &[0.0, 576.0, 768.0], &[784.0, -16.0]);

    // advance-boundary cell dividers: each sort in its own cell, the
    // divider dropping from the top overshoot down to the deeper of the
    // two adjacent dimension rows, with knockout nodes at the ends
    let row_y = |j: i64| {
        if j % 2 == 0 {
            MEASUREMENT_ROW_BOTTOM
        } else {
            MEASUREMENT_ROW_BOTTOM + MEASUREMENT_ROW_GAP
        }
    };
    {
        let mut bounds = vec![0.0];
        let mut acc = 0.0;
        for o in &outlines {
            acc += o.width;
            bounds.push(acc);
        }
        let n = outlines.len() as i64;
        for (i, &b) in bounds.iter().enumerate() {
            let i = i as i64;
            let deepest = row_y(i.min(n - 1)).min(row_y((i - 1).clamp(0, n - 1)));
            cell_dividers_colored(
                &mut sheet,
                &[f.x(b)],
                f.y(784.0),
                deepest,
                role::og::construction(),
                role::og::background(),
            );
        }
    }

    // glyphs: hero fill + grid-level point language
    let first_center = outlines.first().expect("at least one glyph").width / 2.0;
    let last = outlines.last().expect("at least one glyph");
    let last_center = run - last.width / 2.0;
    let mut ox = 0.0;
    for o in &outlines {
        let center = ox + o.width / 2.0;
        let t = ((center - first_center) / (last_center - first_center)).clamp(0.0, 1.0);
        let body_color = mix_color(role::og::gradient_start(), role::og::gradient_end(), t);
        draw_body_strong(&mut sheet, o, SCALE, f.x(ox), f.baseline, body_color);
        draw_points_colored_on(
            &mut sheet,
            o,
            SCALE,
            f.x(ox),
            f.baseline,
            body_color,
            role::og::structure_point(),
            role::og::correction_point(),
            role::og::background(),
        );
        ox += o.width;
    }

    // advance / sidebearing dimension zone, staggered rows like a real sheet
    let mut ox = 0.0;
    for (j, o) in outlines.iter().enumerate() {
        let (i0, i1) = ink(o);
        advance_row_colored(
            &mut sheet,
            &f,
            row_y(j as i64),
            &[(ox, o.width)],
            &[(i0, i1)],
            role::og::dimension_line(),
            role::og::correction_point(),
            role::og::structure_point(),
            role::og::correction_point(),
            role::og::background(),
        );
        ox += o.width;
    }

    // Fit the post title to the exact width of the outer blue boundaries.
    // Its baseline uses the same inset as the top and side margins.
    let title_width = f.x(run) - f.x(0.0);
    let title_unit_width = renderer.text_width(TITLE, Some(&virtua), 1.0, &[]);
    let title_size = title_width / title_unit_width;
    sheet
        .ctx
        .font(&virtua)
        .clear_font_variations()
        .font_size(title_size)
        .fill(role::og::title())
        .text_align(TextAlign::Left)
        .text(TITLE, f.x(0.0), side_margin);

    let outputs = OutputPaths::from_args();
    sheet.save(&renderer, &outputs.blog("share-card.png"));
    sheet.save(&renderer, &outputs.og("virtua-grotesk.png"));
}
