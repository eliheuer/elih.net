//! The §10 proof figures for the Virtua Grotesk post, four sheets in the
//! house dimension-sheet language:
//!
//!   fig-fractions.png : one stem, two ems — 96/1024 terminates, 96/1000 never
//!   fig-midpoint.png  : de Casteljau at t=1/2 — midpoints all the way down
//!   fig-ladder.png    : how far can an em halve — 1024 vs 1000 vs 729 vs 700
//!   fig-bits.png      : grid level = trailing zeros (2-adic valuation / CTZ)
//!
//! Run from this directory:
//!
//!     cargo run --release --bin proofs
//!
//! Writes into ../../src/content/blog/virtua-grotesk/.

use designbot::prelude::*;
use designbot_render::Renderer;

const W: f64 = 2520.0;
const H: f64 = 1320.0;
const MARGIN: f64 = 64.0;
const HEADER_RULE_Y: f64 = 1210.0;
const FOOTER_RULE_Y: f64 = 110.0;

fn bg() -> Color {
    Color::rgb(0x10, 0x10, 0x10)
}
fn grid_16() -> Color {
    Color::rgb(0x1e, 0x1e, 0x1e)
}
fn grid_64() -> Color {
    Color::rgb(0x3a, 0x3a, 0x3a)
}
fn dim() -> Color {
    Color::rgb(0x4a, 0x4a, 0x4a)
}
fn curve() -> Color {
    Color::rgb(230, 230, 230)
}
fn green() -> Color {
    Color::rgb(0x15, 0xc4, 0x74)
}
fn red() -> Color {
    Color::rgb(0xff, 0x45, 0x35)
}
fn purple() -> Color {
    Color::rgb(0x8c, 0x6c, 0xff)
}
fn gray() -> Color {
    Color::rgb(0x8a, 0x8a, 0x8a)
}

// --- sfnt family-name reader (same as og.rs) ---------------------------------

fn read_u16(d: &[u8], o: usize) -> u16 {
    u16::from_be_bytes([d[o], d[o + 1]])
}
fn read_u32(d: &[u8], o: usize) -> u32 {
    u32::from_be_bytes([d[o], d[o + 1], d[o + 2], d[o + 3]])
}
fn find_table(d: &[u8], tag: &[u8; 4]) -> Option<usize> {
    let n = read_u16(d, 4) as usize;
    (0..n)
        .map(|i| 12 + i * 16)
        .find(|&r| &d[r..r + 4] == tag)
        .map(|r| read_u32(d, r + 8) as usize)
}
fn load_family(renderer: &mut Renderer, path: &str) -> String {
    let data = std::fs::read(path).unwrap_or_else(|e| panic!("read {path}: {e}"));
    renderer
        .load_font(path)
        .unwrap_or_else(|e| panic!("load {path}: {e:?}"));
    let name = find_table(&data, b"name").expect("no name table");
    let count = read_u16(&data, name + 2) as usize;
    let string_off = name + read_u16(&data, name + 4) as usize;
    for want in [16u16, 1] {
        for i in 0..count {
            let rec = name + 6 + i * 12;
            if read_u16(&data, rec) == 3 && read_u16(&data, rec + 6) == want {
                let len = read_u16(&data, rec + 8) as usize;
                let off = string_off + read_u16(&data, rec + 10) as usize;
                let units: Vec<u16> = data[off..off + len]
                    .chunks_exact(2)
                    .map(|c| u16::from_be_bytes([c[0], c[1]]))
                    .collect();
                return String::from_utf16_lossy(&units);
            }
        }
    }
    panic!("no Windows family name in {path}");
}

// --- drawing ------------------------------------------------------------------

struct Sheet<'a> {
    ctx: Canvas,
    renderer: &'a Renderer,
    mono: String,
}

impl Sheet<'_> {
    fn mono_width(&self, txt: &str, size: f64) -> f64 {
        self.renderer.text_width(txt, Some(&self.mono), size, &[])
    }

    fn label(&mut self, txt: &str, x: f64, y: f64, size: f64, color: Color, align: i8) {
        let w = self.mono_width(txt, size);
        let x = match align {
            -1 => x,
            0 => x - w / 2.0,
            _ => x - w,
        };
        self.ctx
            .font(&self.mono)
            .clear_font_variations()
            .font_size(size)
            .fill(color)
            .text_align(TextAlign::Left)
            .text(txt, x, y);
    }

    fn label_padded(&mut self, txt: &str, x: f64, y: f64, size: f64, color: Color, align: i8) {
        let w = self.mono_width(txt, size);
        let pad = 10.0;
        let x0 = match align {
            -1 => x,
            0 => x - w / 2.0,
            _ => x - w,
        };
        self.ctx.fill(bg()).no_stroke();
        self.ctx
            .rect(x0 - pad, y - 0.28 * size, w + 2.0 * pad, 1.3 * size);
        self.label(txt, x0, y, size, color, -1);
    }

    /// Header/footer rules and the standard captions.
    fn frame(&mut self, title: &str, caption: &str) {
        self.ctx.stroke(green()).stroke_width(2.5).no_fill();
        self.ctx.line(MARGIN, HEADER_RULE_Y, W - MARGIN, HEADER_RULE_Y);
        self.ctx.line(MARGIN, FOOTER_RULE_Y, W - MARGIN, FOOTER_RULE_Y);
        self.label(title, MARGIN, HEADER_RULE_Y + 24.0, 30.0, green(), -1);
        self.label(
            "VIRTUA GROTESK / EM 1024 = 2^10",
            W - MARGIN,
            HEADER_RULE_Y + 24.0,
            30.0,
            green(),
            1,
        );
        self.label(caption, MARGIN, 64.0, 30.0, green(), -1);
        self.label(
            "GITHUB.COM/ELIHEUER/VIRTUA-GROTESK",
            W - MARGIN,
            64.0,
            30.0,
            green(),
            1,
        );
    }

    fn save(&self, renderer: &Renderer, out: &std::path::Path) {
        std::fs::create_dir_all(out.parent().unwrap()).unwrap();
        renderer
            .render_to_png(&self.ctx, out.to_str().unwrap())
            .unwrap();
        println!("wrote {}", out.display());
    }
}

fn new_sheet<'a>(renderer: &'a Renderer, mono: &str) -> Sheet<'a> {
    let mut sheet = Sheet {
        ctx: Canvas::new(W, H),
        renderer,
        mono: mono.to_string(),
    };
    sheet.ctx.background(bg());
    sheet
}

// --- fig-fractions ------------------------------------------------------------

/// One row of bit cells. `bits` as 0/1, `color` for the 1-cells.
fn bit_row(sheet: &mut Sheet, x0: f64, y0: f64, cell: f64, gap: f64, bits: &[u8], color: Color) {
    for (i, b) in bits.iter().enumerate() {
        let x = x0 + i as f64 * (cell + gap);
        if *b == 1 {
            sheet.ctx.fill(color).stroke(color).stroke_width(2.0);
            sheet.ctx.rect(x, y0, cell, cell);
            sheet.label("1", x + cell / 2.0, y0 + cell * 0.28, cell * 0.6, bg(), 0);
        } else {
            sheet.ctx.no_fill().stroke(dim()).stroke_width(2.0);
            sheet.ctx.rect(x, y0, cell, cell);
            sheet.label("0", x + cell / 2.0, y0 + cell * 0.28, cell * 0.6, dim(), 0);
        }
    }
}

fn fig_fractions(renderer: &Renderer, mono: &str, out: &std::path::Path) {
    let mut sheet = new_sheet(renderer, mono);

    // 96/1000 = 12/125: first fractional bits, computed by long division.
    let mut num = 12u32;
    let mut bits_b = Vec::new();
    for _ in 0..40 {
        num *= 2;
        if num >= 125 {
            bits_b.push(1u8);
            num -= 125;
        } else {
            bits_b.push(0u8);
        }
    }
    let bits_a: [u8; 5] = [0, 0, 0, 1, 1]; // 3/32 = 0.00011

    let cell = 40.0;
    let gap = 8.0;
    let prefix_size = 44.0;

    // row A: the binary em
    let x0_block = 238.0;
    let ya = 808.0;
    sheet.label("96 / 1024", x0_block, ya + 180.0, 36.0, green(), -1);
    sheet.label("= 3/32, THE STEM OVER THE BINARY EM", x0_block + 260.0, ya + 180.0, 26.0, gray(), -1);
    sheet.label("0.", x0_block, ya + 10.0, prefix_size, curve(), -1);
    let x0 = x0_block + 60.0;
    bit_row(&mut sheet, x0, ya, cell, gap, &bits_a, green());
    let end_x = x0 + bits_a.len() as f64 * (cell + gap) + 30.0;
    sheet.label("EXACT AFTER 5 BITS", end_x, ya + 10.0, 30.0, green(), -1);
    sheet.label("= 0.09375, HELD VERBATIM", x0_block, ya - 70.0, 26.0, gray(), -1);

    // row B: the decimal em
    let yb = 388.0;
    sheet.label("96 / 1000", x0_block, yb + 180.0, 36.0, red(), -1);
    sheet.label("= 12/125, THE SAME STEM OVER THE DECIMAL EM", x0_block + 260.0, yb + 180.0, 26.0, gray(), -1);
    sheet.label("0.", x0_block, yb + 10.0, prefix_size, curve(), -1);
    bit_row(&mut sheet, x0, yb, cell, gap, &bits_b, red());
    let end_x = x0 + bits_b.len() as f64 * (cell + gap) + 20.0;
    sheet.label("\u{2026}", end_x, yb + 10.0, prefix_size, red(), -1);
    sheet.label(
        "REPEATS FOREVER, PERIOD 100 BITS. A 64-BIT FLOAT HOLDS 0.09600000000000000200",
        x0_block,
        yb - 70.0,
        26.0,
        red(),
        -1,
    );

    sheet.frame(
        "ONE STEM, TWO EMS",
        "A BINARY MACHINE WRITES 96/1024 DOWN EXACTLY; 96/1000 IT CAN ONLY APPROXIMATE",
    );
    sheet.save(renderer, out);
}

// --- fig-midpoint ---------------------------------------------------------------

fn fig_midpoint(renderer: &Renderer, mono: &str, out: &std::path::Path) {
    let mut sheet = new_sheet(renderer, mono);

    // panel: font units x 0..640, y 0..512 at S px/unit
    const S: f64 = 1.6;
    const PX: f64 = 460.0; // canvas x of unit x=0, block centered
    const PY: f64 = 248.0; // canvas y of unit y=0, panel centered between the rules
    let cx = |ux: f64| PX + ux * S;
    let cy = |uy: f64| PY + uy * S;
    let (ux1, uy1) = (640.0, 512.0);

    // grid: 16 faint, 64 bright
    for (step, color, width) in [(16.0, grid_16(), 1.5), (64.0, grid_64(), 2.5)] {
        sheet.ctx.no_fill().stroke(color).stroke_width(width);
        let mut u = 0.0;
        while u <= ux1 {
            sheet.ctx.line(cx(u), cy(0.0), cx(u), cy(uy1));
            u += step;
        }
        let mut u = 0.0;
        while u <= uy1 {
            sheet.ctx.line(cx(0.0), cy(u), cx(ux1), cy(u));
            u += step;
        }
    }

    // the arch, control points on the 64-grid
    let p: [(f64, f64); 4] = [(64.0, 128.0), (192.0, 448.0), (448.0, 448.0), (576.0, 128.0)];
    let mid = |a: (f64, f64), b: (f64, f64)| ((a.0 + b.0) / 2.0, (a.1 + b.1) / 2.0);
    let m01 = mid(p[0], p[1]);
    let m12 = mid(p[1], p[2]);
    let m23 = mid(p[2], p[3]);
    let mm0 = mid(m01, m12);
    let mm1 = mid(m12, m23);
    let c = mid(mm0, mm1);

    // control polygon
    sheet.ctx.no_fill().stroke(gray()).stroke_width(2.5);
    for w in p.windows(2) {
        sheet.ctx.line(cx(w[0].0), cy(w[0].1), cx(w[1].0), cy(w[1].1));
    }
    // round-1 and round-2 construction lines
    sheet.ctx.stroke(green()).stroke_width(2.5);
    sheet.ctx.line(cx(m01.0), cy(m01.1), cx(m12.0), cy(m12.1));
    sheet.ctx.line(cx(m12.0), cy(m12.1), cx(m23.0), cy(m23.1));
    sheet.ctx.stroke(purple()).stroke_width(2.5);
    sheet.ctx.line(cx(mm0.0), cy(mm0.1), cx(mm1.0), cy(mm1.1));

    // the curve itself
    {
        let mut path = kurbo::BezPath::new();
        path.move_to((cx(p[0].0), cy(p[0].1)));
        path.curve_to(
            (cx(p[1].0), cy(p[1].1)),
            (cx(p[2].0), cy(p[2].1)),
            (cx(p[3].0), cy(p[3].1)),
        );
        sheet.ctx.no_fill().stroke(curve()).stroke_width(3.5);
        sheet.ctx.draw_path(path);
    }

    // markers, knocked out with bg
    let mut dot = |pt: (f64, f64), color: Color, r: f64, square: bool, ctx_sheet: &mut Sheet| {
        ctx_sheet.ctx.fill(bg()).stroke(color).stroke_width(2.5);
        if square {
            ctx_sheet.ctx.rect(cx(pt.0) - r, cy(pt.1) - r, 2.0 * r, 2.0 * r);
        } else {
            ctx_sheet.ctx.oval(cx(pt.0) - r, cy(pt.1) - r, 2.0 * r, 2.0 * r);
        }
    };
    for q in p {
        dot(q, gray(), 9.0, true, &mut sheet);
    }
    for q in [m01, m12, m23] {
        dot(q, green(), 9.0, false, &mut sheet);
    }
    for q in [mm0, mm1] {
        dot(q, purple(), 9.0, false, &mut sheet);
    }
    dot(c, red(), 11.0, false, &mut sheet);

    // coordinate labels
    let coord = |v: (f64, f64)| format!("({},{})", v.0, v.1);
    sheet.label_padded(&coord(p[0]), cx(p[0].0) - 20.0, cy(p[0].1) - 50.0, 24.0, gray(), -1);
    sheet.label_padded(&coord(p[1]), cx(p[1].0) - 60.0, cy(p[1].1) + 30.0, 24.0, gray(), -1);
    sheet.label_padded(&coord(p[2]), cx(p[2].0) - 60.0, cy(p[2].1) + 30.0, 24.0, gray(), -1);
    sheet.label_padded(&coord(p[3]), cx(p[3].0) - 60.0, cy(p[3].1) - 50.0, 24.0, gray(), -1);
    sheet.label_padded(&coord(m01), cx(m01.0) - 200.0, cy(m01.1), 24.0, green(), -1);
    sheet.label_padded(&coord(m12), cx(m12.0) - 60.0, cy(m12.1) + 62.0, 24.0, green(), -1);
    sheet.label_padded(&coord(m23), cx(m23.0) + 30.0, cy(m23.1), 24.0, green(), -1);
    sheet.label_padded(&coord(mm0), cx(mm0.0) - 210.0, cy(mm0.1) + 26.0, 24.0, purple(), -1);
    sheet.label_padded(&coord(mm1), cx(mm1.0) + 40.0, cy(mm1.1) + 26.0, 24.0, purple(), -1);
    sheet.label_padded(&coord(c), cx(c.0) - 55.0, cy(c.1) - 56.0, 24.0, red(), -1);

    // legend, right of the panel
    let lx = cx(ux1) + 90.0;
    let rows: [(&str, Color, bool); 4] = [
        ("CONTROL POINTS · ON 64", gray(), true),
        ("ROUND 1 MIDPOINTS · ON 32", green(), false),
        ("ROUND 2 MIDPOINTS · ON 16", purple(), false),
        ("SPLIT POINT · ON 16", red(), false),
    ];
    for (i, (txt, color, square)) in rows.iter().enumerate() {
        let y = 960.0 - i as f64 * 60.0;
        sheet.ctx.fill(bg()).stroke(*color).stroke_width(2.5);
        if *square {
            sheet.ctx.rect(lx, y - 2.0, 18.0, 18.0);
        } else {
            sheet.ctx.oval(lx, y - 2.0, 18.0, 18.0);
        }
        sheet.label(txt, lx + 36.0, y, 26.0, *color, -1);
    }
    sheet.label("A MIDPOINT OF TWO MULTIPLES OF 2^K", lx, 640.0, 24.0, gray(), -1);
    sheet.label("IS A MULTIPLE OF 2^(K-1):", lx, 604.0, 24.0, gray(), -1);
    sheet.label("EACH ROUND DESCENDS ONE RUNG,", lx, 568.0, 24.0, gray(), -1);
    sheet.label("AND THE LADDER HAS TEN.", lx, 532.0, 24.0, gray(), -1);

    sheet.frame(
        "DE CASTELJAU AT T = 1/2",
        "SPLITTING A CURVE IS MIDPOINTS ALL THE WAY DOWN, AND MIDPOINTS OF DYADIC POINTS ARE DYADIC",
    );
    sheet.save(renderer, out);
}

// --- fig-ladder -----------------------------------------------------------------

fn fig_ladder(renderer: &Renderer, mono: &str, out: &std::path::Path) {
    let mut sheet = new_sheet(renderer, mono);

    let col_x = [387.0, 969.0, 1551.0, 2133.0];
    let titles = ["EM 1024", "EM 1000", "EM 729 = 3^6", "EM 700"];
    let y_top = 1000.0;
    let dy = 72.0;
    let box_w = 180.0;
    let box_h = 46.0;

    struct Chain<'a> {
        values: Vec<u32>,
        div: &'a str,
        color: Color,
        dead_end: Option<&'a str>,
        verdict: (&'a str, Color),
    }
    let chains = [
        Chain {
            values: vec![1024, 512, 256, 128, 64, 32, 16, 8, 4, 2, 1],
            div: "\u{f7}2",
            color: green(),
            dead_end: None,
            verdict: ("TEN RUNGS, EM TO UNIT", green()),
        },
        Chain {
            values: vec![1000, 500, 250, 125],
            div: "\u{f7}2",
            color: gray(),
            dead_end: Some("125 IS ODD \u{b7} DEAD END"),
            verdict: ("THREE RUNGS", red()),
        },
        Chain {
            values: vec![729, 243, 81, 27, 9, 3, 1],
            div: "\u{f7}3",
            color: gray(),
            dead_end: None,
            verdict: ("SIX RUNGS, BUT 1/3 IS NO FLOAT", red()),
        },
        Chain {
            values: vec![700, 350, 175],
            div: "\u{f7}2",
            color: gray(),
            dead_end: Some("175 IS ODD \u{b7} DEAD END"),
            verdict: ("TWO RUNGS", red()),
        },
    ];

    for (i, chain) in chains.iter().enumerate() {
        let x = col_x[i];
        sheet.label(titles[i], x, 1104.0, 30.0, chain.color, 0);
        for (j, v) in chain.values.iter().enumerate() {
            let y_c = y_top - j as f64 * dy; // box center
            sheet.ctx.no_fill().stroke(chain.color).stroke_width(2.5);
            sheet.ctx.rect(x - box_w / 2.0, y_c - box_h / 2.0, box_w, box_h);
            sheet.label(&v.to_string(), x, y_c - 9.0, 26.0, chain.color, 0);
            if j + 1 < chain.values.len() {
                sheet.label(chain.div, x + box_w / 2.0 + 28.0, y_c - dy / 2.0 - 9.0, 22.0, dim(), -1);
                sheet.ctx.no_fill().stroke(dim()).stroke_width(2.0);
                sheet
                    .ctx
                    .line(x, y_c - box_h / 2.0, x, y_c - dy + box_h / 2.0);
            }
        }
        if let Some(msg) = chain.dead_end {
            let y_c = y_top - chain.values.len() as f64 * dy;
            sheet.label("\u{d7}", x, y_c - 9.0, 34.0, red(), 0);
            sheet.label(msg, x, y_c - 60.0, 24.0, red(), 0);
        }
        sheet.label(chain.verdict.0, x, 196.0, 24.0, chain.verdict.1, 0);
    }

    sheet.frame(
        "HOW FAR CAN AN EM HALVE?",
        "ONLY A BINARY EM LADDERS FROM THE EM TO THE UNIT",
    );
    sheet.save(renderer, out);
}

// --- fig-bits -------------------------------------------------------------------

fn fig_bits(renderer: &Renderer, mono: &str, out: &std::path::Path) {
    let mut sheet = new_sheet(renderer, mono);

    let rows: [(&str, u32, Color); 5] = [
        ("THE EM", 1024, green()),
        ("CAP HEIGHT", 768, green()),
        ("X-HEIGHT", 576, green()),
        ("STEM", 96, green()),
        ("THE HAND", 116, red()),
    ];
    let cell = 104.0;
    let gap = 12.0;
    let bits_x0 = 820.0;
    let n_bits = 11usize;

    for (i, (name, value, color)) in rows.iter().enumerate() {
        let y0 = 956.0 - i as f64 * 168.0; // cell bottom
        let level = value.trailing_zeros() as usize;

        sheet.label(name, MARGIN, y0 + 20.0, 26.0, gray(), -1);
        sheet.label(&value.to_string(), 760.0, y0 + 20.0, 34.0, *color, 1);

        for b in 0..n_bits {
            let bit_index = n_bits - 1 - b; // msb first
            let x = bits_x0 + b as f64 * (cell + gap);
            let is_one = (value >> bit_index) & 1 == 1;
            if is_one {
                sheet.ctx.fill(*color).stroke(*color).stroke_width(2.0);
                sheet.ctx.rect(x, y0, cell, cell);
                sheet.label("1", x + cell / 2.0, y0 + cell * 0.32, 46.0, bg(), 0);
            } else {
                sheet.ctx.no_fill().stroke(dim()).stroke_width(2.0);
                sheet.ctx.rect(x, y0, cell, cell);
                sheet.label("0", x + cell / 2.0, y0 + cell * 0.32, 46.0, dim(), 0);
            }
        }

        // bracket under the trailing-zero run
        if level > 0 {
            let x_start = bits_x0 + (n_bits - level) as f64 * (cell + gap);
            let x_end = bits_x0 + n_bits as f64 * (cell + gap) - gap;
            let yb = y0 - 22.0;
            sheet.ctx.no_fill().stroke(*color).stroke_width(2.5);
            sheet.ctx.line(x_start, yb, x_end, yb);
            sheet.ctx.line(x_start, yb, x_start, yb + 12.0);
            sheet.ctx.line(x_end, yb, x_end, yb + 12.0);
        }

        let tag = if *value == 116 {
            format!("{} ZEROS \u{2192} ON 4, OFF 8", level)
        } else {
            format!("{} ZEROS \u{2192} ON {}", level, 1u32 << level)
        };
        sheet.label(&tag, W - MARGIN, y0 + 20.0, 26.0, *color, 1);
    }

    sheet.frame(
        "THE LEVEL IS IN THE LOW BITS",
        "GRID LEVEL = TRAILING ZEROS (THE 2-ADIC VALUATION); HARDWARE READS IT IN ONE INSTRUCTION, CTZ",
    );
    sheet.save(renderer, out);
}

// --- main -----------------------------------------------------------------------

fn main() {
    let home = std::env::var("HOME").unwrap();
    let mono_path = format!("{home}/GH/repos/google-fonts/ofl/geistmono/GeistMono[wght].ttf");

    let mut renderer = Renderer::new(W as u32, H as u32);
    let mono = load_family(&mut renderer, &mono_path);

    let here = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let post = here
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("src/content/blog/virtua-grotesk");

    fig_fractions(&renderer, &mono, &post.join("fig-fractions.png"));
    fig_midpoint(&renderer, &mono, &post.join("fig-midpoint.png"));
    fig_ladder(&renderer, &mono, &post.join("fig-ladder.png"));
    fig_bits(&renderer, &mono, &post.join("fig-bits.png"));
}
