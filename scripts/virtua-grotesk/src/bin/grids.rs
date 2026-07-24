//! Flat-vs-nested grid comparison for the Virtua Grotesk post, §10.
//!
//! Two panels, both showing the SAME zoomed crop of the lowercase a's lower
//! bowl from the Regular UFO — the counter whose left extremum is pulled to
//! x=116, on the 2-unit grid but off the 8-unit grid (three points: one
//! smooth anchor, two off-curve handles).
//!
//!   Panel 01, flat grid (Replica's scheme): one lattice, one color; every
//!   point renders identical gray, and the off-grid cluster is tagged
//!   "correction or mistake?" — the data can't say.
//!
//!   Panel 02, nested grid (Virtua): the 2 / 8 / 64 levels drawn at three
//!   intensities; on-8 points green (machine structure), the x=116 cluster
//!   red (the hand) — the level a point sits on labels the decision.
//!
//! Run from this directory:
//!
//!     cargo run --release --bin grids
//!
//! Writes ../../src/content/blog/virtua-grotesk/fig-grid-labels.png.
//!
//! Inputs read at render time, from sibling checkouts:
//!     ~/GH/repos/virtua-grotesk/sources/VirtuaGrotesk-Regular.ufo
//!     ~/GH/repos/google-fonts/ofl/geistmono/GeistMono[wght].ttf

use designbot::prelude::*;
use designbot_render::Renderer;
use kurbo::Affine;
#[allow(unused_imports)]
use virtua_grotesk_figures::*;

// The crop, in font units, and its scale on the canvas.
const UX0: f64 = 64.0;
const UX1: f64 = 464.0;
const UY0: f64 = -16.0;
const UY1: f64 = 304.0;
const S: f64 = 2.875; // canvas px per font unit -> 1150 x 920 panel
const PANEL_W: f64 = (UX1 - UX0) * S;
const PANEL_H: f64 = (UY1 - UY0) * S;
const PANEL_GAP: f64 = (W - 2.0 * PANEL_W) / 3.0;
const PANEL_BOTTOM: f64 = (H - PANEL_H) / 2.0;

// Theme tokens, shared with og.rs / figs.rs.
fn grid_flat() -> Color {
    role::grid::flat()
}
fn grid_2() -> Color {
    role::grid::faint()
}
fn grid_8() -> Color {
    role::grid::structure()
}
fn grid_64() -> Color {
    role::grid::major()
}
fn border() -> Color {
    role::figure::pen()
}
fn curve() -> Color {
    role::figure::pen()
}
/// Panel-local transform: font units -> rasterized panel.
fn cx(ux: f64) -> f64 {
    (ux - UX0) * S
}
fn cy(uy: f64) -> f64 {
    (uy - UY0) * S
}

fn on8(v: f64) -> bool {
    v.rem_euclid(8.0) == 0.0
}

fn draw_panel(outline: &Outline, nested: bool) -> Canvas {
    let mut ctx = Canvas::new(PANEL_W, PANEL_H);
    ctx.background(role::canvas::background());
    ctx.line_cap("round");

    let draw_lattice = |step: f64, color: Color, width: f64, ctx: &mut Canvas| {
        ctx.no_fill().stroke(color).stroke_width(width);
        let mut ux = UX0;
        while ux <= UX1 {
            ctx.line(cx(ux), cy(UY0), cx(ux), cy(UY1));
            ux += step;
        }
        let mut uy = (UY0 / step).ceil() * step;
        while uy <= UY1 {
            ctx.line(cx(UX0), cy(uy), cx(UX1), cy(uy));
            uy += step;
        }
    };
    if nested {
        draw_lattice(2.0, grid_2(), line::HAIRLINE, &mut ctx);
        draw_lattice(8.0, grid_8(), line::THIN, &mut ctx);
        draw_lattice(64.0, grid_64(), line::REGULAR, &mut ctx);
    } else {
        draw_lattice(8.0, grid_flat(), line::MEDIUM, &mut ctx);
    }

    // The baseline belongs to the same one-pen construction system.
    ctx.no_fill().stroke(role::figure::pen()).stroke_width(8.0);
    ctx.line(cx(UX0), cy(0.0), cx(UX1), cy(0.0));

    let place = Affine::new([S, 0.0, 0.0, S, -UX0 * S, -UY0 * S]);
    ctx.fill(role::figure::yellow())
        .stroke(curve())
        .stroke_width(10.0);
    ctx.draw_path(place * outline.path.clone());

    ctx
}

fn main() {
    let mono_path = inputs::geist_mono();
    let glyphs_dir = inputs::virtua_sources().join("VirtuaGrotesk-Regular.ufo/glyphs");

    let mut renderer = Renderer::new(W as u32, H as u32);
    let mono = load_family(&mut renderer, mono_path.to_str().unwrap());
    let outline = load_outline(&glyphs_dir, "a");

    let outputs = OutputPaths::from_args();
    let out = outputs.blog("fig-grid-labels.png");

    let mut sheet = Sheet {
        ctx: Canvas::new(W, H),
        renderer: &renderer,
        mono: mono.clone(),
    };
    sheet.ctx.background(role::canvas::background());
    sheet.ctx.line_cap("round");

    let panel_left = |i: usize| PANEL_GAP + i as f64 * (PANEL_W + PANEL_GAP);
    let panel_renderer = Renderer::new(PANEL_W as u32, PANEL_H as u32);

    // Rasterize each panel independently. This is a real crop: glyph geometry
    // outside one panel cannot spill into its neighbor.
    for i in 0..2 {
        let panel = draw_panel(&outline, i == 1);
        let rgba = panel_renderer.render_frames(&panel).remove(0).0;
        sheet.ctx.image_rgba(
            rgba,
            PANEL_W as u32,
            PANEL_H as u32,
            panel_left(i),
            PANEL_BOTTOM,
            1.0,
        );
    }

    // ── panel borders ──
    for i in 0..2 {
        let pl = panel_left(i);
        sheet.ctx.no_fill().stroke(border()).stroke_width(10.0);
        sheet.ctx.rect(pl, PANEL_BOTTOM, PANEL_W, PANEL_H);
    }

    // Handles and markers sit above the crop. This keeps edge points whole
    // while the filled glyph remains isolated inside its own panel.
    let in_window = |x: f64, y: f64| {
        (UX0 - 4.0..=UX1 + 4.0).contains(&x) && (UY0 - 4.0..=UY1 + 4.0).contains(&y)
    };
    for i in 0..2 {
        let pl = panel_left(i);
        for ((x1, y1), (x2, y2)) in &outline.handles {
            if !(in_window(*x1, *y1) && in_window(*x2, *y2)) {
                continue;
            }
            sheet
                .ctx
                .no_fill()
                .stroke(role::figure::pen())
                .stroke_width(10.0);
            sheet.ctx.line(
                pl + cx(*x1),
                PANEL_BOTTOM + cy(*y1),
                pl + cx(*x2),
                PANEL_BOTTOM + cy(*y2),
            );
        }
        for (x, y, point_role) in &outline.points {
            if !in_window(*x, *y) {
                continue;
            }
            let correction = !on8(*x) || !on8(*y);
            let fill = if i == 0 {
                role::figure::point_fill()
            } else if correction {
                role::figure::red()
            } else {
                role::figure::green()
            };
            sheet
                .ctx
                .fill(fill)
                .stroke(role::figure::pen())
                .stroke_width(9.0);
            let (px, py) = (pl + cx(*x), PANEL_BOTTOM + cy(*y));
            match point_role {
                PtRole::Smooth | PtRole::Off => {
                    sheet.ctx.oval(px - 18.0, py - 18.0, 36.0, 36.0);
                }
                PtRole::Corner => {
                    sheet.ctx.rect(px - 18.0, py - 18.0, 36.0, 36.0);
                }
            }
        }
    }

    write_png(&renderer, &sheet.ctx, &out);
}
