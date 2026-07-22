//! Three real n outlines interpolated from the current Virtua Grotesk
//! Regular and Bold sources. The middle outline is calculated at render time
//! and snapped to the same 2-unit source grid.
//!
//! This composition deliberately keeps only the visual proof: three large
//! glyphs, their source points, and the stem/counter dimensions that change.
//! Run with `cargo run --release --bin interpn`.

use designbot_render::Renderer;
use kurbo::{BezPath, PathEl, Point};
use virtua_grotesk_figures::*;

fn snap2(v: f64) -> f64 {
    (v / 2.0).round() * 2.0
}

fn lp(a: Point, b: Point, t: f64) -> Point {
    Point::new(snap2(a.x + (b.x - a.x) * t), snap2(a.y + (b.y - a.y) * t))
}

fn interp(a: &Outline, b: &Outline, t: f64) -> Outline {
    let mut path = BezPath::new();
    for (ea, eb) in a.path.elements().iter().zip(b.path.elements()) {
        path.push(match (ea, eb) {
            (PathEl::MoveTo(p), PathEl::MoveTo(q)) => PathEl::MoveTo(lp(*p, *q, t)),
            (PathEl::LineTo(p), PathEl::LineTo(q)) => PathEl::LineTo(lp(*p, *q, t)),
            (PathEl::QuadTo(p1, p), PathEl::QuadTo(q1, q)) => {
                PathEl::QuadTo(lp(*p1, *q1, t), lp(*p, *q, t))
            }
            (PathEl::CurveTo(p1, p2, p), PathEl::CurveTo(q1, q2, q)) => {
                PathEl::CurveTo(lp(*p1, *q1, t), lp(*p2, *q2, t), lp(*p, *q, t))
            }
            (PathEl::ClosePath, PathEl::ClosePath) => PathEl::ClosePath,
            _ => panic!("n masters are not point-compatible"),
        });
    }
    let points = a
        .points
        .iter()
        .zip(&b.points)
        .map(|((ax, ay, r), (bx, by, _))| {
            (snap2(ax + (bx - ax) * t), snap2(ay + (by - ay) * t), *r)
        })
        .collect();
    let handles = a
        .handles
        .iter()
        .zip(&b.handles)
        .map(|(((aax, aay), (ahx, ahy)), ((bax, bay), (bhx, bhy)))| {
            (
                (snap2(aax + (bax - aax) * t), snap2(aay + (bay - aay) * t)),
                (snap2(ahx + (bhx - ahx) * t), snap2(ahy + (bhy - ahy) * t)),
            )
        })
        .collect();
    Outline {
        path,
        points,
        handles,
        width: snap2(a.width + (b.width - a.width) * t),
    }
}

#[derive(Clone, Copy, Debug)]
struct NDimensions {
    ink_left: f64,
    inner_left: f64,
    inner_right: f64,
}

/// Read the stem and counter edges from the outline being drawn. The two
/// smooth on-curve points at y=344 define the straight counter walls in every
/// compatible n source. Fail loudly if that topology changes instead of
/// displaying a stale hardcoded label.
fn n_dimensions(outline: &Outline) -> NDimensions {
    let ink_left = outline
        .points
        .iter()
        .map(|(x, _, _)| *x)
        .fold(f64::INFINITY, f64::min);
    let mut counter_edges: Vec<f64> = outline
        .points
        .iter()
        .filter_map(|(x, y, role)| {
            if (*y - 344.0).abs() < f64::EPSILON && !matches!(role, PtRole::Off) {
                Some(*x)
            } else {
                None
            }
        })
        .collect();
    counter_edges.sort_by(f64::total_cmp);
    assert_eq!(
        counter_edges.len(),
        2,
        "n counter measurement expects two on-curve edges at y=344"
    );
    NDimensions {
        ink_left,
        inner_left: counter_edges[0],
        inner_right: counter_edges[1],
    }
}

fn main() {
    let mono_path = inputs::geist_mono();
    let sources = inputs::virtua_sources();
    let reg_dir = sources.join("VirtuaGrotesk-Regular.ufo/glyphs");
    let bold_dir = sources.join("VirtuaGrotesk-Bold.ufo/glyphs");

    let mut renderer = Renderer::new(W as u32, H as u32);
    let mono = load_family(&mut renderer, mono_path.to_str().unwrap());
    let mut sheet = new_sheet(&renderer, &mono);
    sheet.ctx.line_cap("round");
    let technical = TechnicalStyle::section_three()
        .with_grid_level_points()
        .with_grid(32.0, line::HERO)
        .with_subtle_grid();

    let regular = load_outline(&reg_dir, "n");
    let bold = load_outline(&bold_dir, "n");
    let midpoint = interp(&regular, &bold, 0.5);
    let outlines = [regular, midpoint, bold];
    let dimensions: [NDimensions; 3] = std::array::from_fn(|index| n_dimensions(&outlines[index]));
    let fills = [
        role::figure::orange(),
        role::figure::yellow(),
        role::figure::green(),
    ];

    // Composition controls. Rendering conventions come from the shared
    // section 03 preset; only this figure's placement remains local.
    const OUTER_MARGIN: f64 = 88.0;
    const INK_GAP_UNITS: f64 = 32.0;

    let ink: Vec<(f64, f64, f64, f64)> = outlines
        .iter()
        .map(|outline| {
            outline.points.iter().fold(
                (
                    f64::INFINITY,
                    f64::NEG_INFINITY,
                    f64::INFINITY,
                    f64::NEG_INFINITY,
                ),
                |(left, right, bottom, top), (px, py, _)| {
                    (left.min(*px), right.max(*px), bottom.min(*py), top.max(*py))
                },
            )
        })
        .collect();

    // Size from the visible ink rather than the sorts. This recovers the
    // larger earlier composition while preserving a constant 32-unit gap.
    let total_ink_units = ink
        .iter()
        .map(|(left, right, _, _)| right - left)
        .sum::<f64>()
        + 2.0 * INK_GAP_UNITS;
    let s = (W - 2.0 * OUTER_MARGIN) / total_ink_units;
    let bottom = ink
        .iter()
        .map(|(_, _, bottom, _)| *bottom)
        .fold(f64::INFINITY, f64::min);
    let top = ink
        .iter()
        .map(|(_, _, _, top)| *top)
        .fold(f64::NEG_INFINITY, f64::max);
    let baseline = (H - (top - bottom) * s) / 2.0 - bottom * s;

    // All three origins differ by multiples of eight source units. One grid
    // therefore remains truthful across the full image.
    let mut origins_px = Vec::with_capacity(outlines.len());
    let mut next_ink_left = OUTER_MARGIN;
    for (left, right, _, _) in &ink {
        let origin = next_ink_left - left * s;
        origins_px.push(origin);
        next_ink_left = origin + right * s + INK_GAP_UNITS * s;
    }
    let origins: Vec<f64> = origins_px.iter().map(|origin| origin / s).collect();

    let mut frame = Frame {
        s,
        x0: 0.0,
        baseline,
    };
    technical.center_grid_vertically(&mut frame);

    // One 32-unit source grid runs continuously behind all three sorts.
    technical.continuous_background_grid(&mut sheet, &frame, origins[0]);

    const MEASUREMENT_Y: f64 = 192.0;
    for (index, ((outline, fill), x)) in outlines.iter().zip(fills).zip(origins).enumerate() {
        technical.glyph(&mut sheet, outline, &frame, x, fill);

        let dimension = dimensions[index];
        let stem = dimension.inner_left - dimension.ink_left;
        let counter = dimension.inner_right - dimension.inner_left;
        technical.measurement(
            &mut sheet,
            &frame,
            x,
            TechnicalMeasurement::edges(
                index,
                (dimension.ink_left, MEASUREMENT_Y),
                (dimension.inner_left, MEASUREMENT_Y),
                stem as i64,
            ),
        );
        technical.measurement(
            &mut sheet,
            &frame,
            x,
            TechnicalMeasurement::edges(
                index,
                (dimension.inner_left, MEASUREMENT_Y),
                (dimension.inner_right, MEASUREMENT_Y),
                counter as i64,
            ),
        );
    }

    let outputs = OutputPaths::from_args();
    sheet.save(&renderer, &outputs.blog("fig-interp-outlines.png"));
}
