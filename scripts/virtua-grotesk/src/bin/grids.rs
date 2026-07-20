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

const GAP: f64 = 96.0;
const SLOT: f64 = (W - 2.0 * MARGIN - GAP) / 2.0; // 1116

// The crop, in font units, and its scale on the canvas.
const UX0: f64 = 64.0;
const UX1: f64 = 464.0;
const UY0: f64 = -16.0;
const UY1: f64 = 304.0;
const S: f64 = 2.87; // canvas px per font unit -> 1148 x 918.4 panel
const PANEL_BOTTOM: f64 = 168.0;
const PANEL_TOP: f64 = PANEL_BOTTOM + (UY1 - UY0) * S;
const INSET_X: f64 = (SLOT - (UX1 - UX0) * S) / 2.0;

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
    color::gray_600()
}
fn curve() -> Color {
    role::figure::pen()
}
/// Panel-local transform: font units -> canvas.
fn cx(panel_left: f64, ux: f64) -> f64 {
    panel_left + (ux - UX0) * S
}
fn cy(uy: f64) -> f64 {
    PANEL_BOTTOM + (uy - UY0) * S
}

fn on8(v: f64) -> bool {
    v.rem_euclid(8.0) == 0.0
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

    let panel_left = |i: usize| MARGIN + i as f64 * (SLOT + GAP) + INSET_X;
    let panel_w = (UX1 - UX0) * S;

    // ── grids, per panel ──
    for i in 0..2 {
        let pl = panel_left(i);
        let draw_lattice = |step: f64, color: Color, width: f64, ctx: &mut Canvas| {
            ctx.no_fill().stroke(color).stroke_width(width);
            let mut ux = UX0;
            while ux <= UX1 {
                ctx.line(cx(pl, ux), cy(UY0), cx(pl, ux), cy(UY1));
                ux += step;
            }
            let mut uy = (UY0 / step).ceil() * step;
            while uy <= UY1 {
                ctx.line(cx(pl, UX0), cy(uy), cx(pl, UX1), cy(uy));
                uy += step;
            }
        };
        if i == 0 {
            // flat: one lattice, one color
            draw_lattice(8.0, grid_flat(), line::THIN, &mut sheet.ctx);
        } else {
            // nested: three levels at three intensities
            draw_lattice(2.0, grid_2(), line::HAIRLINE, &mut sheet.ctx);
            draw_lattice(8.0, grid_8(), line::THIN, &mut sheet.ctx);
            draw_lattice(64.0, grid_64(), line::REGULAR, &mut sheet.ctx);
        }
        // baseline, house blue
        sheet.ctx.no_fill().stroke(blue()).stroke_width(line::HERO);
        sheet.ctx.line(cx(pl, UX0), cy(0.0), cx(pl, UX1), cy(0.0));
    }

    // ── the outline, identical in both panels ──
    for i in 0..2 {
        let pl = panel_left(i);
        let place = Affine::new([S, 0.0, 0.0, S, pl - UX0 * S, PANEL_BOTTOM - UY0 * S]);
        // The glyph is deliberately identical in both panels; only the grid
        // changes. One shared fill prevents color from implying a second
        // variable and keeps crop spill visually continuous.
        let fill = role::figure::orange();
        sheet
            .ctx
            .fill(fill)
            .stroke(curve())
            .stroke_width(line::HERO);
        sheet.ctx.draw_path(place * outline.path.clone());
    }

    // ── mask the crop spill (no clip API; bg is solid) ──
    {
        let ctx = &mut sheet.ctx;
        ctx.fill(role::canvas::background()).no_stroke();
        ctx.rect(0.0, 0.0, W, PANEL_BOTTOM); // below
        ctx.rect(0.0, PANEL_TOP, W, H - PANEL_TOP); // above
        ctx.rect(0.0, 0.0, panel_left(0), H); // left of panel 01
        let gap_x0 = panel_left(0) + panel_w;
        ctx.rect(gap_x0, 0.0, panel_left(1) - gap_x0, H); // between panels
        let right_x0 = panel_left(1) + panel_w;
        ctx.rect(right_x0, 0.0, W - right_x0, H); // right of panel 02
    }

    // ── panel borders ──
    for i in 0..2 {
        let pl = panel_left(i);
        sheet
            .ctx
            .no_fill()
            .stroke(border())
            .stroke_width(line::HERO);
        sheet
            .ctx
            .rect(pl, PANEL_BOTTOM, panel_w, PANEL_TOP - PANEL_BOTTOM);
    }

    // ── handles + point markers, colored by panel scheme ──
    let in_window = |x: f64, y: f64| {
        (UX0 - 4.0..=UX1 + 4.0).contains(&x) && (UY0 - 4.0..=UY1 + 4.0).contains(&y)
    };
    for i in 0..2 {
        let pl = panel_left(i);
        // handle lines
        for ((x1, y1), (x2, y2)) in &outline.handles {
            if !(in_window(*x1, *y1) && in_window(*x2, *y2)) {
                continue;
            }
            sheet
                .ctx
                .no_fill()
                .stroke(role::figure::pen())
                .stroke_width(line::HERO);
            sheet.ctx.line(cx(pl, *x1), cy(*y1), cx(pl, *x2), cy(*y2));
        }
        // markers, knocked out with the background color
        for (x, y, role) in &outline.points {
            if !in_window(*x, *y) {
                continue;
            }
            let correction = !on8(*x) || !on8(*y);
            let fill = if i == 1 && correction {
                role::figure::correction_point_fill()
            } else {
                role::figure::point_fill()
            };
            sheet
                .ctx
                .fill(fill)
                .stroke(role::figure::pen())
                .stroke_width(line::HERO);
            let (px, py) = (cx(pl, *x), cy(*y));
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
    }

    write_png(&renderer, &sheet.ctx, &out);
}
