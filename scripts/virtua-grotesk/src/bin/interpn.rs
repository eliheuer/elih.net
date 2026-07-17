//! fig-interp-outlines.png : the real n outline interpolated from Regular
//! to Bold at 1/4, 1/2, 3/4, every intermediate a grid-native glyph.
//!
//! Each n sits on its own 8-unit grid. Points are colored by grid level:
//! green on the 8-unit structure grid, red off 8 on the 2-unit correction
//! grid. The stem is dimensioned under each weight: 96, 120, 144, 168,
//! 192, every value landing on the grid.
//!
//!     cargo run --release --bin interpn

use designbot::prelude::*;
use designbot_render::Renderer;
use kurbo::{Affine, PathEl, Point};
#[allow(unused_imports)]
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
    let home = std::env::var("HOME").unwrap();
    let mono_path = format!("{home}/GH/repos/google-fonts/ofl/geistmono/GeistMono[wght].ttf");
    let sources = std::path::PathBuf::from(&home).join("GH/repos/virtua-grotesk/sources");
    let reg_dir = sources.join("VirtuaGrotesk-Regular.ufo/glyphs");
    let bold_dir = sources.join("VirtuaGrotesk-Bold.ufo/glyphs");

    let mut renderer = Renderer::new(W as u32, H as u32);
    let mono = load_family(&mut renderer, &mono_path);
    let mut sheet = new_sheet(&renderer, &mono);

    let n_reg = load_outline(&reg_dir, "n");
    let n_bold = load_outline(&bold_dir, "n");

    let weights: [(f64, &str, i64); 3] = [
        (0.0, "Regular", 96),
        (0.5, "1/2", 144),
        (1.0, "Bold", 192),
    ];
    let cols = weights.len() as f64;

    let _ = cols;
    // ONE 8-unit grid; three n's sit on it, sized to fill the height, spaced
    // by a constant ink-gap so the tracking is even.
    let edge = 72.0;
    let ink_h = 584.0; // baseline..x-height+overshoot
    let s = (H - 2.0 * edge) / ink_h; // fill the height
    let gp = 8.0 * s;
    let baseline = ((H / 2.0 - ink_h / 2.0 * s) / gp).round() * gp;
    let grid_dim = Color::rgb(0x24, 0x24, 0x24);
    let grid_maj = Color::rgb(0x40, 0x40, 0x40);

    // one continuous grid, phase-aligned to x = 0 and to the baseline
    let mut gx = 0.0;
    while gx <= W + 0.1 {
        let maj = ((gx / gp).round() as i64) % 8 == 0;
        sheet.ctx.no_fill().stroke(if maj { grid_maj } else { grid_dim }).stroke_width(PEN_LIGHT);
        sheet.ctx.line(gx, 0.0, gx, H);
        gx += gp;
    }
    let mut gy = baseline.rem_euclid(gp);
    while gy <= H + 0.1 {
        let maj = (((gy - baseline) / gp).round() as i64).rem_euclid(8) == 0;
        sheet.ctx.no_fill().stroke(if maj { grid_maj } else { grid_dim }).stroke_width(PEN_LIGHT);
        sheet.ctx.line(0.0, gy, W, gy);
        gy += gp;
    }

    // precompute the three outlines and their ink extents
    let outs: Vec<Outline> = weights.iter().map(|(t, _, _)| interp(&n_reg, &n_bold, *t)).collect();
    let ink: Vec<(f64, f64)> = outs
        .iter()
        .map(|o| {
            let l = o.points.iter().fold(f64::MAX, |m, (x, _, _)| m.min(*x));
            let r = o.points.iter().fold(f64::MIN, |m, (x, _, _)| m.max(*x));
            (l, r)
        })
        .collect();
    let sum_ink: f64 = ink.iter().map(|(l, r)| r - l).sum();
    // even gap (pixels) so the three ink boxes fill the inner width
    let gap = ((W - 2.0 * edge) - sum_ink * s) / 2.0;

    // place left to right with a constant ink gap; origins snapped to the grid
    let mut prev_right = edge; // pixel x where the next ink block starts
    for (i, o) in outs.iter().enumerate() {
        let (il, ir) = ink[i];
        let x0 = ((prev_right - il * s) / gp).round() * gp;
        prev_right = x0 + ir * s + gap;

        // stroke-only outline so overlaps read as clean crossings, not muddy fill
        let place = Affine::new([s, 0.0, 0.0, s, x0, baseline]);
        sheet.ctx.no_fill().stroke(Color::rgb(0xcc, 0xcc, 0xcc)).stroke_width(PEN);
        sheet.ctx.draw_path(place * o.path.clone());
        draw_points(&mut sheet, o, s, x0, baseline);

        let stem = weights[i].2;
        let dy = baseline + 300.0 * s;
        sheet.dim_h(x0 + il * s, x0 + (il + stem as f64) * s, dy, &stem.to_string(), green());
        if let Some((px, py, _)) = o
            .points
            .iter()
            .filter(|(_, _, r)| matches!(r, PtRole::Smooth))
            .max_by(|a, b| a.1.total_cmp(&b.1))
        {
            sheet.label_padded(
                &format!("({},{})", *px as i64, *py as i64),
                x0 + px * s,
                baseline + py * s + 26.0,
                SMALL_TEXT,
                gray(),
                0,
            );
        }
    }

    let here = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let post = here
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("src/content/blog/virtua-grotesk");
    sheet.save(&renderer, &post.join("fig-interp-outlines.png"));
}
