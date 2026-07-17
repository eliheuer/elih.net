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

    let edge = 20.0; // near-edge padding; outlines stay well inside it
    let col_w = (W - 2.0 * edge) / cols;
    // scale to fill each column with the widest weight (Bold), keep a hair of gap
    let s = 0.985 * col_w / n_bold.width;
    // vertically center the glyph body (grid_lo..grid_hi) in the canvas
    let (grid_lo, grid_hi) = (-24.0f64, 620.0f64);
    let baseline = H / 2.0 - (grid_lo + grid_hi) / 2.0 * s;
    // draw the grid tall, filling the canvas, so the field reads as one plane
    let field_top = H - edge;
    let field_bot = edge;
    let gy_hi = (field_top - baseline) / s;
    let gy_lo = (field_bot - baseline) / s;
    let grid_dim = Color::rgb(0x26, 0x26, 0x26);
    let grid_maj = Color::rgb(0x42, 0x42, 0x42);

    for (i, (t, _, _)) in weights.iter().enumerate() {
        let o = interp(&n_reg, &n_bold, *t);
        let x0 = edge + i as f64 * col_w + (col_w - o.width * s) / 2.0;

        // 8-unit grid, brighter every 64, spanning the full canvas height
        let mut gx = 0.0;
        while gx <= o.width + 0.1 {
            let maj = (gx as i64) % 64 == 0;
            sheet.ctx.no_fill().stroke(if maj { grid_maj } else { grid_dim }).stroke_width(PEN_LIGHT);
            sheet.ctx.line(x0 + gx * s, baseline + gy_lo * s, x0 + gx * s, baseline + gy_hi * s);
            gx += 8.0;
        }
        let mut gy = (gy_lo / 8.0).ceil() * 8.0;
        while gy <= gy_hi + 0.1 {
            let maj = (gy as i64).rem_euclid(64) == 0;
            sheet.ctx.no_fill().stroke(if maj { grid_maj } else { grid_dim }).stroke_width(PEN_LIGHT);
            sheet.ctx.line(x0, baseline + gy * s, x0 + o.width * s, baseline + gy * s);
            gy += 8.0;
        }

        draw_body(&mut sheet, &o, s, x0, baseline);
        draw_points(&mut sheet, &o, s, x0, baseline);
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
