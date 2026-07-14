//! Optical-correction illustration for the Virtua Grotesk post, §03.
//!
//! A single cubic arch over the two-level grid (2-unit faint, 8-unit
//! brighter). Every coordinate is a multiple of 8 except the apex trio,
//! overshot +4 onto the 2-unit grid — the eye's correction. The family
//! point language colors it automatically: green = on 8 (machine), red =
//! off 8, on 2 (the hand). A "+4" dimension and a callout name the move.
//!
//! Run from this directory:
//!
//!     cargo run --release --bin optical
//!
//! Writes ../../src/content/blog/virtua-grotesk/fig-optical-correction.png.

use designbot::prelude::*;
use designbot_render::Renderer;
use kurbo::{Affine, BezPath};
#[allow(unused_imports)]
use virtua_grotesk_figures::*;

const UNIT: f64 = 11.0; // canvas px per unit: 8-cell = 88px, 2-cell = 22px

fn main() {
    let home = std::env::var("HOME").unwrap();
    let mono_path = format!("{home}/GH/repos/google-fonts/ofl/geistmono/GeistMono[wght].ttf");

    let mut renderer = Renderer::new(W as u32, H as u32);
    let mono = load_family(&mut renderer, &mono_path);
    let mut sheet = new_sheet(&renderer, &mono);

    // arch geometry, in units: on-curve ends + apex, extremum handles
    let l = (0.0, 0.0);
    let t = (64.0, 68.0); // grid intent 64; +4 overshoot, the correction
    let r = (128.0, 0.0);
    let l_up = (0.0, 40.0);
    let t_left = (24.0, 68.0);
    let t_right = (104.0, 68.0);
    let r_up = (128.0, 40.0);

    // frame: arch centered, content -8..80 units vertically
    let origin_x = MARGIN + (W - 2.0 * MARGIN - 128.0 * UNIT) / 2.0;
    let origin_y = 176.0 + 8.0 * UNIT;
    let cx = |ux: f64| origin_x + ux * UNIT;
    let cy = |uy: f64| origin_y + uy * UNIT;

    // two-level grid across the content zone
    {
        let (y0, y1) = (cy(-8.0), cy(80.0));
        let ctx = &mut sheet.ctx;
        let u_lo = ((MARGIN - origin_x) / UNIT / 2.0).floor() as i64 * 2;
        let u_hi = ((W - MARGIN - origin_x) / UNIT / 2.0).ceil() as i64 * 2;
        let mut u = u_lo;
        while u <= u_hi {
            let x = origin_x + u as f64 * UNIT;
            if x >= MARGIN && x <= W - MARGIN {
                if u.rem_euclid(8) == 0 {
                    ctx.stroke(grid()).stroke_width(PEN_LIGHT);
                } else {
                    ctx.stroke(Color::rgb(0x1a, 0x1a, 0x1a)).stroke_width(1.5);
                }
                ctx.line(x, y0, x, y1);
            }
            u += 2;
        }
        let mut v = -8i64;
        while v <= 80 {
            let y = origin_y + v as f64 * UNIT;
            if v.rem_euclid(8) == 0 {
                ctx.stroke(grid()).stroke_width(PEN_LIGHT);
            } else {
                ctx.stroke(Color::rgb(0x1a, 0x1a, 0x1a)).stroke_width(1.5);
            }
            ctx.line(MARGIN, y, W - MARGIN, y);
            v += 2;
        }
    }

    // handles
    for (on, off) in [(l, l_up), (t, t_left), (t, t_right), (r, r_up)] {
        sheet.ctx.no_fill().stroke(handle_color()).stroke_width(PEN_LIGHT);
        sheet.ctx.line(cx(on.0), cy(on.1), cx(off.0), cy(off.1));
    }

    // the curve
    {
        let mut path = BezPath::new();
        path.move_to(l);
        path.curve_to(l_up, t_left, t);
        path.curve_to(t_right, r_up, r);
        let to_canvas = Affine::translate((origin_x, origin_y)) * Affine::scale(UNIT);
        sheet.ctx.no_fill().stroke(curve_color()).stroke_width(5.0);
        sheet.ctx.draw_path(to_canvas * path);
    }

    // points, colored by grid level (the apex trio is off 8: red)
    for (p, role) in [
        (l, PtRole::Corner),
        (r, PtRole::Corner),
        (t, PtRole::Smooth),
        (l_up, PtRole::Off),
        (t_left, PtRole::Off),
        (t_right, PtRole::Off),
        (r_up, PtRole::Off),
    ] {
        let color = if on8(p.0, p.1) { green() } else { red() };
        let (px, py) = (cx(p.0), cy(p.1));
        sheet.ctx.fill(bg()).stroke(color).stroke_width(PEN);
        match role {
            PtRole::Smooth => {
                sheet.ctx.oval(px - 11.0, py - 11.0, 22.0, 22.0);
            }
            PtRole::Corner => {
                sheet.ctx.rect(px - 10.0, py - 10.0, 20.0, 20.0);
            }
            PtRole::Off => {
                sheet.ctx.oval(px - 8.0, py - 8.0, 16.0, 16.0);
            }
        }
    }

    // the +4: dimension from the 64-line (grid intent) to the apex
    sheet.dim_v(cx(88.0), cy(64.0), cy(68.0), "+4", red(), true);

    // callout + legend
    correction_callout(
        &mut sheet,
        (cx(64.0) + 14.0, cy(68.0) + 8.0),
        (cx(78.0), cy(78.0)),
        -1,
    );
    legend(&mut sheet, W - MARGIN, cy(-6.0));

    sheet.frame(
        "OPTICAL CORRECTION / THE TWO-LEVEL GRID",
        "VIRTUA GROTESK / EM 1024 = 2^10",
        "STRUCTURE LANDS ON THE 8-GRID; THE EYE'S +4 LANDS ON THE 2-GRID",
    );

    let here = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let out = here
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("src/content/blog/virtua-grotesk/fig-optical-correction.png");
    sheet.save(&renderer, &out);
}
