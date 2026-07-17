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

use designbot::prelude::*;
use designbot_render::Renderer;
#[allow(unused_imports)]
use virtua_grotesk_figures::*;

// weight (0..1) -> x, shared by both lines so equal weights align
fn xt(t: f64, x_l: f64, x_r: f64) -> f64 {
    x_l + t * (x_r - x_l)
}

fn main() {
    let home = std::env::var("HOME").unwrap();
    let mono_path = format!("{home}/GH/repos/google-fonts/ofl/geistmono/GeistMono[wght].ttf");
    let mut renderer = Renderer::new(W as u32, H as u32);
    let mono = load_family(&mut renderer, &mono_path);
    let mut sheet = new_sheet(&renderer, &mono);

    let x_l = MARGIN + 250.0;
    let x_r = W - MARGIN - 250.0;
    let tick_h = 18.0;
    let minor = Color::rgb(0x30, 0x30, 0x30);
    let major = Color::rgb(0x66, 0x66, 0x66);

    // ---- line 1: em 1024, stem 96 -> 192, grid every 2 (major every 8) ----
    let y1 = H - MARGIN - 470.0;
    let (r1, b1) = (96.0, 192.0);
    sheet.ctx.no_fill().stroke(major).stroke_width(PEN);
    sheet.ctx.line(x_l - 40.0, y1, x_r + 40.0, y1);
    let mut v = r1;
    while v <= b1 + 0.01 {
        let x = xt((v - r1) / (b1 - r1), x_l, x_r);
        let is8 = (v as i64) % 8 == 0;
        sheet.ctx.no_fill().stroke(if is8 { major } else { minor }).stroke_width(PEN_LIGHT);
        let h = if is8 { tick_h } else { tick_h * 0.55 };
        sheet.ctx.line(x, y1 - h, x, y1 + h);
        v += 2.0;
    }
    sheet.label("em 1024 / grid 2", x_l - 40.0, y1 + 92.0, LABEL_TEXT, green(), -1);
    // endpoints
    sheet.label_padded("Regular 96", x_l, y1 - 40.0, DIM_TEXT, green(), 0);
    sheet.label_padded("Bold 192", x_r, y1 - 40.0, DIM_TEXT, red(), 0);
    node(&mut sheet, x_l, y1, 9.0);
    node(&mut sheet, x_r, y1, 9.0);
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
        sheet.ctx.oval(x - 8.0, y1 - 8.0, 16.0, 16.0);
        sheet.label(w, x, y1 - 44.0, SMALL_TEXT, dim_color(), 0);
        sheet.label(&val.to_string(), x, y1 + 52.0, SMALL_TEXT, green(), 0);
    }

    // ---- line 2: em 1000, stem 90 -> 180, grid every 10 ----
    let y2 = MARGIN + 360.0;
    let (r2, b2) = (90.0, 180.0);
    sheet.ctx.no_fill().stroke(major).stroke_width(PEN);
    sheet.ctx.line(x_l - 40.0, y2, x_r + 40.0, y2);
    let mut v = r2;
    while v <= b2 + 0.01 {
        let x = xt((v - r2) / (b2 - r2), x_l, x_r);
        sheet.ctx.no_fill().stroke(major).stroke_width(PEN_LIGHT);
        sheet.ctx.line(x, y2 - tick_h, x, y2 + tick_h);
        v += 10.0;
    }
    sheet.label("em 1000 / grid 10", x_l - 40.0, y2 + 92.0, LABEL_TEXT, red(), -1);
    sheet.label_padded("Regular 90", x_l, y2 - 40.0, DIM_TEXT, gray(), 0);
    sheet.label_padded("Bold 180", x_r, y2 - 40.0, DIM_TEXT, gray(), 0);
    node(&mut sheet, x_l, y2, 9.0);
    node(&mut sheet, x_r, y2, 9.0);
    // same weights, now off the grid
    let x_half = xt(0.5, x_l, x_r);
    sheet.ctx.fill(red()).no_stroke();
    sheet.ctx.oval(x_half - 8.0, y2 - 8.0, 16.0, 16.0);
    sheet.label("1/2", x_half, y2 - 44.0, SMALL_TEXT, dim_color(), 0);
    sheet.label("135", x_half, y2 + 52.0, SMALL_TEXT, red(), 0);
    sheet.label("off grid", x_half, y2 + 84.0, SMALL_TEXT, red(), 0);

    let x_q = xt(0.25, x_l, x_r);
    sheet.ctx.fill(red()).no_stroke();
    sheet.ctx.oval(x_q - 8.0, y2 - 8.0, 16.0, 16.0);
    sheet.label("1/4", x_q, y2 - 44.0, SMALL_TEXT, dim_color(), 0);
    sheet.label("112.5", x_q, y2 + 52.0, SMALL_TEXT, red(), 0);
    sheet.label("not integer", x_q, y2 + 84.0, SMALL_TEXT, red(), 0);

    // alignment guides: the SAME weight, landing on a tick vs between ticks
    for t in [0.5f64, 0.25] {
        let x = xt(t, x_l, x_r);
        let mut yy = y2 + 40.0;
        sheet.ctx.no_fill().stroke(Color::rgb(0x3a, 0x3a, 0x3a)).stroke_width(PEN_LIGHT);
        while yy < y1 - 40.0 {
            sheet.ctx.line(x, yy, x, (yy + 14.0).min(y1 - 40.0));
            yy += 26.0;
        }
    }

    sheet.label(
        "same weight, exact grid value on 1024, off the grid on 1000",
        W / 2.0,
        (y1 + y2) / 2.0,
        LABEL_TEXT,
        gray(),
        0,
    );

    sheet.hud_title(&[
        "Interpolation stays on the grid",
        "the n stem, Regular to Bold, sampled at binary-fraction weights",
    ]);
    sheet.attribution(Some("Virtua Grotesk / stem width in font units"));

    let here = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let post = here
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("src/content/blog/virtua-grotesk");
    sheet.save(&renderer, &post.join("fig-interp.png"));
}
