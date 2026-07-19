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

fn main() {
    let mono_path = inputs::geist_mono();
    let mut renderer = Renderer::new(W as u32, H as u32);
    let mono = load_family(&mut renderer, mono_path.to_str().unwrap());
    let mut sheet = new_sheet(&renderer, &mono);

    let x_l = MARGIN + 180.0;
    let x_r = W - MARGIN - 180.0;
    let tick_h = 28.0;
    let minor = color::gray_800();
    let major = color::gray_475();

    // ---- line 1: em 1024, stem 96 -> 192, grid every 2 (major every 8) ----
    let y1 = 900.0;
    let (r1, b1) = (96.0, 192.0);
    sheet
        .ctx
        .no_fill()
        .stroke(role::figure::pen())
        .stroke_width(line::HERO);
    sheet.ctx.line(x_l - 40.0, y1, x_r + 40.0, y1);
    let mut v = r1;
    while v <= b1 + 0.01 {
        let x = xt((v - r1) / (b1 - r1), x_l, x_r);
        let is8 = (v as i64) % 8 == 0;
        sheet
            .ctx
            .no_fill()
            .stroke(if is8 { major } else { minor })
            .stroke_width(if is8 { line::HERO } else { line::REGULAR });
        let h = if is8 { tick_h } else { tick_h * 0.55 };
        sheet.ctx.line(x, y1 - h, x, y1 + h);
        v += 2.0;
    }
    sheet.label(
        "1024",
        MARGIN,
        y1 - 14.0,
        type_size::XXXL,
        role::figure::pen(),
        -1,
    );
    // endpoints
    sheet.label_padded("96", x_l, y1 - 58.0, type_size::XXL, green(), 0);
    sheet.label_padded("192", x_r, y1 - 58.0, type_size::XXL, green(), 0);
    node(&mut sheet, x_l, y1, 13.0);
    node(&mut sheet, x_r, y1, 13.0);
    // dyadic interpolations, all landing on a tick
    let pts1 = [
        ("1/2", 0.5, 144),
        ("1/4", 0.25, 120),
        ("3/4", 0.75, 168),
        ("1/8", 0.125, 108),
        ("3/8", 0.375, 132),
        ("5/8", 0.625, 156),
        ("7/8", 0.875, 180),
    ];
    for (w, t, val) in pts1 {
        let x = xt(t, x_l, x_r);
        sheet.ctx.fill(green()).no_stroke();
        sheet.ctx.oval(x - 13.0, y1 - 13.0, 26.0, 26.0);
        sheet.label(
            w,
            x,
            y1 - 58.0,
            type_size::XL,
            role::annotation::dimensions(),
            0,
        );
        sheet.label(&val.to_string(), x, y1 + 58.0, type_size::XL, green(), 0);
    }

    // ---- line 2: em 1000, stem 90 -> 180, grid every 10 ----
    let y2 = 400.0;
    let (r2, b2) = (90.0, 180.0);
    sheet
        .ctx
        .no_fill()
        .stroke(role::figure::pen())
        .stroke_width(line::HERO);
    sheet.ctx.line(x_l - 40.0, y2, x_r + 40.0, y2);
    let mut v = r2;
    while v <= b2 + 0.01 {
        let x = xt((v - r2) / (b2 - r2), x_l, x_r);
        sheet.ctx.no_fill().stroke(major).stroke_width(line::HERO);
        sheet.ctx.line(x, y2 - tick_h, x, y2 + tick_h);
        v += 10.0;
    }
    sheet.label(
        "1000",
        MARGIN,
        y2 - 14.0,
        type_size::XXXL,
        role::figure::pen(),
        -1,
    );
    sheet.label_padded("90", x_l, y2 - 58.0, type_size::XXL, red(), 0);
    sheet.label_padded("180", x_r, y2 - 58.0, type_size::XXL, red(), 0);
    node(&mut sheet, x_l, y2, 13.0);
    node(&mut sheet, x_r, y2, 13.0);
    // same weights, now off the grid
    let x_half = xt(0.5, x_l, x_r);
    sheet.ctx.fill(red()).no_stroke();
    sheet.ctx.oval(x_half - 15.0, y2 - 15.0, 30.0, 30.0);
    sheet.label(
        "1/2",
        x_half,
        y2 - 58.0,
        type_size::XL,
        role::annotation::dimensions(),
        0,
    );
    sheet.label("135", x_half, y2 + 58.0, type_size::XL, red(), 0);

    let x_q = xt(0.25, x_l, x_r);
    sheet.ctx.fill(red()).no_stroke();
    sheet.ctx.oval(x_q - 15.0, y2 - 15.0, 30.0, 30.0);
    sheet.label(
        "1/4",
        x_q,
        y2 - 58.0,
        type_size::XL,
        role::annotation::dimensions(),
        0,
    );
    sheet.label("112.5", x_q, y2 + 58.0, type_size::XL, red(), 0);

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
