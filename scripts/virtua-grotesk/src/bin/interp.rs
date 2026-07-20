//! fig-interp.png : interpolation between two grid masters stays on the
//! grid on a power-of-two em, and leaves it on a decimal em.
//!
//! Both number lines put the Regular stem (96 / 90) at the left and the
//! Bold stem (192 / 180) at the right, so a given interpolation weight
//! sits at the SAME x on both. On 1024 every dyadic weight lands on a
//! grid tick; on 1000 the halves land between ticks and the quarters
//! stop being integers at all.
//!
//!     cargo run --release --bin interp

use designbot_render::Renderer;
#[allow(unused_imports)]
use virtua_grotesk_figures::*;

// weight (0..1) -> x, shared by both lines so equal weights align
fn xt(t: f64, x_l: f64, x_r: f64) -> f64 {
    x_l + t * (x_r - x_l)
}

fn axis_node(sheet: &mut Sheet, x: f64, y: f64, r: f64) {
    sheet
        .ctx
        .fill(role::figure::point_fill())
        .stroke(role::figure::pen())
        .stroke_width(8.0);
    sheet.ctx.oval(x - r, y - r, r * 2.0, r * 2.0);
}

fn main() {
    let mono_path = inputs::geist_mono();
    let mut renderer = Renderer::new(W as u32, H as u32);
    let mono = load_family(&mut renderer, mono_path.to_str().unwrap());
    let mut sheet = new_sheet(&renderer, &mono);

    // Leave a real optical gutter for the large em labels at each end.
    let x_l = MARGIN + 300.0;
    let x_r = W - MARGIN - 300.0;
    let tick_h = 38.0;
    let minor = color::gray_800();
    let major = color::gray_475();

    // ---- line 1: em 1024, stem 96 -> 192, grid every 2 (major every 8) ----
    let y1 = 980.0;
    let (r1, b1) = (96.0, 192.0);
    sheet
        .ctx
        .no_fill()
        .stroke(role::figure::pen())
        .stroke_width(10.0);
    sheet.ctx.line(x_l - 40.0, y1, x_r + 40.0, y1);
    let mut v = r1;
    while v <= b1 + 0.01 {
        let x = xt((v - r1) / (b1 - r1), x_l, x_r);
        let is8 = (v as i64) % 8 == 0;
        sheet
            .ctx
            .no_fill()
            .stroke(if is8 { major } else { minor })
            .stroke_width(if is8 { 9.0 } else { line::HERO });
        let h = if is8 { tick_h } else { tick_h * 0.55 };
        sheet.ctx.line(x, y1 - h, x, y1 + h);
        v += 2.0;
    }
    sheet.label("1024", MARGIN, y1 - 24.0, 78.0, role::figure::pen(), -1);
    // endpoints
    sheet.label_padded("96", x_l, y1 - 84.0, 64.0, green(), 0);
    sheet.label_padded("192", x_r, y1 - 84.0, 64.0, green(), 0);
    axis_node(&mut sheet, x_l, y1, 22.0);
    axis_node(&mut sheet, x_r, y1, 22.0);
    // Three large featured interpolations make the exact landings legible at
    // social-card size; the smaller ticks still show the complete 2-unit grid.
    let pts1 = [("1/2", 0.5, 144), ("1/4", 0.25, 120), ("3/4", 0.75, 168)];
    for (w, t, val) in pts1 {
        let x = xt(t, x_l, x_r);
        sheet.ctx.fill(green()).no_stroke();
        sheet.ctx.oval(x - 22.0, y1 - 22.0, 44.0, 44.0);
        sheet.label(w, x, y1 - 88.0, 58.0, role::annotation::dimensions(), 0);
        sheet.label(&val.to_string(), x, y1 + 76.0, 58.0, green(), 0);
    }

    // ---- line 2: em 1000, stem 90 -> 180, grid every 10 ----
    let y2 = 330.0;
    let (r2, b2) = (90.0, 180.0);
    sheet
        .ctx
        .no_fill()
        .stroke(role::figure::pen())
        .stroke_width(10.0);
    sheet.ctx.line(x_l - 40.0, y2, x_r + 40.0, y2);
    let mut v = r2;
    while v <= b2 + 0.01 {
        let x = xt((v - r2) / (b2 - r2), x_l, x_r);
        sheet.ctx.no_fill().stroke(major).stroke_width(9.0);
        sheet.ctx.line(x, y2 - tick_h, x, y2 + tick_h);
        v += 10.0;
    }
    sheet.label("1000", MARGIN, y2 - 24.0, 78.0, role::figure::pen(), -1);
    sheet.label_padded("90", x_l, y2 - 84.0, 64.0, red(), 0);
    sheet.label_padded("180", x_r, y2 - 84.0, 64.0, red(), 0);
    axis_node(&mut sheet, x_l, y2, 22.0);
    axis_node(&mut sheet, x_r, y2, 22.0);
    // same weights, now off the grid
    let x_half = xt(0.5, x_l, x_r);
    sheet.ctx.fill(red()).no_stroke();
    sheet.ctx.oval(x_half - 22.0, y2 - 22.0, 44.0, 44.0);
    sheet.label(
        "1/2",
        x_half,
        y2 - 88.0,
        58.0,
        role::annotation::dimensions(),
        0,
    );
    sheet.label("135", x_half, y2 + 76.0, 58.0, red(), 0);

    let x_q = xt(0.25, x_l, x_r);
    sheet.ctx.fill(red()).no_stroke();
    sheet.ctx.oval(x_q - 22.0, y2 - 22.0, 44.0, 44.0);
    sheet.label(
        "1/4",
        x_q,
        y2 - 88.0,
        58.0,
        role::annotation::dimensions(),
        0,
    );
    sheet.label("112.5", x_q, y2 + 76.0, 58.0, red(), 0);

    // alignment guides: the SAME weight, landing on a tick vs between ticks
    for t in [0.5f64, 0.25] {
        let x = xt(t, x_l, x_r);
        let mut yy = y2 + 40.0;
        sheet
            .ctx
            .no_fill()
            .stroke(role::chart::axis())
            .stroke_width(line::REGULAR);
        while yy < y1 - 40.0 {
            sheet.ctx.line(x, yy, x, (yy + 14.0).min(y1 - 40.0));
            yy += 26.0;
        }
    }

    let outputs = OutputPaths::from_args();
    sheet.save(&renderer, &outputs.blog("fig-interp.png"));
}
