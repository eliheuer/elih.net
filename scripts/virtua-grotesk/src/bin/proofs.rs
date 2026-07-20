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
            .stroke_width(line::HERO);
        sheet.ctx.rect(x, y0, cell, cell);
        sheet.label(
            if *bit == 1 { "1" } else { "0" },
            x + cell / 2.0,
            y0 + cell * 0.28,
            cell * 0.58,
            role::figure::pen(),
            0,
        );
    }
}

fn fig_fractions(renderer: &Renderer, mono: &str, out: &std::path::Path) {
    let mut sheet = new_sheet(renderer, mono);

    let mut num = 12u32;
    let mut repeating = Vec::new();
    // Twelve bits are enough to show the repeating structure at card size;
    // the ellipsis makes clear that the expansion continues indefinitely.
    for _ in 0..12 {
        num *= 2;
        if num >= 125 {
            repeating.push(1u8);
            num -= 125;
        } else {
            repeating.push(0u8);
        }
    }
    let exact = [0, 0, 0, 1, 1];

    let exact_cell = 240.0;
    let exact_gap = 28.0;
    let exact_width = exact.len() as f64 * exact_cell + (exact.len() - 1) as f64 * exact_gap;
    let exact_left = (W - exact_width) / 2.0 + 100.0;
    sheet.label("1024", MARGIN, 1020.0, 76.0, role::figure::pen(), -1);
    sheet.label(
        "0.",
        exact_left - 92.0,
        886.0,
        104.0,
        role::figure::pen(),
        1,
    );
    bit_row(
        &mut sheet,
        exact_left,
        770.0,
        exact_cell,
        exact_gap,
        &exact,
        role::figure::green(),
    );

    let repeat_cell = 152.0;
    let repeat_gap = 14.0;
    let repeat_width =
        repeating.len() as f64 * repeat_cell + (repeating.len() - 1) as f64 * repeat_gap;
    let repeat_left = (W - repeat_width) / 2.0 + 100.0;
    sheet.label("1000", MARGIN, 402.0, 76.0, role::figure::pen(), -1);
    sheet.label(
        "0.",
        repeat_left - 92.0,
        304.0,
        104.0,
        role::figure::pen(),
        1,
    );
    bit_row(
        &mut sheet,
        repeat_left,
        244.0,
        repeat_cell,
        repeat_gap,
        &repeating,
        role::figure::red(),
    );
    sheet.label("…", W - MARGIN, 286.0, 104.0, role::figure::red(), 1);

    sheet.save(renderer, out);
}

fn fig_midpoint(renderer: &Renderer, mono: &str, out: &std::path::Path) {
    let mut sheet = new_sheet(renderer, mono);

    const S: f64 = 2.60;
    const PX: f64 = 594.0;
    const PY: f64 = 100.0;
    let cx = |ux: f64| PX + ux * S;
    let cy = |uy: f64| PY + uy * S;

    // A single 64-unit lattice supplies scale without turning back into graph
    // paper. It reaches the image edges like the hero's construction lines.
    sheet
        .ctx
        .no_fill()
        .stroke(role::grid::standard())
        .stroke_width(line::THIN);
    let mut x = MARGIN;
    while x <= W - MARGIN {
        sheet.ctx.line(x, MARGIN, x, H - MARGIN);
        x += 64.0 * S;
    }
    let mut y = MARGIN;
    while y <= H - MARGIN {
        sheet.ctx.line(MARGIN, y, W - MARGIN, y);
        y += 64.0 * S;
    }

    let p = [
        (64.0, 128.0),
        (192.0, 448.0),
        (448.0, 448.0),
        (576.0, 128.0),
    ];
    let mid = |a: (f64, f64), b: (f64, f64)| ((a.0 + b.0) / 2.0, (a.1 + b.1) / 2.0);
    let m01 = mid(p[0], p[1]);
    let m12 = mid(p[1], p[2]);
    let m23 = mid(p[2], p[3]);
    let mm0 = mid(m01, m12);
    let mm1 = mid(m12, m23);
    let split = mid(mm0, mm1);

    let line_between = |sheet: &mut Sheet, a: (f64, f64), b: (f64, f64), color: Color| {
        sheet.ctx.no_fill().stroke(color).stroke_width(line::HERO);
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
        .stroke_width(10.0);
    sheet.ctx.draw_path(curve);

    let marker = |sheet: &mut Sheet, p: (f64, f64), fill: Color, square: bool| {
        sheet
            .ctx
            .fill(fill)
            .stroke(role::figure::pen())
            .stroke_width(line::HERO);
        if square {
            sheet.ctx.rect(cx(p.0) - 13.0, cy(p.1) - 13.0, 26.0, 26.0);
        } else {
            sheet.ctx.oval(cx(p.0) - 13.0, cy(p.1) - 13.0, 26.0, 26.0);
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

    sheet.save(renderer, out);
}

fn fig_ladder(renderer: &Renderer, mono: &str, out: &std::path::Path) {
    let mut sheet = new_sheet(renderer, mono);

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
        for (i, value) in values.iter().enumerate() {
            let y = TOP - i as f64 * DY;
            sheet
                .ctx
                .fill(*color)
                .stroke(role::figure::pen())
                .stroke_width(line::HERO);
            sheet.ctx.rect(x - BW / 2.0, y - BH / 2.0, BW, BH);
            sheet.label(
                &value.to_string(),
                x,
                y - 13.0,
                64.0,
                role::figure::pen(),
                0,
            );
            if i + 1 < values.len() {
                sheet
                    .ctx
                    .no_fill()
                    .stroke(role::figure::pen())
                    .stroke_width(line::HERO);
                sheet.ctx.line(x, y - BH / 2.0, x, y - DY + BH / 2.0);
            }
        }
        if values.last().copied() != Some(1) {
            let y = TOP - values.len() as f64 * DY;
            sheet.label("×", x, y - 4.0, 96.0, role::figure::pen(), 0);
        }
    }

    sheet.save(renderer, out);
}

fn fig_bits(renderer: &Renderer, mono: &str, out: &std::path::Path) {
    let mut sheet = new_sheet(renderer, mono);

    let rows = [1024u32, 768, 576, 96, 116];
    let colors = [
        role::figure::red(),
        role::figure::orange(),
        role::figure::yellow(),
        role::figure::green(),
        role::figure::red(),
    ];
    const CELL: f64 = 144.0;
    const GAP: f64 = 8.0;
    const X0: f64 = 650.0;
    const BITS: usize = 11;

    for ((value, color), row) in rows.into_iter().zip(colors).zip(0..) {
        let y = 1070.0 - row as f64 * 220.0;
        sheet.label(
            &value.to_string(),
            570.0,
            y + 28.0,
            76.0,
            role::figure::pen(),
            1,
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
            .stroke_width(line::HERO);
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
    fig_fractions(&renderer, &mono, &outputs.blog("fig-fractions.png"));
    fig_midpoint(&renderer, &mono, &outputs.blog("fig-midpoint.png"));
    fig_ladder(&renderer, &mono, &outputs.blog("fig-ladder.png"));
    fig_bits(&renderer, &mono, &outputs.blog("fig-bits.png"));
}
