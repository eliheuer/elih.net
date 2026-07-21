//! Three real n outlines interpolated from the current Virtua Grotesk
//! Regular and Bold sources. The middle outline is calculated at render time
//! and snapped to the same 2-unit source grid.
//!
//! This composition deliberately keeps only the visual proof: three large
//! glyphs, their source points, and the stem/counter dimensions that change.
//! Run with `cargo run --release --bin interpn`.

use designbot::prelude::Color;
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
    sheet.ctx.line_cap("round");

    let regular = load_outline(&reg_dir, "n");
    let bold = load_outline(&bold_dir, "n");
    let midpoint = interp(&regular, &bold, 0.5);
    let outlines = [regular, midpoint, bold];
    let stems = [96.0, 144.0, 192.0];
    let fills = [
        role::figure::red(),
        role::figure::orange(),
        role::figure::yellow(),
    ];

    const S: f64 = 1.26;
    const BASELINE: f64 = 286.0;
    // A 32-unit sort gap keeps every glyph origin on the continuous grid.
    const GAP: f64 = 32.0 * S;
    let total = outlines.iter().map(|o| o.width * S).sum::<f64>() + 2.0 * GAP;
    let mut x = MARGIN + (W - 2.0 * MARGIN - total) / 2.0;

    let frame = Frame {
        s: S,
        x0: 0.0,
        baseline: BASELINE,
    };

    // One real 8-unit source grid runs continuously behind all three sorts.
    // Snap the first origin to the grid so every visible point can be checked
    // directly against the same field.
    let grid_step = 8.0 * S;
    sheet
        .ctx
        .no_fill()
        .stroke(role::grid::faint())
        .stroke_width(line::FINE);
    let mut gx = x.rem_euclid(grid_step);
    while gx <= W {
        sheet.ctx.line(gx, 0.0, gx, H);
        gx += grid_step;
    }
    let mut gy = BASELINE.rem_euclid(grid_step);
    while gy <= H {
        sheet.ctx.line(0.0, gy, W, gy);
        gy += grid_step;
    }

    // The four font metrics use the same near-black pen as the outlines.
    {
        let ctx = &mut sheet.ctx;
        ctx.no_fill()
            .stroke(role::figure::pen())
            .stroke_width(line::HERO);
        ctx.line_dash(&[14.0, 14.0]);
        for uy in [-16.0, 592.0] {
            ctx.line(0.0, frame.y(uy), W, frame.y(uy));
        }
        ctx.line_dash(&[]);
        for uy in [0.0, 576.0] {
            ctx.line(0.0, frame.y(uy), W, frame.y(uy));
        }
    }

    let sums = ["64+32", "128+16", "128+64"];
    let counters = [272.0, 232.0, 192.0];
    let counter_sums = ["256+16", "128+64+32+8", "128+64"];

    let measure =
        |sheet: &mut Sheet, x0: f64, x1: f64, y: f64, value: &str, sum: &str, background: Color| {
            sheet
                .ctx
                .no_fill()
                .stroke(role::figure::pen())
                .stroke_width(8.0);
            sheet.ctx.line(x0, y, x1, y);
            sheet.ctx.line(x0, y - 18.0, x0, y + 18.0);
            sheet.ctx.line(x1, y - 18.0, x1, y + 18.0);
            let cx = (x0 + x1) / 2.0;
            sheet.label_padded_weighted_on(
                value,
                cx,
                y + 42.0,
                42.0,
                role::figure::pen(),
                0,
                background,
                560.0,
            );
            sheet.label_padded_weighted_on(
                sum,
                cx,
                y - 30.0,
                34.0,
                role::figure::pen(),
                0,
                background,
                500.0,
            );
        };

    for (index, ((outline, stem), fill)) in outlines.iter().zip(stems).zip(fills).enumerate() {
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
        measure(
            &mut sheet,
            x + ink_left * S,
            x + (ink_left + stem) * S,
            BASELINE + 230.0 * S,
            &format!("{}", stem as i64),
            sums[index],
            fill,
        );
        measure(
            &mut sheet,
            x + (ink_left + stem) * S,
            x + (ink_right - stem) * S,
            BASELINE + 330.0 * S,
            &format!("{}", counters[index] as i64),
            counter_sums[index],
            role::figure::background(),
        );

        x += outline.width * S + GAP;
    }

    let outputs = OutputPaths::from_args();
    sheet.save(&renderer, &outputs.blog("fig-interp-outlines.png"));
}
