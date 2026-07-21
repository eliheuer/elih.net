//! Four graphic proofs for the powers-of-two section. Each composition keeps
//! one visual argument and omits prose already present in the article.

use designbot::prelude::*;
use designbot_render::Renderer;
use virtua_grotesk_figures::*;

fn bit_row(sheet: &mut Sheet, x0: f64, y0: f64, cell: f64, gap: f64, bits: &[u8], color: Color) {
    for (i, bit) in bits.iter().enumerate() {
        let x = x0 + i as f64 * (cell + gap);
        sheet
            .ctx
            .fill(if *bit == 1 {
                color
            } else {
                role::figure::background()
            })
            .stroke(role::figure::pen())
            .stroke_width(8.0);
        sheet.ctx.rect(x, y0, cell, cell);
        sheet.label_weighted(
            if *bit == 1 { "1" } else { "0" },
            x + cell / 2.0,
            y0 + cell * 0.28,
            cell * 0.58,
            role::figure::pen(),
            0,
            560.0,
        );
    }
}

fn fig_midpoint(renderer: &Renderer, mono: &str, out: &std::path::Path) {
    let mut sheet = new_sheet(renderer, mono);
    sheet.ctx.line_cap("round");

    // Match the section 03 arch directly: one 8-unit grid, one 10 px pen,
    // and the same source-unit scale. Geometry and layout stay local so the
    // construction can be art-directed without changing the shared system.
    const UNIT: f64 = 15.0;
    const STROKE: f64 = line::EXTRA_HEAVY;
    const POINT_SIZE: f64 = 44.0;
    const CONSTRUCTION_WIDTH: f64 = 128.0;
    const CONSTRUCTION_HEIGHT: f64 = 80.0;
    let origin_x = (W - CONSTRUCTION_WIDTH * UNIT) / 2.0;
    let origin_y = (H - CONSTRUCTION_HEIGHT * UNIT) / 2.0;
    let cx = |ux: f64| origin_x + ux * UNIT;
    let cy = |uy: f64| origin_y + uy * UNIT;

    // The grid phase is tied to the construction origin, so every original
    // control point lands on the visible 8-unit lattice.
    {
        let u_lo = ((-origin_x / UNIT) / 8.0).floor() as i64 * 8;
        let u_hi = (((W - origin_x) / UNIT) / 8.0).ceil() as i64 * 8;
        let v_lo = ((-origin_y / UNIT) / 8.0).floor() as i64 * 8;
        let v_hi = (((H - origin_y) / UNIT) / 8.0).ceil() as i64 * 8;

        let mut u = u_lo;
        while u <= u_hi {
            sheet
                .ctx
                .no_fill()
                .stroke(role::grid::subtle())
                .stroke_width(STROKE);
            let x = origin_x + u as f64 * UNIT;
            sheet.ctx.line(x, 0.0, x, H);
            u += 8;
        }
        let mut v = v_lo;
        while v <= v_hi {
            sheet
                .ctx
                .no_fill()
                .stroke(role::grid::subtle())
                .stroke_width(STROKE);
            let y = origin_y + v as f64 * UNIT;
            sheet.ctx.line(0.0, y, W, y);
            v += 8;
        }
    }

    // Preserve the original cubic, normalized to the section 03 source scale.
    // Its four original controls all sit on the 8-unit grid. The later colored
    // points show repeated dyadic subdivision.
    let p = [(0.0, 0.0), (32.0, 80.0), (96.0, 80.0), (128.0, 0.0)];
    let mid = |a: (f64, f64), b: (f64, f64)| ((a.0 + b.0) / 2.0, (a.1 + b.1) / 2.0);
    let m01 = mid(p[0], p[1]);
    let m12 = mid(p[1], p[2]);
    let m23 = mid(p[2], p[3]);
    let mm0 = mid(m01, m12);
    let mm1 = mid(m12, m23);
    let split = mid(mm0, mm1);

    let line_between = |sheet: &mut Sheet, a: (f64, f64), b: (f64, f64), color: Color| {
        sheet.ctx.no_fill().stroke(color).stroke_width(STROKE);
        sheet.ctx.line(cx(a.0), cy(a.1), cx(b.0), cy(b.1));
    };
    for pair in p.windows(2) {
        line_between(&mut sheet, pair[0], pair[1], role::figure::red());
    }
    line_between(&mut sheet, m01, m12, role::figure::orange());
    line_between(&mut sheet, m12, m23, role::figure::orange());
    line_between(&mut sheet, mm0, mm1, role::figure::yellow());

    let mut curve = kurbo::BezPath::new();
    curve.move_to((cx(p[0].0), cy(p[0].1)));
    curve.curve_to(
        (cx(p[1].0), cy(p[1].1)),
        (cx(p[2].0), cy(p[2].1)),
        (cx(p[3].0), cy(p[3].1)),
    );
    sheet
        .ctx
        .no_fill()
        .stroke(role::figure::pen())
        .stroke_width(STROKE);
    sheet.ctx.draw_path(curve);

    let marker = |sheet: &mut Sheet, p: (f64, f64), fill: Color, square: bool| {
        sheet
            .ctx
            .fill(fill)
            .stroke(role::figure::pen())
            .stroke_width(STROKE);
        let radius = POINT_SIZE / 2.0;
        if square {
            sheet
                .ctx
                .rect(cx(p.0) - radius, cy(p.1) - radius, POINT_SIZE, POINT_SIZE);
        } else {
            sheet
                .ctx
                .oval(cx(p.0) - radius, cy(p.1) - radius, POINT_SIZE, POINT_SIZE);
        }
    };
    for point in p {
        marker(&mut sheet, point, role::figure::red(), true);
    }
    for point in [m01, m12, m23] {
        marker(&mut sheet, point, role::figure::orange(), false);
    }
    for point in [mm0, mm1] {
        marker(&mut sheet, point, role::figure::yellow(), false);
    }
    marker(&mut sheet, split, role::figure::green(), false);

    // Continue the same midpoint operation recursively. Quarter points are
    // blue and eighth points are purple. Keeping these later rounds to points
    // preserves the clean primary construction while showing that subdivision
    // keeps generating positions on the same dyadic ladder.
    let cubic_point = |t: f64| {
        let mt = 1.0 - t;
        let b0 = mt * mt * mt;
        let b1 = 3.0 * mt * mt * t;
        let b2 = 3.0 * mt * t * t;
        let b3 = t * t * t;
        (
            b0 * p[0].0 + b1 * p[1].0 + b2 * p[2].0 + b3 * p[3].0,
            b0 * p[0].1 + b1 * p[1].1 + b2 * p[2].1 + b3 * p[3].1,
        )
    };
    for t in [0.25, 0.75] {
        marker(&mut sheet, cubic_point(t), role::figure::blue(), false);
    }
    for t in [0.125, 0.375, 0.625, 0.875] {
        marker(&mut sheet, cubic_point(t), role::figure::purple(), false);
    }

    sheet.save(renderer, out);
}

fn fig_ladder(renderer: &Renderer, mono: &str, out: &std::path::Path) {
    let mut sheet = new_sheet(renderer, mono);
    sheet.ctx.line_cap("round");

    let xs = [320.0, 946.0, 1574.0, 2200.0];
    let chains: [(&[u32], Color); 4] = [
        (
            &[1024, 512, 256, 128, 64, 32, 16, 8, 4, 2, 1],
            role::figure::green(),
        ),
        (&[1000, 500, 250, 125], role::figure::red()),
        (&[729, 243, 81, 27, 9, 3, 1], role::figure::orange()),
        (&[700, 350, 175], role::figure::yellow()),
    ];
    const DY: f64 = 108.0;
    const BW: f64 = 440.0;
    const BH: f64 = 88.0;
    const TOP: f64 = 1180.0;

    for ((values, color), x) in chains.iter().zip(xs) {
        let last_y = TOP - (values.len() - 1) as f64 * DY;
        sheet
            .ctx
            .no_fill()
            .stroke(role::figure::pen())
            .stroke_width(10.0);
        sheet.ctx.line(x, TOP, x, last_y);
        for (i, value) in values.iter().enumerate() {
            let y = TOP - i as f64 * DY;
            sheet
                .ctx
                .fill(*color)
                .stroke(role::figure::pen())
                .stroke_width(8.0);
            sheet.ctx.rect(x - BW / 2.0, y - BH / 2.0, BW, BH);
            sheet.label_weighted(
                &value.to_string(),
                x,
                y - 18.0,
                60.0,
                role::figure::pen(),
                0,
                560.0,
            );
        }
        if values.last().copied() != Some(1) {
            let y = last_y - 92.0;
            sheet.label_weighted("×", x, y - 18.0, 86.0, role::figure::pen(), 0, 560.0);
        }
    }

    sheet.save(renderer, out);
}

fn fig_bits(renderer: &Renderer, mono: &str, out: &std::path::Path) {
    let mut sheet = new_sheet(renderer, mono);
    sheet.ctx.line_cap("round");

    let rows = [1024u32, 768, 576, 96, 116];
    let colors = [
        role::figure::red(),
        role::figure::orange(),
        role::figure::yellow(),
        role::figure::green(),
        role::figure::red(),
    ];
    const CELL: f64 = 156.0;
    const GAP: f64 = 8.0;
    const X0: f64 = 620.0;
    const BITS: usize = 11;

    for ((value, color), row) in rows.into_iter().zip(colors).zip(0..) {
        let y = 1084.0 - row as f64 * 224.0;
        sheet.label_weighted(
            &value.to_string(),
            530.0,
            y + 34.0,
            84.0,
            role::figure::pen(),
            1,
            560.0,
        );
        for b in 0..BITS {
            let bit_index = BITS - 1 - b;
            let bit = ((value >> bit_index) & 1) as u8;
            bit_row(
                &mut sheet,
                X0 + b as f64 * (CELL + GAP),
                y,
                CELL,
                0.0,
                &[bit],
                color,
            );
        }
        let zeros = value.trailing_zeros() as usize;
        let x_start = X0 + (BITS - zeros) as f64 * (CELL + GAP);
        let x_end = X0 + BITS as f64 * (CELL + GAP) - GAP;
        let by = y - 22.0;
        sheet
            .ctx
            .no_fill()
            .stroke(role::figure::pen())
            .stroke_width(8.0);
        sheet.ctx.line(x_start, by, x_end, by);
        sheet.ctx.line(x_start, by, x_start, by + 14.0);
        sheet.ctx.line(x_end, by, x_end, by + 14.0);
    }

    sheet.save(renderer, out);
}

fn main() {
    let mono_path = inputs::geist_mono();
    let mut renderer = Renderer::new(W as u32, H as u32);
    let mono = load_family(&mut renderer, mono_path.to_str().unwrap());
    let outputs = OutputPaths::from_args();
    fig_midpoint(&renderer, &mono, &outputs.blog("fig-midpoint.png"));
    fig_ladder(&renderer, &mono, &outputs.blog("fig-ladder.png"));
    fig_bits(&renderer, &mono, &outputs.blog("fig-bits.png"));
}
