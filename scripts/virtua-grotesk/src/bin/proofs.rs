//! Four graphic proofs for the powers-of-two section. Each composition keeps
//! one visual argument and omits prose already present in the article.

use designbot::prelude::*;
use designbot_render::Renderer;
use virtua_grotesk_figures::*;

fn bit_row(sheet: &mut Sheet, x0: f64, y0: f64, cell: f64, gap: f64, bits: &[u8], color: Color) {
    let vb = ValueBox {
        w: cell,
        h: cell,
        stroke: line::BOX,
        text_size: cell * 0.58,
        text_dy: cell * 0.28,
        weight: 560.0,
    };
    for (i, bit) in bits.iter().enumerate() {
        let x = x0 + i as f64 * (cell + gap);
        let fill = if *bit == 1 {
            color
        } else {
            role::figure::background()
        };
        vb.draw(sheet, x, y0, fill, if *bit == 1 { "1" } else { "0" });
    }
}

fn fig_midpoint(renderer: &Renderer, mono: &str, out: &std::path::Path) {
    let mut sheet = new_sheet(renderer, mono);
    sheet.ctx.line_cap("round");

    // Match the section 03 arch directly: one 4-unit grid, one 10 px pen,
    // and the same source-unit scale. Geometry and layout stay local so the
    // construction can be art-directed without changing the shared system.
    const UNIT: f64 = 16.5;
    const STROKE: f64 = line::EXTRA_HEAVY;
    const POINT_SIZE: f64 = 44.0;
    const CONSTRUCTION_WIDTH: f64 = 136.0;
    const CONSTRUCTION_HEIGHT: f64 = 64.0;
    let origin_x = (W - CONSTRUCTION_WIDTH * UNIT) / 2.0;
    let origin_y = (H - CONSTRUCTION_HEIGHT * UNIT) / 2.0;
    let cx = |ux: f64| origin_x + ux * UNIT;
    let cy = |uy: f64| origin_y + uy * UNIT;

    // The grid phase is tied to the construction origin. The denser visible
    // lattice makes the central midpoint construction legible as it moves
    // from the coarse source coordinates into finer dyadic levels.
    {
        const GRID: i64 = 4;
        let u_lo = ((-origin_x / UNIT) / GRID as f64).floor() as i64 * GRID;
        let u_hi = (((W - origin_x) / UNIT) / GRID as f64).ceil() as i64 * GRID;
        let v_lo = ((-origin_y / UNIT) / GRID as f64).floor() as i64 * GRID;
        let v_hi = (((H - origin_y) / UNIT) / GRID as f64).ceil() as i64 * GRID;

        let mut u = u_lo;
        while u <= u_hi {
            sheet
                .ctx
                .no_fill()
                .stroke(role::grid::subtle())
                .stroke_width(STROKE);
            let x = origin_x + u as f64 * UNIT;
            if x > STROKE && x < W - STROKE {
                sheet.ctx.line(x, 0.0, x, H);
            }
            u += GRID;
        }
        let mut v = v_lo;
        while v <= v_hi {
            sheet
                .ctx
                .no_fill()
                .stroke(role::grid::subtle())
                .stroke_width(STROKE);
            let y = origin_y + v as f64 * UNIT;
            if y > STROKE && y < H - STROKE {
                sheet.ctx.line(0.0, y, W, y);
            }
            v += GRID;
        }
    }

    // The controls sit about two visible grid cells from every image edge.
    // Every control and midpoint in this complete split lands on the visible
    // 4-unit grid.
    let p = [(0.0, 0.0), (40.0, 64.0), (96.0, 64.0), (136.0, 0.0)];
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

    let dot = Marker {
        size: POINT_SIZE,
        stroke: STROKE,
    };
    let marker = |sheet: &mut Sheet, p: (f64, f64), fill: Color, square: bool| {
        dot.draw(sheet, cx(p.0), cy(p.1), fill, square);
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
    sheet.ctx.line_cap("round");

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
    const CHAIN_COUNT: f64 = 4.0;
    const LONGEST_STEP_COUNT: f64 = 10.0;

    // Four equal-width stacks leave five equal horizontal spaces: the two
    // outer margins and the three gaps between stacks. Center the longest
    // eleven-rung stack vertically, then align every shorter stack to its top.
    const GAP: f64 = (W - CHAIN_COUNT * BW) / (CHAIN_COUNT + 1.0);
    const TOP: f64 = (H + LONGEST_STEP_COUNT * DY) / 2.0;
    let xs = [
        GAP + BW / 2.0,
        2.0 * GAP + 1.5 * BW,
        3.0 * GAP + 2.5 * BW,
        4.0 * GAP + 3.5 * BW,
    ];

    let rung = ValueBox {
        w: BW,
        h: BH,
        stroke: line::BOX,
        text_size: 60.0,
        text_dy: BH / 2.0 - 18.0,
        weight: 560.0,
    };
    for ((values, color), x) in chains.iter().zip(xs) {
        let last_y = TOP - (values.len() - 1) as f64 * DY;
        sheet
            .ctx
            .no_fill()
            .stroke(role::figure::pen())
            .stroke_width(line::EXTRA_HEAVY);
        sheet.ctx.line(x, TOP, x, last_y);
        for (i, value) in values.iter().enumerate() {
            let y = TOP - i as f64 * DY;
            rung.draw(
                &mut sheet,
                x - BW / 2.0,
                y - BH / 2.0,
                *color,
                &value.to_string(),
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
    const GAP: f64 = 32.0;
    const BITS: usize = 11;
    const LABEL_SIZE: f64 = 84.0;
    const LABEL_GAP: f64 = 80.0;
    const ROW_DY: f64 = 224.0;
    const BRACKET_DROP: f64 = 42.0;
    const BRACKET_CAP: f64 = 16.0;

    // Center the visible composition, including the widest row label, rather
    // than centering the bit blocks alone. The vertical bounds include the
    // first row's cells and the last row's trailing-zero bracket.
    let bits_width = BITS as f64 * CELL + (BITS - 1) as f64 * GAP;
    let label_width = sheet.mono_width_weighted("1024", LABEL_SIZE, 560.0);
    let content_width = label_width + LABEL_GAP + bits_width;
    let content_left = (W - content_width) / 2.0;
    let label_right = content_left + label_width;
    let x0 = label_right + LABEL_GAP;
    let content_height = 4.0 * ROW_DY + CELL + BRACKET_DROP;
    let content_bottom = (H - content_height) / 2.0;
    let first_row_y = content_bottom + BRACKET_DROP + 4.0 * ROW_DY;

    for ((value, color), row) in rows.into_iter().zip(colors).zip(0..) {
        let y = first_row_y - row as f64 * ROW_DY;
        sheet.label_weighted(
            &value.to_string(),
            label_right,
            y + 34.0,
            LABEL_SIZE,
            role::figure::pen(),
            1,
            560.0,
        );
        for b in 0..BITS {
            let bit_index = BITS - 1 - b;
            let bit = ((value >> bit_index) & 1) as u8;
            bit_row(
                &mut sheet,
                x0 + b as f64 * (CELL + GAP),
                y,
                CELL,
                0.0,
                &[bit],
                color,
            );
        }
        let zeros = value.trailing_zeros() as usize;
        let x_start = x0 + (BITS - zeros) as f64 * (CELL + GAP);
        let x_end = x0 + bits_width;
        let by = y - BRACKET_DROP;
        sheet
            .ctx
            .no_fill()
            .stroke(role::figure::pen())
            .stroke_width(line::BOX);
        sheet.ctx.line(x_start, by, x_end, by);
        sheet.ctx.line(x_start, by, x_start, by + BRACKET_CAP);
        sheet.ctx.line(x_end, by, x_end, by + BRACKET_CAP);
    }

    sheet.save(renderer, out);
}

fn fig_scaling(renderer: &Renderer, mono: &str, out: &std::path::Path) {
    let mut sheet = new_sheet(renderer, mono);
    sheet.ctx.line_cap("round");

    // Device value of coordinate c at pixel size p on an em: 64·c·p/em
    // sixty-fourths. A cell is green when that value is an integer on the
    // rasterizer's 1/64 px grid, red when it must round. Rows use real
    // design values: the x-height (576 / 562), the 64- and 96-unit
    // structural measures, the 88 counter measurement, and the corrected
    // 116. The 1000-em panel is red everywhere.
    let sizes = [12u64, 13, 14, 15, 16, 17, 18];
    let rows: [(u64, u64); 5] = [(576, 562), (116, 116), (96, 96), (88, 88), (64, 64)];
    const CELL: f64 = 128.0;
    const PITCH: f64 = 140.0;
    const GRID_W: f64 = 6.0 * PITCH + CELL;
    const PANEL_W: f64 = 1116.0;
    const OUTER_MARGIN: f64 = 96.0;
    const PANEL_GAP: f64 = 96.0;
    const STUB_W: f64 = PANEL_W - GRID_W;
    const STUB_GAP: f64 = 24.0;
    const TABLE_BOTTOM: f64 = 210.0;
    const ROW_DY: f64 = 140.0;
    const TEXT_SIZE: f64 = 68.0;
    const TEXT_WEIGHT: f32 = 560.0;
    const COLUMN_BASELINE: f64 = 950.0;
    const TITLE_BASELINE: f64 = 1058.0;
    let panels: [(u64, f64, usize); 2] = [
        (1024, OUTER_MARGIN, 0),
        (1000, OUTER_MARGIN + PANEL_W + PANEL_GAP, 1),
    ];
    let cell_box = ValueBox {
        w: CELL,
        h: CELL,
        stroke: line::BOX,
        text_size: TEXT_SIZE,
        text_dy: 38.0,
        weight: TEXT_WEIGHT,
    };

    for (em, panel_left, which) in panels {
        let grid_left = panel_left + STUB_W;
        sheet.label_weighted(
            &em.to_string(),
            panel_left + PANEL_W / 2.0,
            TITLE_BASELINE,
            TEXT_SIZE,
            role::figure::pen(),
            0,
            TEXT_WEIGHT,
        );
        for (i, p) in sizes.iter().enumerate() {
            sheet.label_weighted(
                &p.to_string(),
                grid_left + i as f64 * PITCH + CELL / 2.0,
                COLUMN_BASELINE,
                TEXT_SIZE,
                role::figure::pen(),
                0,
                TEXT_WEIGHT,
            );
        }
        for (r, row) in rows.iter().enumerate() {
            let c = if which == 0 { row.0 } else { row.1 };
            let y = TABLE_BOTTOM + (rows.len() - 1 - r) as f64 * ROW_DY;
            sheet.label_weighted(
                &c.to_string(),
                grid_left - STUB_GAP,
                y + cell_box.text_dy,
                TEXT_SIZE,
                role::figure::pen(),
                1,
                TEXT_WEIGHT,
            );
            for (i, p) in sizes.iter().enumerate() {
                let exact = (64 * c * p) % em == 0;
                let x = grid_left + i as f64 * PITCH;
                let fill = if exact {
                    role::figure::green()
                } else {
                    role::figure::red()
                };
                cell_box.draw(&mut sheet, x, y, fill, if exact { "" } else { "×" });
            }
        }
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
    fig_scaling(&renderer, &mono, &outputs.blog("fig-scaling.png"));
}
