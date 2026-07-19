//! OG / share card for the Virtua Grotesk post: the word "Grid" at 1:1
//! (one font unit = one canvas pixel at scale 1), on the family frame with
//! the full point language — forest-teal points on the 8-unit machine grid,
//! red points off 8 on the 2-unit human grid. Hero fill (fill_strong), the
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

#[derive(Clone, Copy)]
struct GlyphMeasurement {
    glyph: usize,
    x0: f64,
    y0: f64,
    x1: f64,
    y1: f64,
    value: i64,
    offset: f64,
    end_inset: f64,
    label_dx: f64,
    label_dy: f64,
    sum_dx: f64,
    sum_dy: f64,
}

const POINT_MEASUREMENT_END_INSET: f64 = 30.0;
const EDGE_MEASUREMENT_END_INSET: f64 = 20.0;
const HORIZONTAL_VALUE_BASELINE_OFFSET: f64 = 20.0;
const HORIZONTAL_SUM_BASELINE_OFFSET: f64 = -48.0;
const VERTICAL_LABEL_GAP: f64 = 16.0;
const VERTICAL_LABEL_BASELINE_OFFSET: f64 = -12.0;

// EDIT HERE: individually placed glyph measurements in source-font units.
// Each entry can move independently without changing any glyph geometry.
const GLYPH_MEASUREMENTS: [GlyphMeasurement; 7] = [
    GlyphMeasurement {
        glyph: 0,
        x0: 48.0,
        y0: 384.0,
        x1: 156.0,
        y1: 384.0,
        value: 108,
        offset: 0.0,
        end_inset: POINT_MEASUREMENT_END_INSET,
        label_dx: 0.0,
        label_dy: 0.0,
        sum_dx: 0.0,
        sum_dy: 0.0,
    },
    GlyphMeasurement {
        glyph: 0,
        x0: 560.0,
        y0: 320.0,
        x1: 560.0,
        y1: 416.0,
        value: 96,
        offset: 0.0,
        end_inset: EDGE_MEASUREMENT_END_INSET,
        label_dx: 0.0,
        label_dy: 0.0,
        sum_dx: 0.0,
        sum_dy: 0.0,
    },
    GlyphMeasurement {
        glyph: 0,
        x0: 664.0,
        y0: 80.0,
        x1: 760.0,
        y1: 80.0,
        value: 96,
        offset: 0.0,
        end_inset: EDGE_MEASUREMENT_END_INSET,
        label_dx: 0.0,
        label_dy: 0.0,
        sum_dx: 0.0,
        sum_dy: 0.0,
    },
    GlyphMeasurement {
        glyph: 1,
        x0: 64.0,
        y0: 224.0,
        x1: 160.0,
        y1: 224.0,
        value: 96,
        offset: 0.0,
        end_inset: EDGE_MEASUREMENT_END_INSET,
        label_dx: 0.0,
        label_dy: 0.0,
        sum_dx: 0.0,
        sum_dy: 0.0,
    },
    GlyphMeasurement {
        glyph: 2,
        x0: 64.0,
        y0: 288.0,
        x1: 160.0,
        y1: 288.0,
        value: 96,
        offset: 0.0,
        end_inset: EDGE_MEASUREMENT_END_INSET,
        label_dx: 0.0,
        label_dy: 0.0,
        sum_dx: 0.0,
        sum_dy: 0.0,
    },
    GlyphMeasurement {
        glyph: 3,
        x0: 32.0,
        y0: 288.0,
        x1: 132.0,
        y1: 288.0,
        value: 100,
        offset: 0.0,
        end_inset: POINT_MEASUREMENT_END_INSET,
        label_dx: 0.0,
        label_dy: 0.0,
        sum_dx: 0.0,
        sum_dy: 0.0,
    },
    GlyphMeasurement {
        glyph: 3,
        x0: 480.0,
        y0: 680.0,
        x1: 576.0,
        y1: 680.0,
        value: 96,
        offset: 0.0,
        end_inset: EDGE_MEASUREMENT_END_INSET,
        label_dx: 0.0,
        label_dy: 0.0,
        sum_dx: 0.0,
        sum_dy: 0.0,
    },
];

// EDIT HERE: composition controls. Measurements remain close to the baseline
// so the outlines can use more of the canvas without losing drawing detail.
const SCALE: f64 = 1.18;
// Temporarily enable this while art directing positions in exact grid units.
// Set it back to false for the final image.
const SHOW_BACKGROUND_GRID: bool = false;
// Social platforms add their own title treatment over the lower part of the
// card. Keep this false for the share image; it remains available for variants.
const SHOW_TITLE: bool = false;
const STROKE_WIDTH: f64 = line::HERO;
const POINT_SIZE: f64 = 20.0;
const GLYPH_FILL_ALPHA: u8 = 255;
const STRUCTURE_GRID_UNIT: f64 = 8.0;
const BACKGROUND_GRID_UNIT: f64 = 64.0;
const TOP_OVERSHOOT: f64 = 784.0;
const MEASUREMENT_ROW_OFFSETS: [f64; 4] = [0.0, 0.0, 0.0, 0.0];
const BOTTOM_MEASUREMENT_RAISE: f64 = 48.0;
const MEASUREMENT_LABEL_OFFSET: f64 = 30.0;
const MEASUREMENT_EDGE_LABEL_INSET: f64 = 18.0;
const MEASUREMENT_TEXT_SIZE: f64 = 32.0;
const MEASUREMENT_TEXT_WEIGHT: f32 = 600.0;
const GLYPH_MEASUREMENT_CAP_HALF_LENGTH: f64 = 12.0;
const GLYPH_MEASUREMENT_LINE_GAP: f64 = 28.0;
const TITLE_BASELINE_OFFSET: f64 = 16.0;

fn ink(o: &Outline) -> (f64, f64) {
    let mut lo = f64::INFINITY;
    let mut hi = f64::NEG_INFINITY;
    for (x, _, _) in &o.points {
        lo = lo.min(*x);
        hi = hi.max(*x);
    }
    (lo, hi)
}

/// Compact capped dimensions remain legible when a sidebearing is too narrow
/// to contain opposing arrowheads. Placement uses the constants above.
fn measurement_span(
    sheet: &mut Sheet,
    x0: f64,
    x1: f64,
    y: f64,
    text: &str,
    color: Color,
    label_x: f64,
    label_align: i8,
) {
    let tick = 12.0;
    sheet.ctx.no_fill().stroke(color).stroke_width(STROKE_WIDTH);
    sheet.ctx.line(x0, y, x1, y);
    sheet.ctx.line(x0, y - tick, x0, y + tick);
    sheet.ctx.line(x1, y - tick, x1, y + tick);
    sheet.label_weighted(
        text,
        label_x,
        y + MEASUREMENT_LABEL_OFFSET,
        MEASUREMENT_TEXT_SIZE,
        color,
        label_align,
        MEASUREMENT_TEXT_WEIGHT,
    );
}

fn glyph_measurement(sheet: &mut Sheet, f: &Frame, origin: f64, m: GlyphMeasurement) {
    let p0 = (f.x(origin + m.x0), f.y(m.y0));
    let p1 = (f.x(origin + m.x1), f.y(m.y1));
    let dx = p1.0 - p0.0;
    let dy = p1.1 - p0.1;
    let length = (dx * dx + dy * dy).sqrt();
    let direction = (dx / length, dy / length);
    let normal = (-dy / length, dx / length);
    let q0 = (
        p0.0 + normal.0 * m.offset + direction.0 * m.end_inset,
        p0.1 + normal.1 * m.offset + direction.1 * m.end_inset,
    );
    let q1 = (
        p1.0 + normal.0 * m.offset - direction.0 * m.end_inset,
        p1.1 + normal.1 * m.offset - direction.1 * m.end_inset,
    );
    let cap = GLYPH_MEASUREMENT_CAP_HALF_LENGTH;
    let color = role::og::construction();
    sheet.ctx.no_fill().stroke(color).stroke_width(STROKE_WIDTH);
    sheet.ctx.line(q0.0, q0.1, q1.0, q1.1);
    for q in [q0, q1] {
        sheet.ctx.line(
            q.0 - normal.0 * cap,
            q.1 - normal.1 * cap,
            q.0 + normal.0 * cap,
            q.1 + normal.1 * cap,
        );
    }
    let midpoint = ((q0.0 + q1.0) / 2.0, (q0.1 + q1.1) / 2.0);
    let decomposition = p2sum(m.value);
    let parts: Vec<&str> = decomposition.split('+').collect();
    let sum_lines = if parts.len() > 2 {
        vec![parts[..2].join("+"), format!("+{}", parts[2..].join("+"))]
    } else {
        vec![decomposition]
    };

    let horizontal = dx.abs() >= dy.abs();
    let (value_x, value_y, value_align, sum_x, sum_y, sum_align) = if horizontal {
        (
            midpoint.0 + m.label_dx,
            midpoint.1 + HORIZONTAL_VALUE_BASELINE_OFFSET + m.label_dy,
            0,
            midpoint.0 + m.sum_dx,
            midpoint.1 + HORIZONTAL_SUM_BASELINE_OFFSET + m.sum_dy,
            0,
        )
    } else {
        (
            midpoint.0 - VERTICAL_LABEL_GAP + m.label_dx,
            midpoint.1 + VERTICAL_LABEL_BASELINE_OFFSET + m.label_dy,
            1,
            midpoint.0 + VERTICAL_LABEL_GAP + m.sum_dx,
            midpoint.1 + VERTICAL_LABEL_BASELINE_OFFSET + m.sum_dy,
            -1,
        )
    };
    sheet.label_weighted(
        &m.value.to_string(),
        value_x,
        value_y,
        MEASUREMENT_TEXT_SIZE,
        color,
        value_align,
        MEASUREMENT_TEXT_WEIGHT,
    );
    for (index, line) in sum_lines.iter().enumerate() {
        sheet.label_weighted(
            line,
            sum_x,
            sum_y - index as f64 * GLYPH_MEASUREMENT_LINE_GAP,
            MEASUREMENT_TEXT_SIZE,
            color,
            sum_align,
            MEASUREMENT_TEXT_WEIGHT,
        );
    }
}

fn construction_node(sheet: &mut Sheet, x: f64, y: f64) {
    marker_with_fill_sized(
        sheet,
        x,
        y,
        PtRole::Smooth,
        role::og::construction(),
        role::og::structure_point_fill(),
        POINT_SIZE,
        STROKE_WIDTH,
    );
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

/// A sparse background lattice linked to the source coordinate system.
fn draw_edge_grid(sheet: &mut Sheet, f: &Frame) {
    let step = BACKGROUND_GRID_UNIT * f.s;
    sheet
        .ctx
        .no_fill()
        .stroke(role::og::grid_minor())
        .stroke_width(STROKE_WIDTH);

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
        .stroke_width(STROKE_WIDTH);
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
    sheet.ctx.line_cap("round");

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
    // Optically align the lower specimen boundary to the same margin as the
    // other three sides, even though it sits slightly below the true descender.
    let visual_descender = (side_margin - f.baseline) / f.s;
    metric_lines_inside_run(
        &mut sheet,
        &f,
        run,
        &[0.0, 576.0, 768.0, visual_descender],
        &[TOP_OVERSHOOT, -16.0],
    );

    // advance-boundary cell dividers: each sort in its own cell, the
    // divider spanning the optically balanced specimen box, with knockout
    // nodes at the ends.
    let mut bounds = vec![0.0];
    let mut acc = 0.0;
    for o in &outlines {
        acc += o.width;
        bounds.push(acc);
    }
    for &b in &bounds {
        cell_dividers_colored(
            &mut sheet,
            &[f.x(b)],
            f.y(TOP_OVERSHOOT),
            side_margin,
            role::og::construction(),
            role::og::structure_point_fill(),
            STROKE_WIDTH,
        );
    }

    // One knockout node at every construction-line intersection,
    // including both overshoot lines and the optically aligned bottom rule.
    let metric_ys = [
        f.y(TOP_OVERSHOOT),
        f.y(768.0),
        f.y(576.0),
        f.y(0.0),
        f.y(-16.0),
        side_margin,
    ];
    for &b in &bounds {
        for &y in &metric_ys {
            construction_node(&mut sheet, f.x(b), y);
        }
    }

    // glyphs: hero fill + grid-level point language
    let body_colors = [
        role::og::gradient_1(),
        role::og::gradient_2(),
        role::og::gradient_3(),
        role::og::gradient_4(),
    ];
    let mut ox = 0.0;
    for (o, body_color) in outlines.iter().zip(body_colors) {
        draw_body_styled(
            &mut sheet,
            o,
            SCALE,
            f.x(ox),
            f.baseline,
            body_color,
            GLYPH_FILL_ALPHA,
            role::og::construction(),
            STROKE_WIDTH,
        );
        draw_points_styled(
            &mut sheet,
            o,
            SCALE,
            f.x(ox),
            f.baseline,
            role::og::construction(),
            role::og::structure_point(),
            role::og::correction_point(),
            role::og::structure_point_fill(),
            role::og::correction_point_fill(),
            PointStyle {
                smooth_size: POINT_SIZE,
                corner_size: POINT_SIZE,
                off_curve_size: POINT_SIZE,
                correction_filled: false,
                stroke_width: STROKE_WIDTH,
            },
        );
        ox += o.width;
    }

    for measurement in GLYPH_MEASUREMENTS {
        glyph_measurement(&mut sheet, &f, bounds[measurement.glyph], measurement);
    }

    // Locally placed capped dimensions. The four row offsets above are
    // intentionally direct art-direction controls.
    let measurement_center = (side_margin + f.baseline) / 2.0 + BOTTOM_MEASUREMENT_RAISE;
    let mut ox = 0.0;
    for (j, o) in outlines.iter().enumerate() {
        let (i0, i1) = ink(o);
        let y = measurement_center + MEASUREMENT_ROW_OFFSETS[j];
        let x0 = f.x(ox);
        let xi0 = f.x(ox + i0);
        let xi1 = f.x(ox + i1);
        let x1 = f.x(ox + o.width);
        measurement_span(
            &mut sheet,
            x0,
            xi0,
            y,
            &format!("{}", i0.round()),
            role::og::structure_point(),
            x0 + MEASUREMENT_EDGE_LABEL_INSET,
            -1,
        );
        measurement_span(
            &mut sheet,
            xi0,
            xi1,
            y,
            &format!("{}", (i1 - i0).round()),
            role::og::structure_point(),
            (xi0 + xi1) / 2.0,
            0,
        );
        measurement_span(
            &mut sheet,
            xi1,
            x1,
            y,
            &format!("{}", (o.width - i1).round()),
            role::og::structure_point(),
            x1 - MEASUREMENT_EDGE_LABEL_INSET,
            1,
        );
        ox += o.width;
    }

    if SHOW_TITLE {
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
            .text(TITLE, f.x(0.0), side_margin + TITLE_BASELINE_OFFSET);
    }

    let outputs = OutputPaths::from_args();
    sheet.save(&renderer, &outputs.blog("share-card.png"));
    sheet.save(&renderer, &outputs.og("virtua-grotesk.png"));
}
