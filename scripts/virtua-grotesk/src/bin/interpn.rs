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

fn main() {
    let mono_path = inputs::geist_mono();
    let sources = inputs::virtua_sources();
    let reg_dir = sources.join("VirtuaGrotesk-Regular.ufo/glyphs");
    let bold_dir = sources.join("VirtuaGrotesk-Bold.ufo/glyphs");

    let mut renderer = Renderer::new(W as u32, H as u32);
    let mono = load_family(&mut renderer, mono_path.to_str().unwrap());
    let mut sheet = new_sheet(&renderer, &mono);

    let regular = load_outline(&reg_dir, "n");
    let bold = load_outline(&bold_dir, "n");
    let midpoint = interp(&regular, &bold, 0.5);
    let outlines = [regular, midpoint, bold];
    let stems = [96.0, 144.0, 192.0];
    let fills = [
        role::figure::red(),
        role::figure::orange(),
        role::figure::green(),
    ];

    const S: f64 = 1.26;
    const BASELINE: f64 = 286.0;
    const GAP: f64 = 32.0;
    let total = outlines.iter().map(|o| o.width * S).sum::<f64>() + 2.0 * GAP;
    let mut x = MARGIN + (W - 2.0 * MARGIN - total) / 2.0;

    let frame = Frame {
        s: S,
        x0: 0.0,
        baseline: BASELINE,
    };
    metric_lines(&mut sheet, &frame, &[0.0, 576.0], &[-16.0, 592.0]);

    for ((outline, stem), fill) in outlines.iter().zip(stems).zip(fills) {
        draw_figure_glyph(&mut sheet, outline, S, x, BASELINE, fill);

        let ink_left = outline
            .points
            .iter()
            .map(|(px, _, _)| *px)
            .fold(f64::INFINITY, f64::min);
        let ink_right = outline
            .points
            .iter()
            .map(|(px, _, _)| *px)
            .fold(f64::NEG_INFINITY, f64::max);
        sheet.dim_h(
            x + ink_left * S,
            x + (ink_left + stem) * S,
            BASELINE + 230.0 * S,
            &format!("{}", stem as i64),
            role::figure::pen(),
        );
        sheet.dim_h(
            x + (ink_left + stem) * S,
            x + (ink_right - stem) * S,
            BASELINE + 330.0 * S,
            &format!("{}", (ink_right - ink_left - 2.0 * stem) as i64),
            role::figure::pen(),
        );

        x += outline.width * S + GAP;
    }

    let outputs = OutputPaths::from_args();
    sheet.save(&renderer, &outputs.blog("fig-interp-outlines.png"));
}
