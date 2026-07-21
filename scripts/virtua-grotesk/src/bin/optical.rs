//! Optical-correction illustration for the Virtua Grotesk post, §03.
//!
//! A single cubic arch over the 8-unit grid. Every coordinate is a
//! multiple of 8 except the apex trio, overshot +4 onto the 2-unit grid.
//! Point fills identify the structural and optical-correction tiers.
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

// Composition controls. Keep these together so the figure can be art-directed
// directly without changing the shared illustration system.
const UNIT: f64 = 15.0; // canvas px per unit: 8-cell = 120px, 2-cell = 30px
const STROKE: f64 = line::EXTRA_HEAVY;
const POINT_SIZE: f64 = 44.0;
const ARCH_WIDTH: f64 = 24.0; // three 8-unit cells at the feet
const INNER_OVERSHOOT: f64 = 2.0; // lift the inner apex trio onto the 2-unit grid

fn main() {
    let mono_path = inputs::geist_mono();

    let mut renderer = Renderer::new(W as u32, H as u32);
    let mono = load_family(&mut renderer, mono_path.to_str().unwrap());
    let mut sheet = new_sheet(&renderer, &mono);
    sheet.ctx.line_cap("round");

    // arch geometry, in units: on-curve ends + apex, extremum handles
    let l = (0.0, 0.0);
    let t = (64.0, 68.0); // grid intent 64; +4 overshoot, the correction
    let r = (128.0, 0.0);
    let l_up = (0.0, 40.0);
    let t_left = (24.0, 68.0);
    let t_right = (104.0, 68.0);
    let r_up = (128.0, 40.0);

    // A second cubic follows the outer curve inward. Both feet close with
    // chamfers whose endpoints remain on the 8-unit structure grid.
    let inner_l = (ARCH_WIDTH, 0.0);
    let inner_t = (64.0, t.1 - ARCH_WIDTH + INNER_OVERSHOOT);
    let inner_r = (128.0 - ARCH_WIDTH, 0.0);
    let inner_l_up = (ARCH_WIDTH, 24.0);
    let inner_t_left = (40.0, inner_t.1);
    let inner_t_right = (88.0, inner_t.1);
    let inner_r_up = (128.0 - ARCH_WIDTH, 24.0);

    let left_bottom_outer = (8.0, -8.0);
    let left_bottom_inner = (16.0, -8.0);
    let right_bottom_inner = (112.0, -8.0);
    let right_bottom_outer = (120.0, -8.0);

    // Frame: the arch itself fills the card. No title or legend competes with
    // the one idea this figure needs to communicate.
    let origin_x = MARGIN + (W - 2.0 * MARGIN - 128.0 * UNIT) / 2.0;
    let origin_y = MARGIN + (H - 2.0 * MARGIN - 88.0 * UNIT) / 2.0 + 12.0 * UNIT;
    let cx = |ux: f64| origin_x + ux * UNIT;
    let cy = |uy: f64| origin_y + uy * UNIT;

    // One full-bleed 8-unit lattice. It uses the same stroke width as every
    // drawn line in the figure, with lower opacity carrying the hierarchy.
    {
        let ctx = &mut sheet.ctx;
        let u_lo = ((-origin_x / UNIT) / 8.0).floor() as i64 * 8;
        let u_hi = (((W - origin_x) / UNIT) / 8.0).ceil() as i64 * 8;
        let v_lo = ((-origin_y / UNIT) / 8.0).floor() as i64 * 8;
        let v_hi = (((H - origin_y) / UNIT) / 8.0).ceil() as i64 * 8;

        let mut u = u_lo;
        while u <= u_hi {
            ctx.no_fill()
                .stroke(role::grid::subtle())
                .stroke_width(STROKE);
            let x = origin_x + u as f64 * UNIT;
            ctx.line(x, 0.0, x, H);
            u += 8;
        }
        let mut v = v_lo;
        while v <= v_hi {
            ctx.no_fill()
                .stroke(role::grid::subtle())
                .stroke_width(STROKE);
            let y = origin_y + v as f64 * UNIT;
            ctx.line(0.0, y, W, y);
            v += 8;
        }
    }

    // Trace one closed arch band. The inner curve runs in reverse, and each
    // foot uses an 8-unit chamfer, horizontal, chamfer sequence.
    {
        let mut path = BezPath::new();
        path.move_to(l);
        path.curve_to(l_up, t_left, t);
        path.curve_to(t_right, r_up, r);
        path.line_to(right_bottom_outer);
        path.line_to(right_bottom_inner);
        path.line_to(inner_r);
        path.curve_to(inner_r_up, inner_t_right, inner_t);
        path.curve_to(inner_t_left, inner_l_up, inner_l);
        path.line_to(left_bottom_inner);
        path.line_to(left_bottom_outer);
        path.line_to(l);
        path.close_path();
        let to_canvas = Affine::translate((origin_x, origin_y)) * Affine::scale(UNIT);
        sheet
            .ctx
            .fill(role::optical::form_fill())
            .stroke(role::optical::curve())
            .stroke_width(STROKE);
        sheet.ctx.draw_path(to_canvas * path);
    }

    // Draw handles above the opaque form so the construction remains clear.
    for (on, off) in [
        (l, l_up),
        (t, t_left),
        (t, t_right),
        (r, r_up),
        (inner_l, inner_l_up),
        (inner_t, inner_t_left),
        (inner_t, inner_t_right),
        (inner_r, inner_r_up),
    ] {
        sheet
            .ctx
            .no_fill()
            .stroke(role::optical::handles())
            .stroke_width(STROKE);
        sheet.ctx.line(cx(on.0), cy(on.1), cx(off.0), cy(off.1));
    }

    // points, colored by grid level (the apex trio is off 8: red)
    for (p, role) in [
        (l, PtRole::Corner),
        (r, PtRole::Corner),
        (left_bottom_outer, PtRole::Corner),
        (left_bottom_inner, PtRole::Corner),
        (right_bottom_inner, PtRole::Corner),
        (right_bottom_outer, PtRole::Corner),
        (inner_l, PtRole::Corner),
        (inner_r, PtRole::Corner),
        (t, PtRole::Smooth),
        (inner_t, PtRole::Smooth),
        (l_up, PtRole::Off),
        (t_left, PtRole::Off),
        (t_right, PtRole::Off),
        (r_up, PtRole::Off),
        (inner_l_up, PtRole::Off),
        (inner_t_left, PtRole::Off),
        (inner_t_right, PtRole::Off),
        (inner_r_up, PtRole::Off),
    ] {
        let fill = if on8(p.0, p.1) {
            role::optical::structure_point()
        } else {
            role::optical::correction_point()
        };
        let (px, py) = (cx(p.0), cy(p.1));
        sheet
            .ctx
            .fill(fill)
            .stroke(role::optical::pen())
            .stroke_width(STROKE);
        let radius = POINT_SIZE / 2.0;
        match role {
            PtRole::Smooth => {
                sheet
                    .ctx
                    .oval(px - radius, py - radius, POINT_SIZE, POINT_SIZE);
            }
            PtRole::Corner => {
                sheet
                    .ctx
                    .rect(px - radius, py - radius, POINT_SIZE, POINT_SIZE);
            }
            PtRole::Off => {
                sheet
                    .ctx
                    .oval(px - radius, py - radius, POINT_SIZE, POINT_SIZE);
            }
        }
    }

    let outputs = OutputPaths::from_args();
    sheet.save(&renderer, &outputs.blog("fig-optical-correction.png"));
}
