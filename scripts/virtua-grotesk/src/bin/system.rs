//! Design-system dimension sheets for the Virtua Grotesk post, §03 — the
//! figures that replace the measurement tables:
//!
//!   fig-system-no.png      : the lowercase system on "no", zoomable
//!   fig-system-ho.png      : the capital system on "HO", zoomable
//!   fig-system-weights.png : "no no" — Regular beside Bold, stems 96 -> 192
//!   fig-system-arabic.png  : alef, beh, medial heh — right to left, same grid
//!
//! Everything shared (palette, frame, point language, annotation engine)
//! lives in the crate library; this bin is only the four compositions.
//! All dimensions are MEASURED from the UFO outlines.
//!
//! Run from this directory:
//!
//!     cargo run --release --bin system

use designbot::prelude::Color;
use designbot_render::Renderer;
use virtua_grotesk_figures::*;

// --- shared system-figure drawing language ----------------------------------------

// Keep these equal to the reviewed OG image. The inline system figures are a
// continuation of that drawing, not a second annotation system.
const SYSTEM_STROKE: f64 = line::HERO;
const SYSTEM_GRID_STROKE: f64 = line::FINE;
const SYSTEM_GRID_UNIT: f64 = 8.0;
const SYSTEM_POINT_SIZE: f64 = 20.0;
const SYSTEM_MEASUREMENT_TEXT_SIZE: f64 = 32.0;
const SYSTEM_MEASUREMENT_TEXT_WEIGHT: f32 = 600.0;
const SYSTEM_MEASUREMENT_CAP: f64 = 12.0;
const SYSTEM_MEASUREMENT_LINE_GAP: f64 = 28.0;
const SYSTEM_POINT_END_INSET: f64 = 30.0;
const SYSTEM_EDGE_END_INSET: f64 = 20.0;
const SYSTEM_COUNTER_LINE_GAP: f64 = 72.0;
const SYSTEM_COUNTER_LABEL_SHIFT: f64 = 160.0;

#[derive(Clone, Copy)]
struct SystemMeasurement {
    glyph: usize,
    p0: (f64, f64),
    p1: (f64, f64),
    value: i64,
    label_shift: (f64, f64),
    sum_break_after: usize,
    knockout: bool,
    line_gap: Option<(f64, f64)>,
    end_inset: f64,
}

const fn point_measurement(
    glyph: usize,
    p0: (f64, f64),
    p1: (f64, f64),
    value: i64,
) -> SystemMeasurement {
    SystemMeasurement {
        glyph,
        p0,
        p1,
        value,
        label_shift: (0.0, 0.0),
        sum_break_after: 0,
        knockout: false,
        line_gap: None,
        end_inset: SYSTEM_POINT_END_INSET,
    }
}

const fn edge_measurement(
    glyph: usize,
    p0: (f64, f64),
    p1: (f64, f64),
    value: i64,
) -> SystemMeasurement {
    SystemMeasurement {
        glyph,
        p0,
        p1,
        value,
        label_shift: (0.0, 0.0),
        sum_break_after: 0,
        knockout: false,
        line_gap: None,
        end_inset: SYSTEM_EDGE_END_INSET,
    }
}

const fn break_measurement_sum(
    mut measurement: SystemMeasurement,
    after: usize,
) -> SystemMeasurement {
    measurement.sum_break_after = after;
    measurement
}

const fn counter_measurement(mut measurement: SystemMeasurement) -> SystemMeasurement {
    measurement.knockout = true;
    measurement
}

const fn gap_measurement_line(
    mut measurement: SystemMeasurement,
    center: f64,
    half_height: f64,
) -> SystemMeasurement {
    measurement.line_gap = Some((center, half_height));
    measurement
}

const fn shift_measurement_label(
    mut measurement: SystemMeasurement,
    dx: f64,
    dy: f64,
) -> SystemMeasurement {
    measurement.label_shift = (dx, dy);
    measurement
}

fn system_construction_node(sheet: &mut Sheet, x: f64, y: f64) {
    marker_with_fill_sized(
        sheet,
        x,
        y,
        PtRole::Smooth,
        role::figure::pen(),
        role::figure::point_fill(),
        SYSTEM_POINT_SIZE,
        SYSTEM_STROKE,
    );
}

/// The real, uniform 8-unit source grid. Vertical coordinates restart at each
/// sort boundary, as they do in a font editor.
fn system_background_grid(
    sheet: &mut Sheet,
    frame: &Frame,
    glyphs: &[(&Outline, f64)],
    bottom: f64,
    top: f64,
) {
    let x0 = frame.x(0.0);
    let x1 = frame.x(glyphs
        .last()
        .map(|(outline, origin)| origin + outline.width)
        .unwrap_or(0.0));

    let mut v = bottom;
    while v <= top {
        sheet
            .ctx
            .no_fill()
            .stroke(role::grid::faint())
            .stroke_width(SYSTEM_GRID_STROKE);
        sheet.ctx.line(x0, frame.y(v), x1, frame.y(v));
        v += SYSTEM_GRID_UNIT;
    }

    for (outline, origin) in glyphs {
        let mut u = 0.0;
        while u <= outline.width {
            sheet
                .ctx
                .no_fill()
                .stroke(role::grid::faint())
                .stroke_width(SYSTEM_GRID_STROKE);
            let x = frame.x(origin + u);
            sheet.ctx.line(x, frame.y(bottom), x, frame.y(top));
            u += SYSTEM_GRID_UNIT;
        }
    }
}

/// Metric rules, advance boundaries, and intersection nodes use the same
/// geometry and pen as the OG image. The frame ends at the true overshoots.
fn system_metric_system(
    sheet: &mut Sheet,
    frame: &Frame,
    run: f64,
    bounds: &[f64],
    solid: &[f64],
    dashed: &[f64],
    top: f64,
    bottom: f64,
) {
    let x0 = frame.x(0.0);
    let x1 = frame.x(run);
    let color = role::figure::pen();

    sheet
        .ctx
        .no_fill()
        .stroke(color)
        .stroke_width(SYSTEM_STROKE);
    sheet.ctx.line_dash(&[10.0, 10.0]);
    for &uy in dashed {
        sheet.ctx.line(x0, frame.y(uy), x1, frame.y(uy));
    }
    sheet.ctx.line_dash(&[]);
    for &uy in solid {
        sheet.ctx.line(x0, frame.y(uy), x1, frame.y(uy));
    }

    let xs: Vec<f64> = bounds.iter().map(|bound| frame.x(*bound)).collect();
    cell_dividers_colored(
        sheet,
        &xs,
        frame.y(top),
        frame.y(bottom),
        color,
        role::figure::point_fill(),
        SYSTEM_STROKE,
    );

    let mut ys: Vec<f64> = dashed.iter().map(|uy| frame.y(*uy)).collect();
    ys.extend(solid.iter().map(|uy| frame.y(*uy)));
    for x in xs {
        for &y in &ys {
            system_construction_node(sheet, x, y);
        }
    }
}

fn system_measurement_label(
    sheet: &mut Sheet,
    measurement: SystemMeasurement,
    text: &str,
    x: f64,
    y: f64,
    align: i8,
) {
    if measurement.knockout {
        sheet.label_padded_weighted_on(
            text,
            x,
            y,
            SYSTEM_MEASUREMENT_TEXT_SIZE,
            role::figure::pen(),
            align,
            role::figure::background(),
            SYSTEM_MEASUREMENT_TEXT_WEIGHT,
        );
    } else {
        sheet.label_weighted(
            text,
            x,
            y,
            SYSTEM_MEASUREMENT_TEXT_SIZE,
            role::figure::pen(),
            align,
            SYSTEM_MEASUREMENT_TEXT_WEIGHT,
        );
    }
}

/// An OG-style capped size label. These are intentionally independent from
/// the removed advance-width band so the enlarged figures can retain the
/// useful stroke and counter dimensions without giving space back to metrics.
fn system_glyph_measurement(
    sheet: &mut Sheet,
    frame: &Frame,
    origin: f64,
    measurement: SystemMeasurement,
) {
    let p0 = (
        frame.x(origin + measurement.p0.0),
        frame.y(measurement.p0.1),
    );
    let p1 = (
        frame.x(origin + measurement.p1.0),
        frame.y(measurement.p1.1),
    );
    let dx = p1.0 - p0.0;
    let dy = p1.1 - p0.1;
    let length = (dx * dx + dy * dy).sqrt();
    let direction = (dx / length, dy / length);
    let normal = (-direction.1, direction.0);
    let q0 = (
        p0.0 + direction.0 * measurement.end_inset,
        p0.1 + direction.1 * measurement.end_inset,
    );
    let q1 = (
        p1.0 - direction.0 * measurement.end_inset,
        p1.1 - direction.1 * measurement.end_inset,
    );
    let color = role::figure::pen();

    sheet
        .ctx
        .no_fill()
        .stroke(color)
        .stroke_width(SYSTEM_STROKE);
    if let Some((gap_center, gap_half_height)) = measurement.line_gap {
        let gap_center = frame.y(gap_center);
        sheet
            .ctx
            .line(q0.0, q0.1, q0.0, gap_center - gap_half_height);
        sheet
            .ctx
            .line(q1.0, gap_center + gap_half_height, q1.0, q1.1);
    } else {
        sheet.ctx.line(q0.0, q0.1, q1.0, q1.1);
    }
    for q in [q0, q1] {
        sheet.ctx.line(
            q.0 - normal.0 * SYSTEM_MEASUREMENT_CAP,
            q.1 - normal.1 * SYSTEM_MEASUREMENT_CAP,
            q.0 + normal.0 * SYSTEM_MEASUREMENT_CAP,
            q.1 + normal.1 * SYSTEM_MEASUREMENT_CAP,
        );
    }

    let midpoint = ((q0.0 + q1.0) / 2.0, (q0.1 + q1.1) / 2.0);
    let decomposition = p2sum(measurement.value);
    let parts: Vec<&str> = decomposition.split('+').collect();
    let sum_lines = if measurement.sum_break_after > 0 && measurement.sum_break_after < parts.len()
    {
        vec![
            parts[..measurement.sum_break_after].join("+"),
            format!("+{}", parts[measurement.sum_break_after..].join("+")),
        ]
    } else {
        vec![decomposition]
    };

    if dx.abs() >= dy.abs() {
        system_measurement_label(
            sheet,
            measurement,
            &measurement.value.to_string(),
            midpoint.0 + measurement.label_shift.0,
            midpoint.1 + 20.0 + measurement.label_shift.1,
            0,
        );
        for (index, line) in sum_lines.iter().enumerate() {
            system_measurement_label(
                sheet,
                measurement,
                line,
                midpoint.0 + measurement.label_shift.0,
                midpoint.1 - 48.0 + measurement.label_shift.1
                    - index as f64 * SYSTEM_MEASUREMENT_LINE_GAP,
                0,
            );
        }
    } else {
        system_measurement_label(
            sheet,
            measurement,
            &measurement.value.to_string(),
            midpoint.0 - 16.0 + measurement.label_shift.0,
            midpoint.1 - 12.0 + measurement.label_shift.1,
            1,
        );
        for (index, line) in sum_lines.iter().enumerate() {
            system_measurement_label(
                sheet,
                measurement,
                line,
                midpoint.0 + 16.0 + measurement.label_shift.0,
                midpoint.1 - 12.0 + measurement.label_shift.1
                    - index as f64 * SYSTEM_MEASUREMENT_LINE_GAP,
                -1,
            );
        }
    }
}

fn system_frame(run: f64, bottom: f64, top: f64) -> Frame {
    let s = ((W - 2.0 * MARGIN) / run).min((H - 2.0 * MARGIN) / (top - bottom));
    Frame {
        s,
        x0: (W - run * s) / 2.0,
        baseline: MARGIN - bottom * s,
    }
}

// --- fig-system-no -----------------------------------------------------------------

fn fig_no(renderer: &Renderer, mono: &str, reg: &std::path::Path, out: &std::path::Path) {
    let mut sheet = new_sheet(renderer, mono);
    sheet.ctx.line_cap("round");

    let n = load_outline(reg, "n");
    let o = load_outline(reg, "o");

    const BOTTOM: f64 = -16.0;
    const TOP: f64 = 592.0;
    let run_units = n.width + o.width;
    let f = system_frame(run_units, BOTTOM, TOP);

    let x_n = 0.0;
    let x_o = n.width;
    let glyphs = [(&n, x_n), (&o, x_o)];
    let bounds = [x_n, x_o, run_units];
    system_background_grid(&mut sheet, &f, &glyphs, BOTTOM, TOP);
    system_metric_system(
        &mut sheet,
        &f,
        run_units,
        &bounds,
        &[0.0, 576.0],
        &[-16.0, 592.0],
        TOP,
        BOTTOM,
    );

    for (outline, ox, fill) in [
        (&n, x_n, role::figure::yellow()),
        (&o, x_o, role::figure::green()),
    ] {
        draw_figure_glyph(&mut sheet, outline, f.s, f.x(ox), f.baseline, fill);
    }

    // Representative stroke sizes plus the open n counter and closed o
    // counter. Values and endpoints are taken directly from the current UFO.
    const MEASUREMENTS: [SystemMeasurement; 6] = [
        edge_measurement(0, (64.0, 288.0), (160.0, 288.0), 96),
        counter_measurement(edge_measurement(0, (160.0, 288.0), (432.0, 288.0), 272)),
        point_measurement(1, (32.0, 288.0), (132.0, 288.0), 100),
        point_measurement(1, (304.0, 504.0), (304.0, 592.0), 88),
        shift_measurement_label(
            gap_measurement_line(
                counter_measurement(point_measurement(1, (304.0, 72.0), (304.0, 504.0), 432)),
                288.0,
                SYSTEM_COUNTER_LINE_GAP,
            ),
            0.0,
            SYSTEM_COUNTER_LABEL_SHIFT,
        ),
        counter_measurement(point_measurement(1, (132.0, 288.0), (484.0, 288.0), 352)),
    ];
    let origins = [x_n, x_o];
    for measurement in MEASUREMENTS {
        system_glyph_measurement(&mut sheet, &f, origins[measurement.glyph], measurement);
    }
    sheet.save(renderer, out);
}

// --- fig-system-ho -----------------------------------------------------------------

fn fig_ho(renderer: &Renderer, mono: &str, reg: &std::path::Path, out: &std::path::Path) {
    let mut sheet = new_sheet(renderer, mono);
    sheet.ctx.line_cap("round");

    let h = load_outline(reg, "H");
    let o = load_outline(reg, "O");

    const BOTTOM: f64 = -16.0;
    const TOP: f64 = 784.0;
    let run_units = h.width + o.width;
    let f = system_frame(run_units, BOTTOM, TOP);

    let x_h = 0.0;
    let x_o = h.width;
    let glyphs = [(&h, x_h), (&o, x_o)];
    let bounds = [x_h, x_o, run_units];
    system_background_grid(&mut sheet, &f, &glyphs, BOTTOM, TOP);
    system_metric_system(
        &mut sheet,
        &f,
        run_units,
        &bounds,
        &[0.0, 768.0],
        &[-16.0, 784.0],
        TOP,
        BOTTOM,
    );

    for (outline, ox, fill) in [
        (&h, x_h, role::figure::orange()),
        (&o, x_o, role::figure::red()),
    ] {
        draw_figure_glyph(&mut sheet, outline, f.s, f.x(ox), f.baseline, fill);
    }

    // The H opening and O counter join the representative stem and crossbar
    // sizes. Values and endpoints are taken directly from the current UFO.
    const MEASUREMENTS: [SystemMeasurement; 7] = [
        point_measurement(0, (80.0, 600.0), (184.0, 600.0), 104),
        counter_measurement(edge_measurement(0, (184.0, 600.0), (584.0, 600.0), 400)),
        edge_measurement(0, (384.0, 360.0), (384.0, 456.0), 96),
        break_measurement_sum(point_measurement(1, (48.0, 384.0), (156.0, 384.0), 108), 2),
        point_measurement(1, (424.0, 684.0), (424.0, 784.0), 100),
        shift_measurement_label(
            gap_measurement_line(
                counter_measurement(point_measurement(1, (424.0, 84.0), (424.0, 684.0), 600)),
                384.0,
                SYSTEM_COUNTER_LINE_GAP,
            ),
            0.0,
            SYSTEM_COUNTER_LABEL_SHIFT,
        ),
        counter_measurement(point_measurement(1, (156.0, 384.0), (692.0, 384.0), 536)),
    ];
    let origins = [x_h, x_o];
    for measurement in MEASUREMENTS {
        system_glyph_measurement(&mut sheet, &f, origins[measurement.glyph], measurement);
    }
    sheet.save(renderer, out);
}

// --- fig-system-weights: no in Regular and Bold --------------------------------------

fn fig_weights(
    renderer: &Renderer,
    mono: &str,
    reg: &std::path::Path,
    bold: &std::path::Path,
    out: &std::path::Path,
) {
    let mut sheet = new_sheet(renderer, mono);

    let rn = load_outline(reg, "n");
    let ro = load_outline(reg, "o");
    let bn = load_outline(bold, "n");
    let bo = load_outline(bold, "o");

    const S: f64 = 0.91;
    const PAIR_GAP: f64 = 72.0;
    let run = (rn.width + ro.width + bn.width + bo.width) * S + PAIR_GAP;
    let f = Frame {
        s: S,
        x0: MARGIN
            + (W - 2.0 * MARGIN - run).min(0.0).max(W * -1.0) / 2.0
            + (W - 2.0 * MARGIN - run).max(0.0) / 2.0,
        baseline: 270.0,
    };
    metric_lines(&mut sheet, &f, &[0.0, 576.0], &[-16.0, 592.0]);

    let x_rn = f.x(0.0);
    let x_ro = f.x(rn.width);
    let x_bn = f.x(rn.width + ro.width) + PAIR_GAP;
    let x_bo = x_bn + bn.width * S;

    for (outline, x, fill) in [
        (&rn, x_rn, role::figure::red()),
        (&ro, x_ro, role::figure::orange()),
        (&bn, x_bn, role::figure::yellow()),
        (&bo, x_bo, role::figure::green()),
    ] {
        draw_figure_glyph(&mut sheet, outline, S, x, f.baseline, fill);
    }

    // stems and curves, both weights, all measured
    sheet.dim_h(
        x_rn + 64.0 * S,
        x_rn + 160.0 * S,
        f.y(256.0),
        "96",
        role::figure::pen(),
    );
    sheet.dim_h(
        x_ro + 32.0 * S,
        x_ro + 132.0 * S,
        f.y(288.0),
        "100",
        role::figure::pen(),
    );
    sheet.dim_v(
        x_ro + 304.0 * S,
        f.y(500.0),
        f.y(592.0),
        "92",
        role::figure::pen(),
        true,
    );
    sheet.dim_h(
        x_bn + 64.0 * S,
        x_bn + 256.0 * S,
        f.y(256.0),
        "192",
        role::figure::pen(),
    );
    sheet.dim_h(
        x_bo + 32.0 * S,
        x_bo + 228.0 * S,
        f.y(288.0),
        "196",
        role::figure::pen(),
    );
    sheet.dim_v(
        x_bo + 344.0 * S,
        f.y(452.0),
        f.y(592.0),
        "140",
        role::figure::pen(),
        true,
    );
    sheet.save(renderer, out);
}

// --- fig-system-arabic: alef, beh, medial heh, right to left --------------------------

fn fig_arabic(renderer: &Renderer, mono: &str, reg: &std::path::Path, out: &std::path::Path) {
    let mut sheet = new_sheet(renderer, mono);

    let alef = load_outline(reg, "alef-ar");
    let beh = load_outline(reg, "beh-ar"); // components resolved: boat + dot
    let heh = load_outline(reg, "heh-ar.medi");

    const S: f64 = 1.12;
    let f = Frame {
        s: S,
        x0: MARGIN,
        baseline: 390.0,
    };
    metric_lines(&mut sheet, &f, &[0.0, 768.0], &[]);

    // three slots, read right to left: alef, beh, medial heh
    let slot_w = (W - 2.0 * MARGIN) / 3.0;
    let center = |i: f64| MARGIN + (i + 0.5) * slot_w;
    let x_alef = center(2.0) - alef.width * S / 2.0;
    let x_beh = center(1.0) - beh.width * S / 2.0;
    let x_heh = center(0.0) - heh.width * S / 2.0;

    draw_figure_glyph(
        &mut sheet,
        &alef,
        S,
        x_alef,
        f.baseline,
        role::figure::green(),
    );
    draw_figure_glyph(
        &mut sheet,
        &beh,
        S,
        x_beh,
        f.baseline,
        role::figure::orange(),
    );
    draw_figure_glyph(&mut sheet, &heh, S, x_heh, f.baseline, role::figure::red());

    // measured dimensions: alef stroke, beh boat stroke, the dot's diameter
    sheet.dim_h(
        x_alef + 64.0 * S,
        x_alef + 160.0 * S,
        f.y(384.0),
        "96",
        role::figure::pen(),
    );
    sheet.dim_v(
        x_beh + 296.0 * S,
        f.y(0.0),
        f.y(72.0),
        "72",
        role::figure::pen(),
        true,
    );
    sheet.dim_h(
        x_beh + 42.0 * S,
        x_beh + 202.0 * S,
        f.y(-192.0),
        "160",
        role::figure::pen(),
    );
    sheet.save(renderer, out);
}

// --- fig-semantic-grid: the whole argument in one image -------------------------------
//
// Left: no, with the dimensions color-coded by layer (96 = machine green,
// 100 and 92 = the hand's +/-4 in red). Top right: why 96 and 100 mean
// different things — not their power-of-two decomposition (every integer
// has one) but their trailing zeros. Bottom right: the measured proof that
// Virtua-12M-v0.1 learned the tiers from geometry alone.

/// Graphic reduction of the original semantic-grid dashboard. The left side
/// shows the real measured outlines; the right side shows the two bit patterns
/// and the two observed rates. Layout constants stay local and explicit so
/// this composition remains easy to art-direct by hand.
fn fig_semantic(renderer: &Renderer, mono: &str, reg: &std::path::Path, out: &std::path::Path) {
    let mut sheet = new_sheet(renderer, mono);
    let n = load_outline(reg, "n");
    let o = load_outline(reg, "o");

    const DIVIDER_X: f64 = 1580.0;
    const S: f64 = 1.18;
    const BASELINE: f64 = 278.0;
    let run = (n.width + o.width) * S;
    let x0 = MARGIN + (DIVIDER_X - MARGIN - run) / 2.0;
    let frame = Frame {
        s: S,
        x0,
        baseline: BASELINE,
    };

    // Metrics stop at the divider; the data column has its own rhythm.
    for (y, dashed) in [(0.0, false), (576.0, false), (-16.0, true), (592.0, true)] {
        sheet
            .ctx
            .no_fill()
            .stroke(blue())
            .stroke_width(line::HERO)
            .line_dash(if dashed { &[12.0, 12.0] } else { &[] });
        sheet
            .ctx
            .line(MARGIN, frame.y(y), DIVIDER_X - 44.0, frame.y(y));
    }
    sheet.ctx.line_dash(&[]);

    draw_figure_glyph(&mut sheet, &n, S, x0, BASELINE, role::figure::red());
    draw_figure_glyph(
        &mut sheet,
        &o,
        S,
        x0 + n.width * S,
        BASELINE,
        role::figure::green(),
    );
    sheet.dim_h(
        x0 + 64.0 * S,
        x0 + 160.0 * S,
        BASELINE + 250.0 * S,
        "96",
        role::figure::pen(),
    );
    sheet.dim_h(
        x0 + (n.width + 32.0) * S,
        x0 + (n.width + 132.0) * S,
        BASELINE + 310.0 * S,
        "100",
        role::figure::pen(),
    );

    sheet
        .ctx
        .no_fill()
        .stroke(role::figure::pen())
        .stroke_width(line::HERO);
    sheet.ctx.line(DIVIDER_X, MARGIN, DIVIDER_X, H - MARGIN);

    let rx = DIVIDER_X + 90.0;
    let cell = 86.0;
    let gap = 10.0;
    let draw_bits = |sheet: &mut Sheet, value: u32, y: f64, fill: Color| {
        sheet.label(
            &value.to_string(),
            rx,
            y + 20.0,
            54.0,
            role::figure::pen(),
            -1,
        );
        for bit in 0..7 {
            let x = rx + 150.0 + bit as f64 * (cell + gap);
            let one = (value >> (6 - bit)) & 1 == 1;
            sheet
                .ctx
                .fill(if one {
                    fill
                } else {
                    role::figure::background()
                })
                .stroke(role::figure::pen())
                .stroke_width(line::HERO);
            sheet.ctx.rect(x, y, cell, cell);
            sheet.label(
                if one { "1" } else { "0" },
                x + cell / 2.0,
                y + 22.0,
                44.0,
                role::figure::pen(),
                0,
            );
        }
    };
    draw_bits(&mut sheet, 96, 1000.0, role::figure::orange());
    draw_bits(&mut sheet, 100, 790.0, role::figure::yellow());

    let bar_w = 690.0;
    let draw_bar = |sheet: &mut Sheet, pct: f64, y: f64, fill: Color| {
        sheet
            .ctx
            .fill(fill)
            .stroke(role::figure::pen())
            .stroke_width(line::HERO);
        sheet.ctx.rect(rx, y, bar_w * pct / 100.0, 92.0);
        sheet.label(
            &format!("{}", pct as i64),
            rx + bar_w * pct / 100.0 + 26.0,
            y + 22.0,
            54.0,
            role::figure::pen(),
            -1,
        );
    };
    draw_bar(&mut sheet, 68.0, 450.0, role::figure::orange());
    draw_bar(&mut sheet, 85.0, 230.0, role::figure::green());

    sheet.save(renderer, out);
}

// --- main ------------------------------------------------------------------------

fn main() {
    let mono_path = inputs::geist_mono();
    let sources = inputs::virtua_sources();
    let reg = sources.join("VirtuaGrotesk-Regular.ufo/glyphs");
    let bold = sources.join("VirtuaGrotesk-Bold.ufo/glyphs");

    let mut renderer = Renderer::new(W as u32, H as u32);
    let mono = load_family(&mut renderer, mono_path.to_str().unwrap());

    let outputs = OutputPaths::from_args();
    fig_semantic(
        &renderer,
        &mono,
        &reg,
        &outputs.blog("fig-semantic-grid.png"),
    );
    fig_no(&renderer, &mono, &reg, &outputs.blog("fig-system-no.png"));
    fig_ho(&renderer, &mono, &reg, &outputs.blog("fig-system-ho.png"));
    fig_weights(
        &renderer,
        &mono,
        &reg,
        &bold,
        &outputs.blog("fig-system-weights.png"),
    );
    fig_arabic(
        &renderer,
        &mono,
        &reg,
        &outputs.blog("fig-system-arabic.png"),
    );
}
