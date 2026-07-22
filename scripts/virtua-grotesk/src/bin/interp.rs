//! fig-interp.png: equal interpolation weights across two endpoint choices.
//!
//! Both rows place the Regular stem at the left endpoint and the Bold stem at
//! the right endpoint. Equal weights therefore share an x coordinate. The
//! 96-to-192 row lands on integer grid values. The 90-to-180 row does not.
//! This compares coordinate arithmetic, not units per em.
//!
//!     cargo run --release --bin interp

use designbot::prelude::Color;
use designbot_render::Renderer;
use virtua_grotesk_figures::*;

const STROKE: f64 = line::HERO;
const NODE_RADIUS: f64 = 24.0;
const AXIS_LEFT: f64 = MARGIN + 100.0;
const AXIS_RIGHT: f64 = W - MARGIN - 100.0;
const UPPER_Y: f64 = 850.0;
const LOWER_Y: f64 = 350.0;

fn weight_x(weight: f64) -> f64 {
    AXIS_LEFT + weight * (AXIS_RIGHT - AXIS_LEFT)
}

fn node(sheet: &mut Sheet, x: f64, y: f64, fill: Color) {
    sheet
        .ctx
        .fill(fill)
        .stroke(role::figure::pen())
        .stroke_width(STROKE);
    sheet.ctx.oval(
        x - NODE_RADIUS,
        y - NODE_RADIUS,
        NODE_RADIUS * 2.0,
        NODE_RADIUS * 2.0,
    );
}

fn axis(sheet: &mut Sheet, y: f64, start: i64, end: i64, step: i64, major: i64) {
    sheet
        .ctx
        .no_fill()
        .stroke(role::figure::pen())
        .stroke_width(STROKE);
    sheet.ctx.line(AXIS_LEFT, y, AXIS_RIGHT, y);

    let mut value = start;
    while value <= end {
        let weight = (value - start) as f64 / (end - start) as f64;
        let x = weight_x(weight);
        let half_height = if value % major == 0 { 38.0 } else { 22.0 };
        sheet.ctx.line(x, y - half_height, x, y + half_height);
        value += step;
    }
}

fn neutral_label(sheet: &mut Sheet, text: &str, x: f64, y: f64, size: f64, align: i8) {
    sheet.label_weighted(text, x, y, size, role::figure::pen(), align, 560.0);
}

fn draw_value(sheet: &mut Sheet, y: f64, weight: f64, value: &str, fill: Color) {
    let x = weight_x(weight);
    node(sheet, x, y, fill);
    neutral_label(sheet, value, x, y + 78.0, 54.0, 0);
    if weight > 0.0 && weight < 1.0 {
        let weight_label = match weight {
            0.25 => "1/4",
            0.5 => "1/2",
            0.75 => "3/4",
            _ => unreachable!("only reviewed interpolation weights are drawn"),
        };
        neutral_label(sheet, weight_label, x, y - 88.0, 50.0, 0);
    }
}

fn main() {
    let mono_path = inputs::geist_mono();
    let mut renderer = Renderer::new(W as u32, H as u32);
    let mono = load_family(&mut renderer, mono_path.to_str().unwrap());
    let mut sheet = new_sheet(&renderer, &mono);
    sheet.ctx.line_cap("round");

    // The guides establish that equal interpolation weights share x. They use
    // the same pen as every other line in the image.
    sheet
        .ctx
        .no_fill()
        .stroke(role::figure::pen())
        .stroke_width(STROKE)
        .line_dash(&[14.0, 18.0]);
    for weight in [0.25, 0.5] {
        let x = weight_x(weight);
        sheet.ctx.line(x, LOWER_Y + 132.0, x, UPPER_Y - 132.0);
    }
    sheet.ctx.line_dash(&[]);

    axis(&mut sheet, UPPER_Y, 96, 192, 2, 8);
    axis(&mut sheet, LOWER_Y, 90, 180, 5, 10);

    // Neutral endpoint nodes belong to the axes. Colored, outlined nodes show
    // the semantic result without assigning color to the number labels.
    draw_value(&mut sheet, UPPER_Y, 0.0, "96", role::figure::point_fill());
    draw_value(&mut sheet, UPPER_Y, 0.25, "120", role::figure::green());
    draw_value(&mut sheet, UPPER_Y, 0.5, "144", role::figure::green());
    draw_value(&mut sheet, UPPER_Y, 0.75, "168", role::figure::green());
    draw_value(&mut sheet, UPPER_Y, 1.0, "192", role::figure::point_fill());

    draw_value(&mut sheet, LOWER_Y, 0.0, "90", role::figure::point_fill());
    draw_value(&mut sheet, LOWER_Y, 0.25, "112.5", role::figure::red());
    draw_value(&mut sheet, LOWER_Y, 0.5, "135", role::figure::red());
    draw_value(&mut sheet, LOWER_Y, 1.0, "180", role::figure::point_fill());

    let outputs = OutputPaths::from_args();
    sheet.save(&renderer, &outputs.blog("fig-interp.png"));
}
