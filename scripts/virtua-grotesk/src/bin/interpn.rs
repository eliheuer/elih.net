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
    // ONE 8-unit grid; three n's sit on it, spaced by a constant ink-gap.
    // The grid pitch divides both canvas dimensions (gcd(2520,1320) = 120),
    // so grid lines land exactly on all four edges.
    let gp = 15.0; // px per 8 units
    let s = gp / 8.0; // 1.875
    let ink_h = 584.0; // baseline..x-height+overshoot
    let baseline = ((H - ink_h * s) / 2.0 / gp).round() * gp - gp;
    let edge = 72.0; // horizontal inset for the outlines
    let grid = Color::rgb(0x2c, 0x2c, 0x2c);

    // uniform 8-unit grid, phase 0 so lines coincide with every edge;
    // the edge lines themselves are skipped (they'd double the image border)
    sheet.ctx.no_fill().stroke(grid).stroke_width(PEN_LIGHT);
    let mut gx = gp;
    while gx <= W - gp + 0.1 {
        sheet.ctx.line(gx, 0.0, gx, H);
        gx += gp;
    }
    let mut gy = gp;
    while gy <= H - gp + 0.1 {
        sheet.ctx.line(0.0, gy, W, gy);
        gy += gp;
    }

    let _ = edge;
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

    // 8 units of pure grid on every side: outer ink pinned at 120px exactly,
    // the middle n centered between its neighbors, origins on grid lines
    let inset = 8.0 * gp;
    let x0_l = ((inset - ink[0].0 * s) / gp).round() * gp;
    let x0_r = (((W - inset) - ink[2].1 * s) / gp).round() * gp;
    let mid_ctr = ((x0_l + ink[0].1 * s) + (x0_r + ink[2].0 * s)) / 2.0;
    let x0_m = ((mid_ctr - (ink[1].0 + ink[1].1) / 2.0 * s) / gp).round() * gp;
    let origins = [x0_l, x0_m, x0_r];

    let mut lab = Labeler::new();

    // one hue per weight so the overlapping outlines stay distinct
    let hues = [
        Color::rgb(0xff, 0x45, 0x35), // Regular: red
        Color::rgb(0xff, 0x98, 0x22), // 1/2: orange
        Color::rgb(0xff, 0xd2, 0x3c), // Bold: yellow
    ];
    for (i, o) in outs.iter().enumerate() {
        let (il, ir) = ink[i];
        let x0 = origins[i];

        let place = Affine::new([s, 0.0, 0.0, s, x0, baseline]);
        let canvas_path = place * o.path.clone();
        lab.ink(canvas_path.clone());
        let (hr, hg, hb) = (hues[i].r, hues[i].g, hues[i].b);
        sheet.ctx
            .fill(Color::rgba(hr, hg, hb, 34))
            .stroke(hues[i])
            .stroke_width(PEN);
        sheet.ctx.draw_path(place * o.path.clone());
        draw_points_handle_stroke(&mut sheet, o, s, x0, baseline, purple());

        let stem = weights[i].2 as f64;
        let ink_w = ir - il;
        // dimensions in the glyph's own hue so spans read as belonging to it
        dim_arrow(&mut sheet, &mut lab, x0 + il * s, x0 + (il + stem) * s,
                  baseline + 200.0 * s, &fmt_val(stem as i64), hues[i]);
        let counter = (ink_w - 2.0 * stem) as i64;
        dim_arrow(&mut sheet, &mut lab, x0 + (il + stem) * s, x0 + (ir - stem) * s,
                  baseline + 260.0 * s, &fmt_val(counter), hues[i]);

        // handle lengths (purple, like the system sheets): the four longest
        let mut hl: Vec<(f64, f64, f64, f64, f64)> = o
            .handles
            .iter()
            .map(|((ax, ay), (hx, hy))| {
                let len = ((hx - ax).powi(2) + (hy - ay).powi(2)).sqrt();
                (len, *ax, *ay, *hx, *hy)
            })
            .collect();
        hl.sort_by(|a, b| b.0.total_cmp(&a.0));
        let glyph_ctr = (il + ir) / 2.0;
        for (len, ax, ay, hx, hy) in hl.into_iter().take(2) {
            lab.marker(x0 + hx * s, baseline + hy * s);
            let (mx, my) = ((ax + hx) / 2.0, (ay + hy) / 2.0);
            let (cmx, cmy) = (x0 + mx * s, baseline + my * s);
            let dir = outward_dir(&canvas_path, cmx, cmy, 26.0);
            lab.anchor(cmx, cmy);
            lab.queue(cmx, cmy, dir, fmt_val(len.round() as i64), purple(), false, i);
        }

        // longest axis-aligned line segments, length + sum in the glyph hue
        let mut segs: Vec<(f64, f64, f64, f64, f64)> = Vec::new();
        let mut cur = (0.0f64, 0.0f64);
        let mut start = cur;
        for el in o.path.elements() {
            match el {
                PathEl::MoveTo(pt) => { cur = (pt.x, pt.y); start = cur; }
                PathEl::LineTo(pt) => {
                    let nxt = (pt.x, pt.y);
                    if (nxt.0 - cur.0).abs() < 0.01 || (nxt.1 - cur.1).abs() < 0.01 {
                        let len = (nxt.0 - cur.0).abs() + (nxt.1 - cur.1).abs();
                        segs.push((len, cur.0, cur.1, nxt.0, nxt.1));
                    }
                    cur = nxt;
                }
                PathEl::CurveTo(_, _, pt) => cur = (pt.x, pt.y),
                PathEl::QuadTo(_, pt) => cur = (pt.x, pt.y),
                PathEl::ClosePath => cur = start,
            }
        }
        // vertical segments only: the horizontal ones collide with the dims
        segs.retain(|(_, ax, _, bx, _)| (bx - ax).abs() < 0.01);
        segs.sort_by(|a, b| b.0.total_cmp(&a.0));
        for (len, ax, ay, bx, by) in segs.into_iter().take(if i == 0 { 1 } else { 0 }) {
            let (mx, my) = ((ax + bx) / 2.0, (ay + by) / 2.0);
            let dir = if mx < glyph_ctr { (1.0, 0.0) } else { (-1.0, 0.0) };
            let (cmx, cmy) = (x0 + mx * s, baseline + my * s);
            lab.queue(cmx, cmy, dir, fmt_val(len.round() as i64), hues[i], false, i);
        }

        // queue a coordinate for EVERY on-curve point; placed after the
        // loop by the point-labeling pass
        // every on-curve point is queued; the near-or-nothing placer
        // labels the uncrowded ones and skips the crowded ones
        for (px, py, _) in &o.points {
            let cpt = (x0 + px * s, baseline + py * s);
            lab.anchor(cpt.0, cpt.1);
            let dir = outward_dir(&canvas_path, cpt.0, cpt.1, 26.0);
            lab.queue(cpt.0, cpt.1, dir, format!("{},{}", *px as i64, *py as i64), gray(), true, i);
        }


        // weight label inside the counter, above the baseline
        let wl_x = x0 + (il + ink_w / 2.0) * s;
        let wl_y = baseline + 56.0 * s;
        sheet.label_padded(weights[i].1, wl_x, wl_y, LABEL_TEXT, dim_color(), 0);
        lab.obstacle_text(wl_x, wl_y, LABEL_TEXT, weights[i].1);

    }

    // all queued labels placed simultaneously (annealing, lib.rs Labeler)
    lab.place_all(&mut sheet);

    let here = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let post = here
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("src/content/blog/virtua-grotesk");
    sheet.save(&renderer, &post.join("fig-interp-outlines.png"));
}
