//! Design-system dimension sheets for the Virtua Grotesk post, §03 — the
//! figures that replace the measurement tables:
//!
//!   fig-system-ohno.png    : the full Latin system on "OHno"
//!   fig-system-no.png      : the lowercase system on "no", zoomable
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

// --- fig-system-ohno ---------------------------------------------------------------

fn fig_ohno(renderer: &Renderer, mono: &str, reg: &std::path::Path, out: &std::path::Path) {
    let mut sheet = new_sheet(renderer, mono);

    let cap_o = load_outline(reg, "O");
    let cap_h = load_outline(reg, "H");
    let n = load_outline(reg, "n");
    let o = load_outline(reg, "o");

    const S: f64 = 0.86;
    let run: f64 = (cap_o.width + cap_h.width + n.width + o.width) * S;
    let f = Frame {
        s: S,
        x0: MARGIN + (W - 2.0 * MARGIN - run) / 2.0,
        baseline: 250.0,
    };

    metric_lines(&mut sheet, &f, &[0.0, 576.0, 768.0], &[784.0, -16.0]);

    let (x_o, x_h, x_n, x_lo) = (
        0.0,
        cap_o.width,
        cap_o.width + cap_h.width,
        cap_o.width + cap_h.width + n.width,
    );
    for (outline, ox, fill) in [
        (&cap_o, x_o, role::figure::red()),
        (&cap_h, x_h, role::figure::orange()),
        (&n, x_n, role::figure::yellow()),
        (&o, x_lo, role::figure::green()),
    ] {
        draw_figure_glyph(&mut sheet, outline, S, f.x(ox), f.baseline, fill);
    }

    // measured stroke dimensions
    sheet.dim_h(
        f.x(x_o + 48.0),
        f.x(x_o + 156.0),
        f.y(384.0),
        "108",
        role::figure::pen(),
    );
    sheet.dim_v(
        f.x(x_o + 424.0),
        f.y(684.0),
        f.y(784.0),
        "100",
        role::figure::pen(),
        true,
    );
    sheet.dim_h(
        f.x(x_h + 80.0),
        f.x(x_h + 184.0),
        f.y(600.0),
        "104",
        role::figure::pen(),
    );
    sheet.dim_v(
        f.x(x_h + 384.0),
        f.y(360.0),
        f.y(456.0),
        "96",
        role::figure::pen(),
        true,
    );
    sheet.dim_h(
        f.x(x_n + 64.0),
        f.x(x_n + 160.0),
        f.y(256.0),
        "96",
        role::figure::pen(),
    );
    sheet.dim_h(
        f.x(x_lo + 32.0),
        f.x(x_lo + 132.0),
        f.y(288.0),
        "100",
        role::figure::pen(),
    );
    sheet.save(renderer, out);
}

// --- fig-system-no -----------------------------------------------------------------

fn fig_no(renderer: &Renderer, mono: &str, reg: &std::path::Path, out: &std::path::Path) {
    let mut sheet = new_sheet(renderer, mono);

    let n = load_outline(reg, "n");
    let o = load_outline(reg, "o");

    const S: f64 = 1.62;
    let run = (n.width + o.width) * S;
    let f = Frame {
        s: S,
        x0: MARGIN + (W - 2.0 * MARGIN - run) / 2.0,
        baseline: 214.0,
    };
    metric_lines(&mut sheet, &f, &[0.0, 576.0], &[-16.0, 592.0]);

    draw_figure_glyph(&mut sheet, &n, S, f.x(0.0), f.baseline, role::figure::red());
    draw_figure_glyph(
        &mut sheet,
        &o,
        S,
        f.x(n.width),
        f.baseline,
        role::figure::green(),
    );

    sheet.dim_h(f.x(64.0), f.x(160.0), f.y(256.0), "96", role::figure::pen());
    sheet.dim_v(
        f.x(n.width + 304.0),
        f.y(500.0),
        f.y(592.0),
        "92",
        role::figure::pen(),
        true,
    );
    sheet.dim_h(
        f.x(n.width + 32.0),
        f.x(n.width + 132.0),
        f.y(288.0),
        "100",
        role::figure::pen(),
    );
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

fn fig_semantic(renderer: &Renderer, mono: &str, reg: &std::path::Path, out: &std::path::Path) {
    let mut sheet = new_sheet(renderer, mono);

    let n = load_outline(reg, "n");
    let o = load_outline(reg, "o");

    // FRAMELESS EXPERIMENT: no rules, no title/caption rows. The grid runs
    // to the 64px margin on all four sides; attribution lives in the info
    // column. Window bounds stay exact 8-multiples.
    const V0: f64 = 48.0;
    const V1: f64 = 608.0; // 560 units tall
    const U0: f64 = -8.0;
    const U1: f64 = 760.0; // 768 units wide = 96 cells of 8
    let box_bottom = MARGIN;
    let box_top = H - MARGIN;
    let s_zoom = (box_top - box_bottom) / (V1 - V0);
    let f = Frame {
        s: s_zoom,
        x0: MARGIN - U0 * s_zoom,
        baseline: box_bottom - V0 * s_zoom,
    };
    let box_right = f.x(U1);

    // the two-layer grid: majors every 8 (structure), minors every 2
    {
        let ctx = &mut sheet.ctx;
        let weight = |q: f64| -> (Color, f64) {
            if q.rem_euclid(8.0) == 0.0 {
                (color::gray_650(), line::THIN)
            } else {
                (color::gray_925(), line::HAIRLINE)
            }
        };
        let mut u = (U0 / 2.0).ceil() * 2.0;
        while u <= U1 {
            let (color, wpen) = weight(u);
            ctx.no_fill().stroke(color).stroke_width(wpen);
            ctx.line(f.x0 + u * s_zoom, box_bottom, f.x0 + u * s_zoom, box_top);
            u += 2.0;
        }
        let mut v = (V0 / 2.0).ceil() * 2.0;
        while v <= V1 {
            let (color, wpen) = weight(v);
            ctx.no_fill().stroke(color).stroke_width(wpen);
            ctx.line(MARGIN, f.y(v), box_right, f.y(v));
            v += 2.0;
        }
    }

    for (outline, ox) in [(&n, 0.0), (&o, n.width)] {
        draw_body(&mut sheet, outline, s_zoom, f.x(ox), f.baseline);
        draw_points(&mut sheet, outline, s_zoom, f.x(ox), f.baseline);
    }
    handle_labels(&mut sheet, &n, s_zoom, f.x(0.0), f.baseline);

    // mask the crop spill
    {
        let ctx = &mut sheet.ctx;
        ctx.fill(role::canvas::background()).no_stroke();
        ctx.rect(0.0, 0.0, W, box_bottom);
        ctx.rect(0.0, box_top, W, H - box_top);
        ctx.rect(0.0, 0.0, MARGIN, H);
        ctx.rect(box_right, 0.0, W - box_right, H);
    }

    // dimension chains, color = layer
    sheet.dim_h(f.x(64.0), f.x(160.0), f.y(256.0), "96", green());
    sheet.dim_h(f.x(160.0), f.x(432.0), f.y(256.0), "272", gray());
    sheet.dim_h(f.x(432.0), f.x(528.0), f.y(256.0), "96", green());
    sheet.dim_h(
        f.x(n.width + 32.0),
        f.x(n.width + 132.0),
        f.y(288.0),
        "100",
        red(),
    );

    sheet.label_padded(
        "(32,288)",
        f.x(n.width + 32.0) - 16.0,
        f.y(288.0) + 26.0,
        SMALL_TEXT,
        gray(),
        1,
    );
    sheet.label_padded(
        "128",
        f.x(n.width + 132.0) + 14.0,
        f.y(352.0) - 7.0,
        SMALL_TEXT,
        purple(),
        -1,
    );
    sheet.label_padded(
        "128",
        f.x(n.width + 132.0) + 14.0,
        f.y(224.0) - 7.0,
        SMALL_TEXT,
        purple(),
        -1,
    );

    // ---- info column --------------------------------------------------------------
    let rx = box_right + 56.0;
    let body = DIM_TEXT;

    sheet.label("The self-labeling grid,", rx, 1216.0, body, green(), -1);
    sheet.label("built on powers of two", rx, 1172.0, body, green(), -1);

    sheet.label("Every integer is a sum of", rx, 1084.0, body, gray(), -1);
    sheet.label("powers of two. The meaning", rx, 1040.0, body, gray(), -1);
    sheet.label("is the trailing zeros:", rx, 996.0, body, gray(), -1);

    let bit_row = |sheet: &mut Sheet, value: u32, y0: f64, color: Color, tag: &str| {
        let cell = 44.0;
        let gap = 6.0;
        sheet.label(&value.to_string(), rx + 56.0, y0 + 10.0, body, color, 1);
        for b in 0..7u32 {
            let bit = (value >> (6 - b)) & 1;
            let x = rx + 72.0 + b as f64 * (cell + gap);
            if bit == 1 {
                sheet.ctx.fill(color).stroke(color).stroke_width(PEN_LIGHT);
                sheet.ctx.rect(x, y0, cell, cell);
                sheet.label(
                    "1",
                    x + cell / 2.0,
                    y0 + cell * 0.26,
                    28.0,
                    role::canvas::background(),
                    0,
                );
            } else {
                sheet
                    .ctx
                    .no_fill()
                    .stroke(role::annotation::dimensions())
                    .stroke_width(PEN_LIGHT);
                sheet.ctx.rect(x, y0, cell, cell);
                sheet.label(
                    "0",
                    x + cell / 2.0,
                    y0 + cell * 0.26,
                    28.0,
                    role::annotation::dimensions(),
                    0,
                );
            }
        }
        let zeros = value.trailing_zeros();
        let x_start = rx + 72.0 + (7 - zeros) as f64 * (cell + gap);
        let x_end = rx + 72.0 + 7.0 * (cell + gap) - gap;
        sheet.ctx.no_fill().stroke(color).stroke_width(PEN);
        sheet.ctx.line(x_start, y0 - 12.0, x_end, y0 - 12.0);
        sheet.ctx.line(x_start, y0 - 12.0, x_start, y0 - 4.0);
        sheet.ctx.line(x_end, y0 - 12.0, x_end, y0 - 4.0);
        sheet.label(tag, x_end, y0 - 56.0, body, color, 1);
    };
    bit_row(&mut sheet, 96, 888.0, green(), "on 32: structure");
    bit_row(&mut sheet, 100, 744.0, red(), "on 4: correction");
    sheet.label("100 = 96 + 4: the curve's", rx, 616.0, body, red(), -1);
    sheet.label("optical correction", rx, 572.0, body, red(), -1);

    sheet.label("Points on the 8-unit grid", rx, 484.0, body, gray(), -1);
    sheet.label(
        "(held-out Bolds, raw output)",
        rx,
        440.0,
        SMALL_TEXT,
        gray(),
        -1,
    );
    let bar_max = W - MARGIN - rx - 71.0;
    let bar = |sheet: &mut Sheet, y: f64, frac: f64, color: Color, label: &str, pct: &str| {
        sheet.label(label, rx, y + 50.0, body, color, -1);
        let w = (bar_max * frac / 0.85).max(6.0);
        sheet
            .ctx
            .fill(fill_strong(color))
            .stroke(color)
            .stroke_width(PEN_LIGHT);
        sheet.ctx.rect(rx, y, w, 34.0);
        sheet.label(pct, rx + w + 16.0, y + 6.0, body, color, -1);
    };
    bar(
        &mut sheet,
        336.0,
        0.0625,
        role::annotation::dimensions(),
        "chance on the 2-grid",
        "6%",
    );
    bar(&mut sheet, 240.0, 0.68, red(), "Virtua-12M-v0.1", "68%");
    bar(&mut sheet, 144.0, 0.85, green(), "human sources", "85%");

    legend(&mut sheet, box_right - 16.0, 84.0);

    // attribution, bottom of the column
    sheet.label(
        "Virtua Grotesk / Virtua-12M-v0.1",
        rx,
        64.0 + 36.0,
        SMALL_TEXT,
        green(),
        -1,
    );
    sheet.label(
        "elih.net/blog/virtua-grotesk",
        rx,
        64.0,
        SMALL_TEXT,
        green(),
        -1,
    );

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
    fig_ohno(&renderer, &mono, &reg, &outputs.blog("fig-system-ohno.png"));
    fig_no(&renderer, &mono, &reg, &outputs.blog("fig-system-no.png"));
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
