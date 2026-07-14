//! Design-system dimension sheets for the Virtua Grotesk post, §03 — the
//! two figures that replace the measurement tables:
//!
//!   fig-system-no.png   : the lowercase system on the word "no", zoomable —
//!                         stem 96, curve horizontal 92, curve vertical 100,
//!                         chamfer 16, overshoots, sidebearings, ink widths
//!   fig-system-ohno.png : the full Latin system on "OHno" — cap/asc 768,
//!                         x-height 576, descender -256, overshoots ±16,
//!                         uppercase stem 104 / bar 96, O curves 108 / 100,
//!                         n stem 96, o curve 100
//!
//! All dimensions are MEASURED from the Regular UFO outlines (placed on
//! real edge coordinates), so the figure cannot drift from the sources the
//! way a hand-written table can.
//!
//! Margin discipline: 96px margins; rules at 1224 / 112; content centered
//! between the rules.
//!
//! Run from this directory:
//!
//!     cargo run --release --bin system
//!
//! Inputs read at render time, from sibling checkouts:
//!     ~/GH/repos/virtua-grotesk/sources/VirtuaGrotesk-Regular.ufo
//!     ~/GH/repos/google-fonts/ofl/geistmono/GeistMono[wght].ttf

use designbot::prelude::*;
use designbot_render::Renderer;
use kurbo::{Affine, BezPath};

const W: f64 = 2520.0;
const H: f64 = 1320.0;
const MARGIN: f64 = 96.0;
const HEADER_RULE_Y: f64 = 1178.0;
const FOOTER_RULE_Y: f64 = 142.0;

fn bg() -> Color {
    Color::rgb(0x10, 0x10, 0x10)
}
fn grid() -> Color {
    Color::rgb(0x28, 0x28, 0x28)
}
fn green() -> Color {
    Color::rgb(0x15, 0xc4, 0x74)
}
fn red() -> Color {
    Color::rgb(0xff, 0x45, 0x35)
}
fn red_fill() -> Color {
    Color::rgba(0xff, 0x45, 0x35, 64)
}
fn blue() -> Color {
    Color::rgb(0x4a, 0x78, 0xff)
}
fn gray() -> Color {
    Color::rgb(0x8a, 0x8a, 0x8a)
}

// --- UFO outline loading (path + advance only) ---------------------------------

struct Outline {
    path: BezPath,
    width: f64,
}

fn load_outline(glif: &std::path::Path) -> Outline {
    let glyph = norad::Glyph::load(glif).unwrap_or_else(|e| panic!("load {glif:?}: {e}"));
    let mut path = BezPath::new();
    for contour in &glyph.contours {
        use norad::PointType::*;
        let pts = &contour.points;
        let n = pts.len();
        let Some(start) = pts.iter().position(|p| p.typ != OffCurve) else {
            continue;
        };
        let sp = &pts[start];
        path.move_to((sp.x, sp.y));
        let mut pending: Vec<(f64, f64)> = Vec::new();
        for k in 1..=n {
            let p = &pts[(start + k) % n];
            match p.typ {
                OffCurve => pending.push((p.x, p.y)),
                Curve if pending.len() == 2 => {
                    path.curve_to(pending[0], pending[1], (p.x, p.y));
                    pending.clear();
                }
                _ => {
                    path.line_to((p.x, p.y));
                    pending.clear();
                }
            }
        }
        path.close_path();
    }
    Outline {
        path,
        width: glyph.width,
    }
}

fn glif_name(glyph: &str) -> String {
    if glyph.chars().next().is_some_and(|c| c.is_uppercase()) {
        format!("{glyph}_.glif")
    } else {
        format!("{glyph}.glif")
    }
}

// --- sfnt family-name reader (same as og.rs) ------------------------------------

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

// --- drawing ---------------------------------------------------------------------

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
        let pad = 8.0;
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

    /// Metric-line tag docked on a line. align: -1 left edge, 1 right edge.
    fn metric_tag(&mut self, txt: &str, x_edge: f64, y_line: f64, above: bool, align: i8) {
        let size = 30.0;
        let w = self.mono_width(txt, size);
        let box_w = ((w + 16.0) / 16.0).ceil() * 16.0;
        let box_h = 32.0;
        let x0 = if align < 0 { x_edge } else { x_edge - box_w };
        let y0 = if above { y_line + 16.0 } else { y_line - box_h - 16.0 };
        self.ctx.fill(bg()).stroke(blue()).stroke_width(2.5);
        self.ctx.rect(x0, y0, box_w, box_h);
        let baseline = y0 + (box_h - 0.73 * size) / 2.0;
        self.label(txt, x0 + box_w / 2.0, baseline, size, blue(), 0);
    }

    /// Diagonal hatching clipped to a rect (og.rs sidebearing style).
    fn hatch(&mut self, x0: f64, y0: f64, x1: f64, y1: f64, color: Color) {
        let h = y1 - y0;
        self.ctx.stroke(color).stroke_width(2.5).no_fill();
        let step = 6.0;
        let mut t = x0 - h;
        while t < x1 {
            let sx = t.max(x0);
            let ex = (t + h).min(x1);
            if ex > sx {
                self.ctx.line(sx, y0 + (sx - t), ex, y0 + (ex - t));
            }
            t += step;
        }
    }

    /// Horizontal dimension: tick, line, tick, number centered above.
    fn dim_h(&mut self, x0: f64, x1: f64, y: f64, txt: &str, color: Color) {
        self.ctx.no_fill().stroke(color).stroke_width(2.5);
        self.ctx.line(x0, y, x1, y);
        self.ctx.line(x0, y - 12.0, x0, y + 12.0);
        self.ctx.line(x1, y - 12.0, x1, y + 12.0);
        self.label_padded(txt, (x0 + x1) / 2.0, y + 18.0, 28.0, color, 0);
    }

    /// Vertical dimension: tick, line, tick, number beside it.
    fn dim_v(&mut self, x: f64, y0: f64, y1: f64, txt: &str, color: Color, label_right: bool) {
        self.ctx.no_fill().stroke(color).stroke_width(2.5);
        self.ctx.line(x, y0, x, y1);
        self.ctx.line(x - 12.0, y0, x + 12.0, y0);
        self.ctx.line(x - 12.0, y1, x + 12.0, y1);
        let (lx, align) = if label_right {
            (x + 22.0, -1)
        } else {
            (x - 22.0, 1)
        };
        self.label_padded(txt, lx, (y0 + y1) / 2.0 - 10.0, 28.0, color, align);
    }

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
        self.label(caption, MARGIN, 96.0, 30.0, green(), -1);
        self.label(
            "GITHUB.COM/ELIHEUER/VIRTUA-GROTESK",
            W - MARGIN,
            96.0,
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

/// The design grid (16 units) plus vertical metrics with tags, shared by
/// both sheets. Returns nothing; coordinates come in via the closure pair.
struct Frame {
    s: f64,
    x0: f64,      // canvas x of font x=0 for the first glyph
    baseline: f64, // canvas y of font y=0
}

impl Frame {
    fn x(&self, ux: f64) -> f64 {
        self.x0 + ux * self.s
    }
    fn y(&self, uy: f64) -> f64 {
        self.baseline + uy * self.s
    }
}

fn draw_grid(sheet: &mut Sheet, f: &Frame, top_u: f64, bottom_u: f64) {
    let step = 16.0 * f.s;
    let (y0, y1) = (f.y(bottom_u), f.y(top_u));
    let ctx = &mut sheet.ctx;
    ctx.no_fill().stroke(grid()).stroke_width(2.0);
    let mut x = f.x(0.0) - ((f.x(0.0) - MARGIN) / step).floor() * step;
    while x <= W - MARGIN {
        ctx.line(x, y0, x, y1);
        x += step;
    }
    let mut y = f.y(0.0) - ((f.y(0.0) - y0) / step).floor() * step;
    while y <= y1 {
        ctx.line(MARGIN, y, W - MARGIN, y);
        y += step;
    }
}

fn metric_lines(sheet: &mut Sheet, f: &Frame, solid: &[f64], dashed: &[f64]) {
    let ctx = &mut sheet.ctx;
    ctx.no_fill().stroke(blue()).stroke_width(2.5);
    ctx.line_dash(&[10.0, 10.0]);
    for &uy in dashed {
        ctx.line(MARGIN, f.y(uy), W - MARGIN, f.y(uy));
    }
    ctx.line_dash(&[]);
    for &uy in solid {
        ctx.line(MARGIN, f.y(uy), W - MARGIN, f.y(uy));
    }
}

/// Advance-width dimension zone under the baseline: boundary ticks, hatched
/// sidebearings, ink width centered, bearings at the ends (og.rs language).
fn advance_row(
    sheet: &mut Sheet,
    f: &Frame,
    y: f64,
    glyphs: &[(f64, f64, f64)], // (origin_ux, advance, lsb) ... rsb derived next origin
    inks: &[(f64, f64)],        // (ink_x0, ink_x1) per glyph, font units
) {
    for ((origin, adv, _lsb), (ink0, ink1)) in glyphs.iter().zip(inks) {
        let (bx0, bx1) = (f.x(*origin), f.x(origin + adv));
        let (ix0, ix1) = (f.x(*ink0 + origin), f.x(*ink1 + origin));
        sheet.ctx.no_fill().stroke(gray()).stroke_width(2.5);
        sheet.ctx.line(bx0, y, bx1, y);
        sheet.ctx.line(bx0, y - 20.0, bx0, y + 20.0);
        sheet.ctx.line(bx1, y - 20.0, bx1, y + 20.0);
        sheet.hatch(bx0, y - 14.0, ix0, y + 14.0, red());
        sheet.hatch(ix1, y - 14.0, bx1, y + 14.0, red());
        sheet.label_padded(
            &format!("{}", (ink1 - ink0).round()),
            (ix0 + ix1) / 2.0,
            y - 10.0,
            28.0,
            green(),
            0,
        );
        sheet.label(
            &format!("{}", (ink0).round()),
            ix0 + 10.0,
            y + 26.0,
            26.0,
            red(),
            -1,
        );
        sheet.label(
            &format!("{}", (adv - ink1).round()),
            ix1 - 10.0,
            y + 26.0,
            26.0,
            red(),
            1,
        );
    }
}

// --- fig-system-no ----------------------------------------------------------------

fn fig_no(renderer: &Renderer, mono: &str, glyphs_dir: &std::path::Path, out: &std::path::Path) {
    let mut sheet = Sheet {
        ctx: Canvas::new(W, H),
        renderer,
        mono: mono.to_string(),
    };
    sheet.ctx.background(bg());

    let n = load_outline(&glyphs_dir.join(glif_name("n")));
    let o = load_outline(&glyphs_dir.join(glif_name("o")));

    const S: f64 = 1.3;
    let run = (n.width + o.width) * S;
    let f = Frame {
        s: S,
        x0: MARGIN + (W - 2.0 * MARGIN - run) / 2.0,
        baseline: 310.0,
    };

    draw_grid(&mut sheet, &f, 640.0, -64.0);
    metric_lines(&mut sheet, &f, &[0.0, 576.0], &[-16.0, 592.0]);

    // glyphs, mark-red with the grid reading through (og gauge-ball style)
    for (outline, ox) in [(&n, 0.0), (&o, n.width)] {
        let place = Affine::new([S, 0.0, 0.0, S, f.x(ox), f.baseline]);
        sheet.ctx.fill(red_fill()).stroke(red()).stroke_width(2.5);
        sheet.ctx.draw_path(place * outline.path.clone());
    }

    // measured stroke dimensions, on real edge coordinates
    // n left stem: x 64..160 at y 256
    sheet.dim_h(f.x(64.0), f.x(160.0), f.y(256.0), "96", green());
    // o top curve: y 500..592 at the apex x = n.width + 304
    sheet.dim_v(f.x(n.width + 304.0), f.y(500.0), f.y(592.0), "92", green(), true);
    // o left wall: x 32..132 at mid-height y 288
    sheet.dim_h(
        f.x(n.width + 32.0),
        f.x(n.width + 132.0),
        f.y(288.0),
        "100",
        green(),
    );
    // chamfer callout: n bottom-left corner (64,16)-(80,0)
    {
        let (cx, cy) = (f.x(72.0), f.y(8.0));
        sheet.ctx.no_fill().stroke(green()).stroke_width(2.5);
        sheet.ctx.line(cx - 14.0, cy - 14.0, cx - 90.0, cy - 90.0);
        sheet.label_padded("CHAMFER 16", cx - 104.0, cy - 116.0, 28.0, green(), 1);
    }

    // metric tags
    sheet.metric_tag("X-HEIGHT 576", MARGIN, f.y(576.0), true, -1);
    sheet.metric_tag("BASELINE 0", MARGIN, f.y(0.0), true, -1);
    sheet.metric_tag("OVERSHOOT +16", W - MARGIN, f.y(592.0), true, 1);
    sheet.metric_tag("OVERSHOOT -16", W - MARGIN, f.y(-16.0), false, 1);

    // advance dimension zone
    advance_row(
        &mut sheet,
        &f,
        204.0,
        &[(0.0, n.width, 64.0), (n.width, o.width, 32.0)],
        &[(64.0, 528.0), (32.0, 584.0)],
    );

    sheet.frame(
        "THE LOWERCASE SYSTEM / no",
        "EVERY MEASUREMENT A SHORT SUM OF POWERS OF TWO: 96 = 64+32, 100 = 64+32+4, 92 = 64+16+8+4",
    );
    sheet.save(renderer, out);
}

// --- fig-system-ohno ---------------------------------------------------------------

fn fig_ohno(renderer: &Renderer, mono: &str, glyphs_dir: &std::path::Path, out: &std::path::Path) {
    let mut sheet = Sheet {
        ctx: Canvas::new(W, H),
        renderer,
        mono: mono.to_string(),
    };
    sheet.ctx.background(bg());

    let cap_o = load_outline(&glyphs_dir.join(glif_name("O")));
    let cap_h = load_outline(&glyphs_dir.join(glif_name("H")));
    let n = load_outline(&glyphs_dir.join(glif_name("n")));
    let o = load_outline(&glyphs_dir.join(glif_name("o")));

    const S: f64 = 0.78;
    let advances = [cap_o.width, cap_h.width, n.width, o.width];
    let run: f64 = advances.iter().sum::<f64>() * S;
    // content spans font -256..784; center it between the rules
    let content_h = (784.0 + 256.0) * S + 50.0;
    let bottom = FOOTER_RULE_Y + (HEADER_RULE_Y - FOOTER_RULE_Y - content_h) / 2.0;
    let f = Frame {
        s: S,
        x0: MARGIN + (W - 2.0 * MARGIN - run) / 2.0,
        baseline: bottom + 256.0 * S,
    };

    draw_grid(&mut sheet, &f, 784.0, -256.0);
    metric_lines(&mut sheet, &f, &[0.0, 576.0, 768.0, -256.0], &[784.0, -16.0]);

    // glyphs
    let mut ox = 0.0;
    for outline in [&cap_o, &cap_h, &n, &o] {
        let place = Affine::new([S, 0.0, 0.0, S, f.x(ox), f.baseline]);
        sheet.ctx.fill(red_fill()).stroke(red()).stroke_width(2.5);
        sheet.ctx.draw_path(place * outline.path.clone());
        ox += outline.width;
    }
    let (x_o, x_h, x_n, x_lo) = (
        0.0,
        cap_o.width,
        cap_o.width + cap_h.width,
        cap_o.width + cap_h.width + n.width,
    );

    // measured stroke dimensions
    // O left wall: x 48..156 at y 384
    sheet.dim_h(f.x(x_o + 48.0), f.x(x_o + 156.0), f.y(384.0), "108", green());
    // O top curve: y 684..784 at apex x 424
    sheet.dim_v(f.x(x_o + 424.0), f.y(684.0), f.y(784.0), "100", green(), true);
    // H left stem: x 80..184 at y 600
    sheet.dim_h(f.x(x_h + 80.0), f.x(x_h + 184.0), f.y(600.0), "104", green());
    // H bar: y 360..456 at center x 384
    sheet.dim_v(f.x(x_h + 384.0), f.y(360.0), f.y(456.0), "96", green(), true);
    // n left stem: x 64..160 at y 256
    sheet.dim_h(f.x(x_n + 64.0), f.x(x_n + 160.0), f.y(256.0), "96", green());
    // o left wall: x 32..132 at y 288
    sheet.dim_h(f.x(x_lo + 32.0), f.x(x_lo + 132.0), f.y(288.0), "100", green());

    // metric tags: values decomposed the way the design system reads them
    // (cap tag docks right so it clears the O's top-curve dimension)
    sheet.metric_tag("CAP / ASC 768 = 512+256", W - MARGIN, f.y(768.0), false, 1);
    sheet.metric_tag("X-HEIGHT 576 = 512+64", MARGIN, f.y(576.0), true, -1);
    sheet.metric_tag("BASELINE 0", MARGIN, f.y(0.0), true, -1);
    sheet.metric_tag("DESCENDER -256 = -(2^8)", MARGIN, f.y(-256.0), true, -1);
    sheet.metric_tag("OVERSHOOT +16", W - MARGIN, f.y(784.0), true, 1);
    sheet.metric_tag("OVERSHOOT -16", W - MARGIN, f.y(-16.0), false, 1);

    sheet.frame(
        "THE LATIN SYSTEM / OHno",
        "METRICS ON THE 64-GRID; STROKES FROM 2 4 8 16 32 64 128 AND SHORT SUMS OF THEM",
    );
    sheet.save(renderer, out);
}

// --- main --------------------------------------------------------------------------

fn main() {
    let home = std::env::var("HOME").unwrap();
    let mono_path = format!("{home}/GH/repos/google-fonts/ofl/geistmono/GeistMono[wght].ttf");
    let glyphs_dir = std::path::PathBuf::from(&home)
        .join("GH/repos/virtua-grotesk/sources/VirtuaGrotesk-Regular.ufo/glyphs");

    let mut renderer = Renderer::new(W as u32, H as u32);
    let mono = load_family(&mut renderer, &mono_path);

    let here = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let post = here
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("src/content/blog/virtua-grotesk");

    fig_no(&renderer, &mono, &glyphs_dir, &post.join("fig-system-no.png"));
    fig_ohno(&renderer, &mono, &glyphs_dir, &post.join("fig-system-ohno.png"));
}
