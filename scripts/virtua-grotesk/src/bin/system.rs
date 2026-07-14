//! Design-system dimension sheets for the Virtua Grotesk post, §03 — the
//! figures that replace the measurement tables:
//!
//!   fig-system-ohno.png    : the full Latin system on "OHno"
//!   fig-system-no.png      : the lowercase system on "no", zoomable
//!   fig-system-weights.png : "no no" — Regular beside Bold, stems 96 -> 192
//!   fig-system-arabic.png  : alef, beh, medial heh — right to left, same grid
//!
//! Every sheet is drawn in the Runebender point language: on-curve points
//! (circle = smooth, square = corner), off-curve handles, and the key
//! semantic distinction of the whole design system — points on the 8-unit
//! machine grid are GREEN, points off 8 but on the 2-unit human grid are
//! RED (optical corrections). Axis-aligned handle lengths from the favored
//! set are labeled in purple; on-curve extrema get their real coordinates.
//! All dimensions are MEASURED from the UFO outlines, so the figures cannot
//! drift from the sources the way a hand-written table can.
//!
//! Run from this directory:
//!
//!     cargo run --release --bin system
//!
//! Inputs read at render time, from sibling checkouts:
//!     ~/GH/repos/virtua-grotesk/sources/VirtuaGrotesk-{Regular,Bold}.ufo
//!     ~/GH/repos/google-fonts/ofl/geistmono/GeistMono[wght].ttf

use designbot::prelude::*;
use designbot_render::Renderer;
use kurbo::{Affine, BezPath};

const W: f64 = 2520.0;
const H: f64 = 1320.0;
const MARGIN: f64 = 64.0;
const HEADER_RULE_Y: f64 = 1210.0;
const FOOTER_RULE_Y: f64 = 110.0;

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
fn purple() -> Color {
    Color::rgb(0x8c, 0x6c, 0xff)
}
fn handle_color() -> Color {
    Color::rgb(0x6a, 0x6a, 0x6a)
}

/// Handle lengths worth calling out: the favored set, 2^k and short sums.
const FAVORED: [f64; 7] = [64.0, 96.0, 128.0, 160.0, 192.0, 224.0, 256.0];

// --- UFO outline loading, components resolved ----------------------------------

#[derive(Clone, Copy)]
enum PtRole {
    Smooth,
    Corner,
    Off,
}

struct Outline {
    path: BezPath,
    points: Vec<(f64, f64, PtRole)>,
    handles: Vec<((f64, f64), (f64, f64))>, // on-curve anchor -> off-curve
    width: f64,
}

/// UFO3 glif filename: every uppercase letter gets an underscore suffix.
fn glif_name(glyph: &str) -> String {
    let mut s = String::new();
    for c in glyph.chars() {
        s.push(c);
        if c.is_uppercase() {
            s.push('_');
        }
    }
    s + ".glif"
}

fn push_on_curve(points: &mut Vec<(f64, f64, PtRole)>, p: &norad::ContourPoint, k: usize, n: usize) {
    if k == n {
        return;
    }
    let role = if p.smooth { PtRole::Smooth } else { PtRole::Corner };
    points.push((p.x, p.y, role));
}

/// Load a glyph's outline with components resolved recursively (the beh is
/// behDotless + dotbelow in the sources; the figure shows plain outlines).
fn load_outline(glyphs_dir: &std::path::Path, name: &str) -> Outline {
    let glif = glyphs_dir.join(glif_name(name));
    let glyph = norad::Glyph::load(&glif).unwrap_or_else(|e| panic!("load {glif:?}: {e}"));
    let mut path = BezPath::new();
    let mut points = Vec::new();
    let mut handles = Vec::new();
    for contour in &glyph.contours {
        use norad::PointType::*;
        let pts = &contour.points;
        let n = pts.len();
        let Some(start) = pts.iter().position(|p| p.typ != OffCurve) else {
            continue;
        };
        let sp = &pts[start];
        path.move_to((sp.x, sp.y));
        let role = if sp.smooth { PtRole::Smooth } else { PtRole::Corner };
        points.push((sp.x, sp.y, role));
        let mut prev_on = (sp.x, sp.y);
        let mut pending: Vec<(f64, f64)> = Vec::new();
        for k in 1..=n {
            let p = &pts[(start + k) % n];
            match p.typ {
                OffCurve => {
                    pending.push((p.x, p.y));
                    points.push((p.x, p.y, PtRole::Off));
                }
                Curve if pending.len() == 2 => {
                    path.curve_to(pending[0], pending[1], (p.x, p.y));
                    handles.push((prev_on, pending[0]));
                    handles.push(((p.x, p.y), pending[1]));
                    pending.clear();
                    prev_on = (p.x, p.y);
                    push_on_curve(&mut points, p, k, n);
                }
                _ => {
                    path.line_to((p.x, p.y));
                    pending.clear();
                    prev_on = (p.x, p.y);
                    push_on_curve(&mut points, p, k, n);
                }
            }
        }
        path.close_path();
    }
    for comp in &glyph.components {
        let sub = load_outline(glyphs_dir, comp.base.as_ref());
        let t = &comp.transform;
        let aff = Affine::new([
            t.x_scale, t.xy_scale, t.yx_scale, t.y_scale, t.x_offset, t.y_offset,
        ]);
        path.extend(aff * sub.path);
        let map = |x: f64, y: f64| {
            (
                t.x_scale * x + t.yx_scale * y + t.x_offset,
                t.xy_scale * x + t.y_scale * y + t.y_offset,
            )
        };
        for (x, y, r) in sub.points {
            let (nx, ny) = map(x, y);
            points.push((nx, ny, r));
        }
        for ((ax, ay), (hx, hy)) in sub.handles {
            handles.push((map(ax, ay), map(hx, hy)));
        }
    }
    Outline {
        path,
        points,
        handles,
        width: glyph.width,
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

    fn dim_h(&mut self, x0: f64, x1: f64, y: f64, txt: &str, color: Color) {
        self.ctx.no_fill().stroke(color).stroke_width(2.5);
        self.ctx.line(x0, y, x1, y);
        self.ctx.line(x0, y - 12.0, x0, y + 12.0);
        self.ctx.line(x1, y - 12.0, x1, y + 12.0);
        self.label_padded(txt, (x0 + x1) / 2.0, y + 18.0, 28.0, color, 0);
    }

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

fn on8(x: f64, y: f64) -> bool {
    x.rem_euclid(8.0) == 0.0 && y.rem_euclid(8.0) == 0.0
}

// --- the annotation engine --------------------------------------------------------

/// Glyph body: mark-red fill with a solid contour, og.rs gauge-ball style.
fn draw_body(sheet: &mut Sheet, o: &Outline, s: f64, x0: f64, baseline: f64) {
    let place = Affine::new([s, 0.0, 0.0, s, x0, baseline]);
    sheet.ctx.fill(red_fill()).stroke(red()).stroke_width(2.5);
    sheet.ctx.draw_path(place * o.path.clone());
}

/// Handles + point markers: circle = smooth, square = corner, small circle =
/// off-curve; GREEN if on the 8-unit machine grid, RED if off 8 (the human's
/// 2-unit optical corrections).
fn draw_points(sheet: &mut Sheet, o: &Outline, s: f64, x0: f64, baseline: f64) {
    sheet.ctx.no_fill().stroke(handle_color()).stroke_width(2.0);
    for ((ax, ay), (hx, hy)) in &o.handles {
        sheet.ctx.line(
            x0 + ax * s,
            baseline + ay * s,
            x0 + hx * s,
            baseline + hy * s,
        );
    }
    for (px, py, role) in &o.points {
        let (cx, cy) = (x0 + px * s, baseline + py * s);
        let color = if on8(*px, *py) { green() } else { red() };
        sheet.ctx.fill(bg()).stroke(color).stroke_width(2.5);
        match role {
            PtRole::Smooth => {
                sheet.ctx.oval(cx - 8.0, cy - 8.0, 16.0, 16.0);
            }
            PtRole::Corner => {
                sheet.ctx.rect(cx - 7.0, cy - 7.0, 14.0, 14.0);
            }
            PtRole::Off => {
                sheet.ctx.oval(cx - 5.5, cy - 5.5, 11.0, 11.0);
            }
        }
    }
}

/// Purple labels on axis-aligned handles whose length is in the favored set.
fn handle_labels(sheet: &mut Sheet, o: &Outline, s: f64, x0: f64, baseline: f64) {
    for ((ax, ay), (hx, hy)) in &o.handles {
        let (dx, dy) = (hx - ax, hy - ay);
        let len = dx.abs().max(dy.abs());
        if !(dx == 0.0 || dy == 0.0) || !FAVORED.contains(&len) {
            continue;
        }
        let (mx, my) = (x0 + (ax + hx) / 2.0 * s, baseline + (ay + hy) / 2.0 * s);
        if dy == 0.0 {
            let ly = if *ay >= 440.0 { my - 30.0 } else { my + 12.0 };
            sheet.label_padded(&format!("{len}"), mx, ly, 20.0, purple(), 0);
        } else {
            sheet.label_padded(&format!("{len}"), mx + 14.0, my - 7.0, 20.0, purple(), -1);
        }
    }
}

/// Real coordinates on the on-curve extrema, gray, offset outward.
fn extrema_labels(sheet: &mut Sheet, o: &Outline, s: f64, x0: f64, baseline: f64) {
    let on: Vec<(f64, f64)> = o
        .points
        .iter()
        .filter(|(_, _, r)| !matches!(r, PtRole::Off))
        .map(|(x, y, _)| (*x, *y))
        .collect();
    if on.is_empty() {
        return;
    }
    let pick = |f: &dyn Fn(&(f64, f64)) -> f64, max: bool| -> (f64, f64) {
        let mut best = on[0];
        for p in &on {
            if (max && f(p) > f(&best)) || (!max && f(p) < f(&best)) {
                best = *p;
            }
        }
        best
    };
    let left = pick(&|p| p.0, false);
    let right = pick(&|p| p.0, true);
    let top = pick(&|p| p.1, true);
    let bottom = pick(&|p| p.1, false);
    let mut seen: Vec<(f64, f64)> = Vec::new();
    for (p, dxy, align) in [
        (left, (-16.0, -7.0), 1),
        (right, (16.0, -7.0), -1),
        (top, (0.0, 18.0), 0),
        (bottom, (0.0, -32.0), 0),
    ] {
        if seen.contains(&p) {
            continue;
        }
        seen.push(p);
        let txt = format!("({},{})", p.0, p.1);
        let w = sheet.mono_width(&txt, 20.0);
        let anchor = x0 + p.0 * s + dxy.0;
        let mut tx = match align {
            1 => anchor - w,
            0 => anchor - w / 2.0,
            _ => anchor,
        };
        tx = tx.clamp(MARGIN + 10.0, W - MARGIN - 10.0 - w);
        sheet.label_padded(&txt, tx, baseline + p.1 * s + dxy.1, 20.0, gray(), -1);
    }
}

fn annotate(sheet: &mut Sheet, o: &Outline, s: f64, x0: f64, baseline: f64) {
    draw_body(sheet, o, s, x0, baseline);
    draw_points(sheet, o, s, x0, baseline);
    handle_labels(sheet, o, s, x0, baseline);
    extrema_labels(sheet, o, s, x0, baseline);
}

/// The machine/hand legend, one row, right-aligned at (x_right, y baseline).
fn legend(sheet: &mut Sheet, x_right: f64, y: f64) {
    let size = 24.0;
    let t2 = "OFF 8, ON 2 = THE HAND";
    let t1 = "ON 8 = MACHINE";
    let w2 = sheet.mono_width(t2, size);
    let w1 = sheet.mono_width(t1, size);
    let x2 = x_right - w2;
    let dot2 = x2 - 26.0;
    let x1 = dot2 - 40.0 - w1;
    let dot1 = x1 - 26.0;
    sheet.label(t2, x2, y, size, red(), -1);
    sheet.label(t1, x1, y, size, green(), -1);
    sheet.ctx.fill(bg()).stroke(red()).stroke_width(2.5);
    sheet.ctx.oval(dot2, y + 1.0, 15.0, 15.0);
    sheet.ctx.fill(bg()).stroke(green()).stroke_width(2.5);
    sheet.ctx.oval(dot1, y + 1.0, 15.0, 15.0);
}

/// Red leader + label calling out one optical-correction point.
fn correction_callout(sheet: &mut Sheet, from: (f64, f64), text_at: (f64, f64), align: i8) {
    sheet.ctx.no_fill().stroke(red()).stroke_width(2.5);
    sheet.ctx.line(from.0, from.1, text_at.0, text_at.1 + 8.0);
    let dx = if align < 0 { 10.0 } else { -10.0 };
    sheet.label_padded(
        "OFF 8, ON 2: OPTICAL CORRECTION",
        text_at.0 + dx,
        text_at.1,
        24.0,
        red(),
        align,
    );
}

struct Frame {
    s: f64,
    x0: f64,
    baseline: f64,
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

fn advance_row(
    sheet: &mut Sheet,
    f: &Frame,
    y: f64,
    glyphs: &[(f64, f64)], // (origin_ux, advance)
    inks: &[(f64, f64)],   // (ink_x0, ink_x1) glyph-local
) {
    for ((origin, adv), (ink0, ink1)) in glyphs.iter().zip(inks) {
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
        sheet.label(&format!("{}", ink0.round()), ix0 + 10.0, y + 26.0, 26.0, red(), -1);
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

    // metric tags
    sheet.metric_tag("CAP / ASC 768 = 512+256", W - MARGIN, f.y(768.0), false, 1);
    sheet.metric_tag("X-HEIGHT 576 = 512+64", MARGIN, f.y(576.0), true, -1);
    sheet.metric_tag("BASELINE 0", MARGIN, f.y(0.0), true, -1);
    sheet.metric_tag("DESCENDER -256 = -(2^8)", MARGIN, f.y(-256.0), true, -1);
    sheet.metric_tag("OVERSHOOT +16", W - MARGIN, f.y(784.0), true, 1);
    sheet.metric_tag("OVERSHOOT -16", W - MARGIN, f.y(-16.0), false, 1);

    legend(&mut sheet, W - MARGIN, f.y(-150.0));

    sheet.frame(
        "THE LATIN SYSTEM / OHno",
        "METRICS ON THE 64-GRID; STROKES FROM 2 4 8 16 32 64 128 AND SHORT SUMS OF THEM",
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
        sheet.ctx.no_fill().stroke(green()).stroke_width(2.5);
        sheet.ctx.line(cx - 14.0, cy - 14.0, cx - 90.0, cy - 90.0);
        sheet.label_padded("CHAMFER 16", cx - 104.0, cy - 116.0, 28.0, green(), 1);
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
        "EVERY MEASUREMENT A SHORT SUM OF POWERS OF TWO: 96 = 64+32, 100 = 64+32+4, 92 = 64+16+8+4",
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

    // pair origins: Regular no, then Bold no after the gap
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
    sheet.label("01 REGULAR / STEM 96", x_rn, label_y, 26.0, green(), -1);
    sheet.label("02 BOLD / STEM 192 = 96\u{b7}2", x_bn, label_y, 26.0, green(), -1);

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
        "THE BOLD IS THE SAME GRID: STEM 96 -> 192, CURVE 100 -> 196, EVERY DELTA IN FAVORED UNITS",
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

    // name labels, pinned 24 under the header rule, numbered in reading order
    sheet.label("01 ALEF / U+0627", center(2.0), HEADER_RULE_Y - 46.0, 26.0, green(), 0);
    sheet.label("02 BEH / U+0628", center(1.0), HEADER_RULE_Y - 46.0, 26.0, green(), 0);
    sheet.label(
        "03 HEH / MEDIAL FORM",
        center(0.0),
        HEADER_RULE_Y - 46.0,
        26.0,
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
        "SAME GRID, SAME RULES: THE ALEF IS PURE MACHINE GRID; THE BEH'S CURVES ARE THE HAND'S 2-GRID",
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

    fig_ohno(&renderer, &mono, &reg, &post.join("fig-system-ohno.png"));
    fig_no(&renderer, &mono, &reg, &post.join("fig-system-no.png"));
    fig_weights(&renderer, &mono, &reg, &bold, &post.join("fig-system-weights.png"));
    fig_arabic(&renderer, &mono, &reg, &post.join("fig-system-arabic.png"));
}
