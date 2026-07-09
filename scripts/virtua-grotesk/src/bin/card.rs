//! Share card / hero for the Virtua Grotesk post: glyphs from the Regular
//! master drawn as outlines over the powers-of-two grid they were designed
//! on, with the Runebender point palette (smooth = green, corner = orange,
//! off-curve = purple). 2400x1260 (2x of 1200x630).
//!
//! Reads glyphs straight from the Virtua Grotesk UFO sources, assumed to be
//! a sibling checkout: ~/GH/repos/virtua-grotesk. Run from this directory:
//!
//!     cargo run --bin card
//!
//! Writes ../../src/content/blog/virtua-grotesk/share-card.png and
//! ../../public/og/virtua-grotesk.png.

use designbot::prelude::*;
use designbot_render::Renderer;
use kurbo::{Affine, BezPath};

const W: f64 = 2400.0;
const H: f64 = 1260.0;
const BASELINE_Y: f64 = 300.0; // canvas y of font y=0 (y-up canvas)
const LETTER_SPACE: f64 = 48.0;
const GLYPHS: &[&str] = &["R_", "a", "two"];

#[derive(Clone, Copy)]
enum Role {
    Smooth,
    Corner,
    Off,
}

struct Outline {
    path: BezPath,
    points: Vec<(f64, f64, Role)>,
    handles: Vec<((f64, f64), (f64, f64))>,
    width: f64,
}

fn load_outline(glif: &std::path::Path) -> Outline {
    let glyph = norad::Glyph::load(glif).expect("failed to load glif");
    let mut path = BezPath::new();
    let mut points = Vec::new();
    let mut handles = Vec::new();

    for contour in &glyph.contours {
        use norad::PointType::*;
        let pts = &contour.points;
        let n = pts.len();
        let Some(start) = pts.iter().position(|p| p.typ != OffCurve) else {
            continue;
        };
        let sp = &pts[start];
        path.move_to((sp.x, sp.y));
        let mut prev_on = (sp.x, sp.y);
        let mut pending: Vec<(f64, f64)> = Vec::new();
        for k in 1..=n {
            let p = &pts[(start + k) % n];
            match p.typ {
                OffCurve => {
                    pending.push((p.x, p.y));
                    points.push((p.x, p.y, Role::Off));
                }
                Curve if pending.len() == 2 => {
                    path.curve_to(pending[0], pending[1], (p.x, p.y));
                    handles.push((prev_on, pending[0]));
                    handles.push(((p.x, p.y), pending[1]));
                    pending.clear();
                    prev_on = (p.x, p.y);
                    push_on_curve(&mut points, p, k, n);
                }
                _ => {
                    path.line_to((p.x, p.y));
                    pending.clear();
                    prev_on = (p.x, p.y);
                    push_on_curve(&mut points, p, k, n);
                }
            }
        }
        path.close_path();
    }
    Outline { path, points, handles, width: glyph.width }
}

fn push_on_curve(points: &mut Vec<(f64, f64, Role)>, p: &norad::ContourPoint, k: usize, n: usize) {
    if k == n {
        return; // the closing point is the start point, already recorded
    }
    let role = if p.smooth { Role::Smooth } else { Role::Corner };
    points.push((p.x, p.y, role));
}

fn main() {
    let home = std::env::var("HOME").unwrap();
    let glyphs_dir =
        std::path::PathBuf::from(&home).join("GH/repos/virtua-grotesk/sources/VirtuaGrotesk-Regular.ufo/glyphs");

    let outlines: Vec<Outline> = GLYPHS
        .iter()
        .map(|name| load_outline(&glyphs_dir.join(format!("{name}.glif"))))
        .collect();

    let total_advance: f64 = outlines.iter().map(|o| o.width).sum::<f64>()
        + LETTER_SPACE * (outlines.len() - 1) as f64;
    let mut cursor = (W - total_advance) / 2.0;

    let mut ctx = Canvas::new(W, H);
    ctx.background(Color::rgb(10, 10, 10));

    // ── the powers-of-two grid, aligned to the first glyph's origin ──
    // minor = 16 units, major = 64, super = 256 (all in font units = px here)
    let origin_x = cursor;
    let mut gx = origin_x % 16.0 - 16.0;
    while gx < W {
        let u = ((gx - origin_x).round() as i64).rem_euclid(256);
        let c = if u == 0 { 66 } else if u % 64 == 0 { 44 } else { 24 };
        ctx.stroke(Color::rgb(c, c, c)).stroke_width(2.0);
        ctx.line(gx, 0.0, gx, H);
        gx += 16.0;
    }
    let mut gy = BASELINE_Y % 16.0 - 16.0;
    while gy < H {
        let u = ((gy - BASELINE_Y).round() as i64).rem_euclid(256);
        let c = if u == 0 { 66 } else if u % 64 == 0 { 44 } else { 24 };
        ctx.stroke(Color::rgb(c, c, c)).stroke_width(2.0);
        ctx.line(0.0, gy, W, gy);
        gy += 16.0;
    }

    // ── vertical metrics: ascender 832, cap 768, x-height 576, baseline 0,
    //    descender -256 — the power-of-two sums from DESIGN.md ──
    for (y, orange) in [(832.0, false), (768.0, false), (576.0, false), (0.0, true), (-256.0, false)] {
        let py = BASELINE_Y + y;
        if orange {
            ctx.stroke(Color::rgb(255, 78, 0)).stroke_width(3.0);
        } else {
            ctx.stroke(Color::rgb(96, 96, 96)).stroke_width(2.0);
        }
        ctx.line(0.0, py, W, py);
    }

    // ── glyph outlines + handles + points ──
    for outline in &outlines {
        let to_canvas = Affine::translate((cursor, BASELINE_Y));

        ctx.stroke(Color::rgb(120, 120, 120)).stroke_width(3.0);
        for ((x1, y1), (x2, y2)) in &outline.handles {
            ctx.line(cursor + x1, BASELINE_Y + y1, cursor + x2, BASELINE_Y + y2);
        }

        ctx.no_fill();
        ctx.stroke(Color::rgb(230, 230, 230)).stroke_width(6.0);
        ctx.draw_path(to_canvas * outline.path.clone());

        ctx.no_stroke();
        for (x, y, role) in &outline.points {
            let (px, py) = (cursor + x, BASELINE_Y + y);
            match role {
                Role::Smooth => {
                    ctx.fill(Color::rgb(24, 184, 111));
                    ctx.oval(px - 11.0, py - 11.0, 22.0, 22.0);
                }
                Role::Corner => {
                    ctx.fill(Color::rgb(255, 152, 15));
                    ctx.rect(px - 10.0, py - 10.0, 20.0, 20.0);
                }
                Role::Off => {
                    ctx.fill(Color::rgb(140, 108, 255));
                    ctx.oval(px - 8.0, py - 8.0, 16.0, 16.0);
                }
            }
        }
        cursor += outline.width + LETTER_SPACE;
    }

    let renderer = Renderer::new(W as u32, H as u32);
    let here = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let repo = here.parent().unwrap().parent().unwrap();
    for out in [
        repo.join("src/content/blog/virtua-grotesk/share-card.png"),
        repo.join("public/og/virtua-grotesk.png"),
    ] {
        std::fs::create_dir_all(out.parent().unwrap()).unwrap();
        renderer.render_to_png(&ctx, out.to_str().unwrap()).unwrap();
        println!("wrote {}", out.display());
    }
}
