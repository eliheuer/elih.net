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

use designbot_render::Renderer;
use kurbo::{Affine, BezPath};
#[allow(unused_imports)]
use virtua_grotesk_figures::*;

const UNIT: f64 = 15.0; // canvas px per unit: 8-cell = 120px, 2-cell = 30px

fn main() {
    let mono_path = inputs::geist_mono();

    let mut renderer = Renderer::new(W as u32, H as u32);
    let mono = load_family(&mut renderer, mono_path.to_str().unwrap());
    let mut sheet = new_sheet(&renderer, &mono);

    // arch geometry, in units: on-curve ends + apex, extremum handles
    let l = (0.0, 0.0);
    let t = (64.0, 68.0); // grid intent 64; +4 overshoot, the correction
    let r = (128.0, 0.0);
    let l_up = (0.0, 40.0);
    let t_left = (24.0, 68.0);
    let t_right = (104.0, 68.0);
    let r_up = (128.0, 40.0);

    // Frame: the arch itself fills the card. No title or legend competes with
    // the one idea this figure needs to communicate.
    let origin_x = MARGIN + (W - 2.0 * MARGIN - 128.0 * UNIT) / 2.0;
    let origin_y = MARGIN + (H - 2.0 * MARGIN - 88.0 * UNIT) / 2.0 + 8.0 * UNIT;
    let cx = |ux: f64| origin_x + ux * UNIT;
    let cy = |uy: f64| origin_y + uy * UNIT;

    // The structural 8-unit lattice stays visible; the single correction
    // line at 68 makes the +4 overshoot legible without a dense 2-unit mesh.
    {
        let (y0, y1) = (cy(-8.0), cy(80.0));
        let ctx = &mut sheet.ctx;
        let u_lo = ((MARGIN - origin_x) / UNIT / 8.0).floor() as i64 * 8;
        let u_hi = ((W - MARGIN - origin_x) / UNIT / 8.0).ceil() as i64 * 8;
        let mut u = u_lo;
        while u <= u_hi {
            let x = origin_x + u as f64 * UNIT;
            if x >= MARGIN && x <= W - MARGIN {
                ctx.stroke(role::grid::standard()).stroke_width(line::THIN);
                ctx.line(x, y0, x, y1);
            }
            u += 8;
        }
        let mut v = -8i64;
        while v <= 80 {
            let y = origin_y + v as f64 * UNIT;
            ctx.stroke(role::grid::standard()).stroke_width(line::THIN);
            ctx.line(MARGIN, y, W - MARGIN, y);
            v += 8;
        }
        ctx.no_fill()
            .stroke(role::figure::red())
            .stroke_width(line::REGULAR)
            .line_dash(&[12.0, 12.0]);
        ctx.line(MARGIN, cy(68.0), W - MARGIN, cy(68.0));
        ctx.line_dash(&[]);
    }

    // handles
    for (on, off) in [(l, l_up), (t, t_left), (t, t_right), (r, r_up)] {
        sheet
            .ctx
            .no_fill()
            .stroke(role::bezier::handles())
            .stroke_width(line::HERO);
        sheet.ctx.line(cx(on.0), cy(on.1), cx(off.0), cy(off.1));
    }

    // the curve
    {
        let mut path = BezPath::new();
        path.move_to(l);
        path.curve_to(l_up, t_left, t);
        path.curve_to(t_right, r_up, r);
        let to_canvas = Affine::translate((origin_x, origin_y)) * Affine::scale(UNIT);
        sheet
            .ctx
            .no_fill()
            .stroke(role::figure::orange())
            .stroke_width(12.0);
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
        let fill = if on8(p.0, p.1) {
            role::figure::point_fill()
        } else {
            role::figure::correction_point_fill()
        };
        let (px, py) = (cx(p.0), cy(p.1));
        sheet
            .ctx
            .fill(fill)
            .stroke(role::figure::pen())
            .stroke_width(line::HERO);
        match role {
            PtRole::Smooth => {
                sheet.ctx.oval(px - 13.0, py - 13.0, 26.0, 26.0);
            }
            PtRole::Corner => {
                sheet.ctx.rect(px - 13.0, py - 13.0, 26.0, 26.0);
            }
            PtRole::Off => {
                sheet.ctx.oval(px - 13.0, py - 13.0, 26.0, 26.0);
            }
        }
    }

    // the +4: dimension from the 64-line (grid intent) to the apex
    sheet.dim_v(
        cx(88.0),
        cy(64.0),
        cy(68.0),
        "+4",
        role::figure::pen(),
        true,
    );

    let outputs = OutputPaths::from_args();
    sheet.save(&renderer, &outputs.blog("fig-optical-correction.png"));
}
