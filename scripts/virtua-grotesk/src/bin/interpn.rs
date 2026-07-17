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
use kurbo::{PathEl, Point};
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
    // ONE 8-unit grid across the whole canvas; the three n's sit on it, big.
    let edge = 88.0; // pure-grid border; outlines stay inside it
    let ink_h = 584.0; // baseline..x-height+overshoot, in glyph units
    let inner = W - 2.0 * edge;
    // scale for a mild ~20% overlap of three Bold-width glyphs across the width
    let s = inner / (2.6 * n_bold.width);
    let gp = 8.0 * s; // pixel pitch of the 8-grid
    // center the ink band (0..ink_h) vertically, baseline on a grid line
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

    // three n's spread across the inner width, overlapping to zoom in, each
    // origin snapped to the grid so on-8 points land on grid lines
    let bw = n_bold.width * s;
    let pitch = (inner - bw) / 2.0; // origin spacing (< bw, so they overlap)
    for (i, (t, _, stem)) in weights.iter().enumerate() {
        let o = interp(&n_reg, &n_bold, *t);
        let x0 = ((edge + i as f64 * pitch + (bw - o.width * s) / 2.0) / gp).round() * gp;
        draw_body(&mut sheet, &o, s, x0, baseline);
        draw_points(&mut sheet, &o, s, x0, baseline);

        // stem dimension: left ink edge to left edge + stem, across mid x-height
        let leftx = o.points.iter().fold(f64::MAX, |m, (px, _, _)| m.min(*px));
        let dy = baseline + 300.0 * s;
        sheet.dim_h(x0 + leftx * s, x0 + (leftx + *stem as f64) * s, dy, &stem.to_string(), green());
        // apex coordinate: the top on-curve point of the arch
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

    // shared vertical metrics, tagged once on the left
    sheet.metric_tag("x-height 576", edge, baseline + 576.0 * s, true, -1);
    sheet.metric_tag("baseline 0", edge, baseline, true, -1);

    let here = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let post = here
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("src/content/blog/virtua-grotesk");
    sheet.save(&renderer, &post.join("fig-interp-outlines.png"));
}
