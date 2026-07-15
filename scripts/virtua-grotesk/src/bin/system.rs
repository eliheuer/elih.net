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

    const S: f64 = 0.84;
    let run: f64 = (cap_o.width + cap_h.width + n.width + o.width) * S;
    let content_h = (784.0 + 256.0) * S + 50.0;
    let bottom = FOOTER_RULE_Y + (HEADER_RULE_Y - FOOTER_RULE_Y - content_h) / 2.0;
    let f = Frame {
        s: S,
        x0: MARGIN + (W - 2.0 * MARGIN - run) / 2.0,
        baseline: bottom + 256.0 * S,
    };

    draw_grid(&mut sheet, &f, 784.0, -256.0);
    metric_lines(&mut sheet, &f, &[0.0, 576.0, 768.0, -256.0], &[784.0, -16.0]);

    let (x_o, x_h, x_n, x_lo) = (
        0.0,
        cap_o.width,
        cap_o.width + cap_h.width,
        cap_o.width + cap_h.width + n.width,
    );
    for (outline, ox) in [(&cap_o, x_o), (&cap_h, x_h), (&n, x_n), (&o, x_lo)] {
        annotate(&mut sheet, outline, S, f.x(ox), f.baseline);
    }

    // measured stroke dimensions
    sheet.dim_h(f.x(x_o + 48.0), f.x(x_o + 156.0), f.y(384.0), "108", green());
    sheet.dim_v(f.x(x_o + 424.0), f.y(684.0), f.y(784.0), "100", green(), true);
    sheet.dim_h(f.x(x_h + 80.0), f.x(x_h + 184.0), f.y(600.0), "104", green());
    sheet.dim_v(f.x(x_h + 384.0), f.y(360.0), f.y(456.0), "96", green(), true);
    sheet.dim_h(f.x(x_n + 64.0), f.x(x_n + 160.0), f.y(256.0), "96", green());
    sheet.dim_h(f.x(x_lo + 32.0), f.x(x_lo + 132.0), f.y(288.0), "100", green());

    // the key innovation, called out on the n's arch (y=500: off 8, on 2)
    correction_callout(
        &mut sheet,
        (f.x(x_n + 296.0), f.y(500.0)),
        (f.x(x_n + 240.0), f.y(676.0)),
        -1,
    );

    sheet.metric_tag("CAP / ASC 768 = 512+256", W - MARGIN, f.y(768.0), false, 1);
    sheet.metric_tag("X-HEIGHT 576 = 512+64", MARGIN, f.y(576.0), true, -1);
    sheet.metric_tag("BASELINE 0", MARGIN, f.y(0.0), true, -1);
    sheet.metric_tag("DESCENDER -256 = -(2^8)", MARGIN, f.y(-256.0), true, -1);
    sheet.metric_tag("OVERSHOOT +16", W - MARGIN, f.y(784.0), true, 1);
    sheet.metric_tag("OVERSHOOT -16", W - MARGIN, f.y(-16.0), false, 1);

    legend(&mut sheet, W - MARGIN, f.y(-150.0));

    sheet.frame(
        "THE LATIN SYSTEM / OHno",
        "VIRTUA GROTESK / EM 1024 = 2^10",
        "METRICS ON 64, STEMS ON 8, CURVES = STEM \u{b1} 4: EVERY LEVEL AT WORK",
    );
    sheet.save(renderer, out);
}

// --- fig-system-no -----------------------------------------------------------------

fn fig_no(renderer: &Renderer, mono: &str, reg: &std::path::Path, out: &std::path::Path) {
    let mut sheet = new_sheet(renderer, mono);

    let n = load_outline(reg, "n");
    let o = load_outline(reg, "o");

    const S: f64 = 1.35;
    let run = (n.width + o.width) * S;
    let f = Frame {
        s: S,
        x0: MARGIN + (W - 2.0 * MARGIN - run) / 2.0,
        baseline: 294.0,
    };

    draw_grid(&mut sheet, &f, 640.0, -64.0);
    metric_lines(&mut sheet, &f, &[0.0, 576.0], &[-16.0, 592.0]);

    annotate(&mut sheet, &n, S, f.x(0.0), f.baseline);
    annotate(&mut sheet, &o, S, f.x(n.width), f.baseline);

    sheet.dim_h(f.x(64.0), f.x(160.0), f.y(256.0), "96", green());
    sheet.dim_v(f.x(n.width + 304.0), f.y(500.0), f.y(592.0), "92", green(), true);
    sheet.dim_h(f.x(n.width + 32.0), f.x(n.width + 132.0), f.y(288.0), "100", green());

    // chamfer callout: n bottom-left corner (64,16)-(80,0)
    {
        let (cx, cy) = (f.x(72.0), f.y(8.0));
        sheet.ctx.no_fill().stroke(green()).stroke_width(PEN);
        sheet.ctx.line(cx - 14.0, cy - 14.0, cx - 90.0, cy - 90.0);
        sheet.label_padded("CHAMFER 16", cx - 104.0, cy - 116.0, DIM_TEXT, green(), 1);
    }

    sheet.metric_tag("X-HEIGHT 576", MARGIN, f.y(576.0), true, -1);
    sheet.metric_tag("BASELINE 0", MARGIN, f.y(0.0), true, -1);
    sheet.metric_tag("OVERSHOOT +16", W - MARGIN, f.y(592.0), true, 1);
    sheet.metric_tag("OVERSHOOT -16", W - MARGIN, f.y(-16.0), false, 1);

    advance_row(
        &mut sheet,
        &f,
        188.0,
        &[(0.0, n.width), (n.width, o.width)],
        &[(64.0, 528.0), (32.0, 584.0)],
    );

    legend(&mut sheet, W - MARGIN, f.y(400.0));

    sheet.frame(
        "THE LOWERCASE SYSTEM / no",
        "VIRTUA GROTESK / EM 1024 = 2^10",
        "STEM 96 = 3\u{b7}32 ON THE 32-GRID; CURVES 96\u{b1}4: THE CORRECTION GRID AT WORK",
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

    const S: f64 = 0.87;
    const PAIR_GAP: f64 = 120.0;
    let run = (rn.width + ro.width + bn.width + bo.width) * S + PAIR_GAP;
    let f = Frame {
        s: S,
        x0: MARGIN + (W - 2.0 * MARGIN - run) / 2.0,
        baseline: 404.0,
    };

    draw_grid(&mut sheet, &f, 640.0, -64.0);
    metric_lines(&mut sheet, &f, &[0.0, 576.0], &[-16.0, 592.0]);

    let x_rn = f.x(0.0);
    let x_ro = f.x(rn.width);
    let x_bn = f.x(rn.width + ro.width) + PAIR_GAP;
    let x_bo = x_bn + bn.width * S;

    annotate(&mut sheet, &rn, S, x_rn, f.baseline);
    annotate(&mut sheet, &ro, S, x_ro, f.baseline);
    annotate(&mut sheet, &bn, S, x_bn, f.baseline);
    annotate(&mut sheet, &bo, S, x_bo, f.baseline);

    // stems and curves, both weights, all measured
    sheet.dim_h(x_rn + 64.0 * S, x_rn + 160.0 * S, f.y(256.0), "96", green());
    sheet.dim_h(x_ro + 32.0 * S, x_ro + 132.0 * S, f.y(288.0), "100", green());
    sheet.dim_v(x_ro + 304.0 * S, f.y(500.0), f.y(592.0), "92", green(), true);
    sheet.dim_h(x_bn + 64.0 * S, x_bn + 256.0 * S, f.y(256.0), "192", green());
    sheet.dim_h(x_bo + 32.0 * S, x_bo + 228.0 * S, f.y(288.0), "196", green());
    sheet.dim_v(x_bo + 344.0 * S, f.y(452.0), f.y(592.0), "140", green(), true);

    // pair labels, above the grid
    let label_y = f.y(640.0) + 44.0;
    sheet.label("01 REGULAR / STEM 96", x_rn, label_y, LABEL_TEXT, green(), -1);
    sheet.label("02 BOLD / STEM 192 = 96\u{b7}2", x_bn, label_y, LABEL_TEXT, green(), -1);

    // the key innovation, called out on the Regular o's inner wall
    correction_callout(
        &mut sheet,
        (x_ro + 132.0 * S, f.y(288.0) - 40.0),
        (x_ro + 260.0 * S, f.y(-120.0)),
        -1,
    );

    sheet.metric_tag("X-HEIGHT 576", MARGIN, f.y(576.0), true, -1);
    sheet.metric_tag("BASELINE 0", MARGIN, f.y(0.0), true, -1);
    sheet.metric_tag("OVERSHOOT +16", W - MARGIN, f.y(592.0), true, 1);
    sheet.metric_tag("OVERSHOOT -16", W - MARGIN, f.y(-16.0), false, 1);

    legend(&mut sheet, W - MARGIN, label_y);

    sheet.frame(
        "TWO MASTERS, ONE SYSTEM / no no",
        "VIRTUA GROTESK / EM 1024 = 2^10",
        "THE BOLD IS THE SAME SYSTEM: STEM 192 = 96\u{b7}2, AND AGAIN CURVE 196 = STEM + 4",
    );
    sheet.save(renderer, out);
}

// --- fig-system-arabic: alef, beh, medial heh, right to left --------------------------

fn fig_arabic(renderer: &Renderer, mono: &str, reg: &std::path::Path, out: &std::path::Path) {
    let mut sheet = new_sheet(renderer, mono);

    let alef = load_outline(reg, "alef-ar");
    let beh = load_outline(reg, "beh-ar"); // components resolved: boat + dot
    let heh = load_outline(reg, "heh-ar.medi");

    const S: f64 = 0.85;
    let f = Frame {
        s: S,
        x0: MARGIN,
        baseline: 424.0,
    };

    draw_grid(&mut sheet, &f, 784.0, -304.0);
    metric_lines(&mut sheet, &f, &[0.0, 768.0], &[]);

    // three slots, read right to left: alef, beh, medial heh
    let slot_w = (W - 2.0 * MARGIN) / 3.0;
    let center = |i: f64| MARGIN + (i + 0.5) * slot_w;
    let x_alef = center(2.0) - alef.width * S / 2.0;
    let x_beh = center(1.0) - beh.width * S / 2.0;
    let x_heh = center(0.0) - heh.width * S / 2.0;

    annotate(&mut sheet, &alef, S, x_alef, f.baseline);
    annotate(&mut sheet, &beh, S, x_beh, f.baseline);
    annotate(&mut sheet, &heh, S, x_heh, f.baseline);

    // name labels, pinned under the header rule, numbered in reading order
    sheet.label("01 ALEF / U+0627", center(2.0), HEADER_RULE_Y - 46.0, LABEL_TEXT, green(), 0);
    sheet.label("02 BEH / U+0628", center(1.0), HEADER_RULE_Y - 46.0, LABEL_TEXT, green(), 0);
    sheet.label(
        "03 HEH / MEDIAL FORM",
        center(0.0),
        HEADER_RULE_Y - 46.0,
        LABEL_TEXT,
        green(),
        0,
    );

    // measured dimensions: alef stroke, beh boat stroke, the dot's diameter
    sheet.dim_h(x_alef + 64.0 * S, x_alef + 160.0 * S, f.y(384.0), "96", green());
    sheet.dim_v(x_beh + 296.0 * S, f.y(0.0), f.y(72.0), "72", green(), true);
    sheet.dim_h(x_beh + 42.0 * S, x_beh + 202.0 * S, f.y(-192.0), "160", green());

    // the beh's tail is dense with 2-grid work: call one out
    correction_callout(
        &mut sheet,
        (x_beh + 538.0 * S, f.y(322.0)),
        (x_beh + 400.0 * S, f.y(560.0)),
        -1,
    );

    sheet.metric_tag("ALEF / CAP 768", MARGIN, f.y(768.0), false, -1);
    sheet.metric_tag("BASELINE 0", MARGIN, f.y(0.0), true, -1);

    legend(&mut sheet, W - MARGIN, 240.0);

    sheet.frame(
        "THE ARABIC SYSTEM / READS RIGHT TO LEFT",
        "VIRTUA GROTESK / EM 1024 = 2^10",
        "SAME GRID, SAME RULES: THE ALEF IS PURE STRUCTURE; THE BEH'S CURVES ARE 2-UNIT CORRECTIONS",
    );
    sheet.save(renderer, out);
}


// --- fig-semantic-grid: the whole argument in one image -------------------------------
//
// Left: no, with the dimensions color-coded by layer (96 = machine green,
// 100 and 92 = the hand's +/-4 in red). Top right: why 96 and 100 mean
// different things — not their power-of-two decomposition (every integer
// has one) but their trailing zeros. Bottom right: the measured proof that
// Virtua-12M-0.8 learned the layers from geometry alone.

fn fig_semantic(renderer: &Renderer, mono: &str, reg: &std::path::Path, out: &std::path::Path) {
    let mut sheet = new_sheet(renderer, mono);

    let n = load_outline(reg, "n");
    let o = load_outline(reg, "o");

    // Zoomed crop with exact 8-multiple bounds, box sitting 24px off the
    // rules on all sides. 560 units tall / 840 wide -> the 8-unit
    // structure grid reads as crisp cells, the 2-unit correction grid as
    // fine lines between them.
    const V0: f64 = 48.0;
    const V1: f64 = 608.0;
    const U0: f64 = -8.0;
    const U1: f64 = 832.0;
    let box_bottom = FOOTER_RULE_Y + 24.0;
    let box_top = HEADER_RULE_Y - 24.0;
    let s_zoom = (box_top - box_bottom) / (V1 - V0);
    let f = Frame {
        s: s_zoom,
        x0: MARGIN - U0 * s_zoom,
        baseline: box_bottom - V0 * s_zoom,
    };
    let box_right = f.x(U1);

    // the two-layer grid, classified per line IN GLYPH COORDINATES so it
    // can never drift off the letter: majors every 8 units (structure),
    // minors every 2 (correction)
    {
        let ctx = &mut sheet.ctx;
        let weight = |q: f64| -> (Color, f64) {
            if q.rem_euclid(8.0) == 0.0 {
                (Color::rgb(0x40, 0x40, 0x40), 2.0) // major: the 8-unit structure grid
            } else {
                (Color::rgb(0x18, 0x18, 0x18), 0.75) // minor: the 2-unit correction grid
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

    // mask the crop spill (bg is solid)
    {
        let ctx = &mut sheet.ctx;
        ctx.fill(bg()).no_stroke();
        ctx.rect(0.0, 0.0, W, box_bottom);
        ctx.rect(0.0, box_top, W, H - box_top);
        ctx.rect(0.0, 0.0, MARGIN, H);
        ctx.rect(box_right, 0.0, W - box_right, H);
    }

    // dimension chains, color = layer
    sheet.dim_h(f.x(64.0), f.x(160.0), f.y(256.0), "96", green());
    sheet.dim_h(f.x(160.0), f.x(432.0), f.y(256.0), "272", gray());
    sheet.dim_h(f.x(432.0), f.x(528.0), f.y(256.0), "96", green());
    sheet.dim_h(f.x(n.width + 32.0), f.x(n.width + 132.0), f.y(288.0), "100", red());

    // key coordinates + the o wall's handle lengths, placed by hand so
    // nothing straddles the crop edge
    sheet.label_padded("(32,288)", f.x(n.width + 32.0) - 16.0, f.y(288.0) + 26.0, SMALL_TEXT, gray(), 1);
    sheet.label_padded("128", f.x(n.width + 132.0) + 14.0, f.y(352.0) - 7.0, SMALL_TEXT, purple(), -1);
    sheet.label_padded("128", f.x(n.width + 132.0) + 14.0, f.y(224.0) - 7.0, SMALL_TEXT, purple(), -1);

    // ---- right column: one text size, tags under the bit rows ----------------------
    let rx = box_right + 56.0;
    let body = DIM_TEXT; // 30px everywhere in the column

    sheet.label("Every integer is a sum of", rx, 1158.0, body, gray(), -1);
    sheet.label("powers of two. The meaning", rx, 1114.0, body, gray(), -1);
    sheet.label("is the trailing zeros:", rx, 1070.0, body, gray(), -1);

    let bit_row = |sheet: &mut Sheet, value: u32, y0: f64, color: Color, tag: &str| {
        let cell = 50.0;
        let gap = 8.0;
        sheet.label(&value.to_string(), rx + 56.0, y0 + 12.0, body, color, 1);
        for b in 0..7u32 {
            let bit = (value >> (6 - b)) & 1;
            let x = rx + 72.0 + b as f64 * (cell + gap);
            if bit == 1 {
                sheet.ctx.fill(color).stroke(color).stroke_width(PEN_LIGHT);
                sheet.ctx.rect(x, y0, cell, cell);
                sheet.label("1", x + cell / 2.0, y0 + cell * 0.28, body, bg(), 0);
            } else {
                sheet.ctx.no_fill().stroke(dim_color()).stroke_width(PEN_LIGHT);
                sheet.ctx.rect(x, y0, cell, cell);
                sheet.label("0", x + cell / 2.0, y0 + cell * 0.28, body, dim_color(), 0);
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
    bit_row(&mut sheet, 96, 968.0, green(), "on 32: structure");
    bit_row(&mut sheet, 100, 830.0, red(), "on 4: correction");
    sheet.label("100 = 96 + 4: the curve's", rx, 700.0, body, red(), -1);
    sheet.label("optical correction", rx, 656.0, body, red(), -1);

    sheet.label("Points on the 8-unit grid", rx, 560.0, body, gray(), -1);
    sheet.label("(held-out Bolds, raw output)", rx, 516.0, SMALL_TEXT, gray(), -1);
    let bar_max = W - MARGIN - rx - 84.0;
    let bar = |sheet: &mut Sheet, y: f64, frac: f64, color: Color, label: &str, pct: &str| {
        sheet.label(label, rx, y + 52.0, body, color, -1);
        let w = (bar_max * frac / 0.85).max(6.0);
        sheet.ctx.fill(fill_strong(color)).stroke(color).stroke_width(PEN_LIGHT);
        sheet.ctx.rect(rx, y, w, 36.0);
        sheet.label(pct, rx + w + 16.0, y + 8.0, body, color, -1);
    };
    bar(&mut sheet, 408.0, 0.0625, dim_color(), "chance on the 2-grid", "6%");
    bar(&mut sheet, 304.0, 0.68, red(), "Virtua-12M-0.8", "68%");
    bar(&mut sheet, 200.0, 0.85, green(), "human sources", "85%");

    legend(&mut sheet, W - MARGIN, 136.0);

    sheet.frame(
        "Multi-layer semantic powers-of-two grid system",
        "Virtua Grotesk / Virtua-12M-0.8",
        "elih.net/blog/virtua-grotesk",
    );
    sheet.save(renderer, out);
}

// --- main ------------------------------------------------------------------------

fn main() {
    let home = std::env::var("HOME").unwrap();
    let mono_path = format!("{home}/GH/repos/google-fonts/ofl/geistmono/GeistMono[wght].ttf");
    let sources = std::path::PathBuf::from(&home).join("GH/repos/virtua-grotesk/sources");
    let reg = sources.join("VirtuaGrotesk-Regular.ufo/glyphs");
    let bold = sources.join("VirtuaGrotesk-Bold.ufo/glyphs");

    let mut renderer = Renderer::new(W as u32, H as u32);
    let mono = load_family(&mut renderer, &mono_path);

    let here = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let post = here
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("src/content/blog/virtua-grotesk");

    fig_semantic(&renderer, &mono, &reg, &post.join("fig-semantic-grid.png"));
    fig_ohno(&renderer, &mono, &reg, &post.join("fig-system-ohno.png"));
    fig_no(&renderer, &mono, &reg, &post.join("fig-system-no.png"));
    fig_weights(&renderer, &mono, &reg, &bold, &post.join("fig-system-weights.png"));
    fig_arabic(&renderer, &mono, &reg, &post.join("fig-system-arabic.png"));
}
