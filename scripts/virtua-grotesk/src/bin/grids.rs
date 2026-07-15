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
    Color::rgb(0x30, 0x30, 0x30)
}
fn grid_2() -> Color {
    Color::rgb(0x1a, 0x1a, 0x1a)
}
fn grid_8() -> Color {
    Color::rgb(0x2c, 0x2c, 0x2c)
}
fn grid_64() -> Color {
    Color::rgb(0x42, 0x42, 0x42)
}
fn border() -> Color {
    Color::rgb(0x4a, 0x4a, 0x4a)
}
fn curve() -> Color {
    Color::rgb(230, 230, 230)
}
fn curve_fill() -> Color {
    Color::rgba(230, 230, 230, 20)
}
fn crop_handle() -> Color {
    Color::rgb(0x6a, 0x6a, 0x6a)
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
    let home = std::env::var("HOME").unwrap();
    let mono_path = format!("{home}/GH/repos/google-fonts/ofl/geistmono/GeistMono[wght].ttf");
    let glyphs_dir = std::path::PathBuf::from(&home)
        .join("GH/repos/virtua-grotesk/sources/VirtuaGrotesk-Regular.ufo/glyphs");

    let mut renderer = Renderer::new(W as u32, H as u32);
    let mono = load_family(&mut renderer, &mono_path);
    let outline = load_outline(&glyphs_dir, "a");

    let here = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let out = here
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("src/content/blog/virtua-grotesk/fig-grid-labels.png");

    let mut sheet = Sheet {
        ctx: Canvas::new(W, H),
        renderer: &renderer,
        mono: mono.clone(),
    };
    sheet.ctx.background(bg());

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
            draw_lattice(8.0, grid_flat(), 2.0, &mut sheet.ctx);
        } else {
            // nested: three levels at three intensities
            draw_lattice(2.0, grid_2(), 1.0, &mut sheet.ctx);
            draw_lattice(8.0, grid_8(), 2.0, &mut sheet.ctx);
            draw_lattice(64.0, grid_64(), 2.5, &mut sheet.ctx);
        }
        // baseline, house blue
        sheet.ctx.no_fill().stroke(blue()).stroke_width(PEN);
        sheet.ctx.line(cx(pl, UX0), cy(0.0), cx(pl, UX1), cy(0.0));
    }

    // ── the outline, identical in both panels ──
    for i in 0..2 {
        let pl = panel_left(i);
        let place = Affine::new([S, 0.0, 0.0, S, pl - UX0 * S, PANEL_BOTTOM - UY0 * S]);
        sheet.ctx.fill(curve_fill()).stroke(curve()).stroke_width(PEN);
        sheet.ctx.draw_path(place * outline.path.clone());
    }

    // ── mask the crop spill (no clip API; bg is solid) ──
    {
        let ctx = &mut sheet.ctx;
        ctx.fill(bg()).no_stroke();
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
        sheet.ctx.no_fill().stroke(border()).stroke_width(PEN);
        sheet.ctx.rect(pl, PANEL_BOTTOM, panel_w, PANEL_TOP - PANEL_BOTTOM);
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
            let correction = !on8(*x2) || !on8(*y2);
            let color = if i == 0 {
                crop_handle()
            } else if correction {
                red()
            } else {
                crop_handle()
            };
            sheet.ctx.no_fill().stroke(color).stroke_width(PEN);
            sheet.ctx.line(cx(pl, *x1), cy(*y1), cx(pl, *x2), cy(*y2));
        }
        // markers, knocked out with the background color
        for (x, y, role) in &outline.points {
            if !in_window(*x, *y) {
                continue;
            }
            let correction = !on8(*x) || !on8(*y);
            let color = if i == 0 {
                gray()
            } else if correction {
                red()
            } else {
                green()
            };
            sheet.ctx.fill(bg()).stroke(color).stroke_width(PEN);
            let (px, py) = (cx(pl, *x), cy(*y));
            match role {
                PtRole::Smooth => {
                    sheet.ctx.oval(px - 9.0, py - 9.0, 18.0, 18.0);
                }
                PtRole::Corner => {
                    sheet.ctx.rect(px - 8.0, py - 8.0, 16.0, 16.0);
                }
                PtRole::Off => {
                    sheet.ctx.oval(px - 7.0, py - 7.0, 14.0, 14.0);
                }
            }
        }
    }

    // ── callouts: leader from the text to the x=116 anchor ──
    for i in 0..2 {
        let pl = panel_left(i);
        let anchor = (cx(pl, 116.0), cy(160.0));
        let text_x = cx(pl, 250.0);
        let text_y = cy(160.0) - 10.0;
        let color = if i == 0 { gray() } else { red() };
        sheet.ctx.no_fill().stroke(color).stroke_width(PEN);
        sheet.ctx.line(anchor.0 + 16.0, anchor.1, text_x - 14.0, text_y + 8.0);
        let (l1, l2) = if i == 0 {
            ("x=116: off grid", "correction or mistake?")
        } else {
            ("x=116: on 2, off 8", "optical correction")
        };
        sheet.label_padded(l1, text_x, text_y + 20.0, 26.0, color, -1);
        sheet.label_padded(l2, text_x, text_y - 22.0, 26.0, color, -1);
    }

    // ── baseline tags ──
    for i in 0..2 {
        let pl = panel_left(i);
        sheet.label_padded("baseline 0", pl + 14.0, cy(0.0) + 12.0, 26.0, blue(), -1);
    }

    // ── panel titles + legends ──
    let title_y = PANEL_TOP + 44.0;
    sheet.label("01 flat grid / one level", panel_left(0), title_y, 30.0, green(), -1);
    sheet.label(
        "all points equally legal",
        panel_left(0) + panel_w,
        title_y,
        26.0,
        gray(),
        1,
    );
    sheet.label(
        "02 nested grid / 64 \u{b7} 8 \u{b7} 2",
        panel_left(1),
        title_y,
        30.0,
        green(),
        -1,
    );
    {
        // right-aligned legend: [green oval] ON 8   [red oval] ON 2, OFF 8
        let size = 26.0;
        let t2 = "on 2, off 8";
        let t1 = "on 8";
        let w2 = sheet.mono_width(t2, size);
        let w1 = sheet.mono_width(t1, size);
        let right = panel_left(1) + panel_w;
        let x2 = right - w2;
        let dot2 = x2 - 26.0;
        let x1 = dot2 - 36.0 - w1;
        let dot1 = x1 - 26.0;
        sheet.label(t2, x2, title_y, size, red(), -1);
        sheet.label(t1, x1, title_y, size, green(), -1);
        sheet.ctx.fill(bg()).stroke(red()).stroke_width(PEN);
        sheet.ctx.oval(dot2, title_y + 1.0, 16.0, 16.0);
        sheet.ctx.fill(bg()).stroke(green()).stroke_width(PEN);
        sheet.ctx.oval(dot1, title_y + 1.0, 16.0, 16.0);
    }

    sheet.hud_title(&[
        "Grid as labeling function",
        "same outline both panels; only the nested grid labels the corrections",
    ]);
    sheet.attribution(None);

    std::fs::create_dir_all(out.parent().unwrap()).unwrap();
    renderer
        .render_to_png(&sheet.ctx, out.to_str().unwrap())
        .unwrap();
    println!("wrote {}", out.display());
}
