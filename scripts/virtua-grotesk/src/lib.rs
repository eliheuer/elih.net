//! The shared design system for every Virtua Grotesk post figure.
//!
//! One palette, one frame, one annotation engine. The bins under src/bin/
//! import everything from here, so a figure cannot drift from the family
//! by construction; `check_margins.py` lints the rendered output against
//! the same spec.
//!
//! FRAME (2520 x 1320, the 1.91:1 social-card ratio):
//!   - outermost ink exactly MARGIN (64) from every canvas edge
//!   - green rules at FOOTER_RULE_Y / HEADER_RULE_Y (112 / 1208)
//!   - frame text (34px mono caps) exactly 24px off its rule
//!   - content block equidistant from the two rules and the side margins
//!
//! PEN (the social treatment, from the OG card's "heavier pen" pass):
//!   - PEN 3.0 for rules, metric lines, dimensions, glyph contours
//!   - PEN_LIGHT 2.0 for the background grid and bezier handles
//!   - two glyph fill strengths: fill() for annotated technical sheets
//!     (grid and point colors read through), fill_strong() for hero
//!     compositions like the OG card
//!
//! POINT LANGUAGE (the key innovation, drawn not told):
//!   - circle = smooth on-curve, square = corner, small circle = off-curve
//!   - GREEN = on the 8-unit machine grid
//!   - RED   = off 8, on the 2-unit human grid: an optical correction

use designbot::prelude::*;
use designbot_render::Renderer;
use kurbo::{Affine, BezPath};

// --- frame ------------------------------------------------------------------------

pub const W: f64 = 2520.0;
pub const H: f64 = 1320.0;
pub const MARGIN: f64 = 64.0;
pub const HEADER_RULE_Y: f64 = 1208.0;
pub const FOOTER_RULE_Y: f64 = 112.0;

// --- pen + type scale ---------------------------------------------------------------

pub const PEN: f64 = 3.0; // rules, metric lines, dims, contours
pub const PEN_LIGHT: f64 = 2.0; // background grid, handles
pub const FRAME_TEXT: f64 = 34.0; // title / caption rows
pub const TAG_TEXT: f64 = 30.0; // blue metric tags
pub const DIM_TEXT: f64 = 30.0; // dimension numbers
pub const LABEL_TEXT: f64 = 28.0; // panel / pair labels
pub const LEGEND_TEXT: f64 = 26.0; // legends, callouts
pub const SMALL_TEXT: f64 = 22.0; // coordinates, handle lengths

// --- palette ------------------------------------------------------------------------

pub fn bg() -> Color {
    Color::rgb(0x10, 0x10, 0x10)
}
pub fn grid() -> Color {
    Color::rgb(0x28, 0x28, 0x28)
}
pub fn green() -> Color {
    Color::rgb(0x15, 0xc4, 0x74)
}
pub fn red() -> Color {
    Color::rgb(0xff, 0x45, 0x35)
}
pub fn blue() -> Color {
    Color::rgb(0x4a, 0x78, 0xff)
}
pub fn gray() -> Color {
    Color::rgb(0x8a, 0x8a, 0x8a)
}
pub fn dim_color() -> Color {
    Color::rgb(0x5a, 0x5a, 0x5a)
}
pub fn purple() -> Color {
    Color::rgb(0x8c, 0x6c, 0xff)
}
pub fn handle_color() -> Color {
    Color::rgb(0x6a, 0x6a, 0x6a)
}
pub fn curve_color() -> Color {
    Color::rgb(230, 230, 230)
}

/// Translucent fill for annotated technical sheets.
pub fn fill_of(stroke: Color) -> Color {
    Color::rgba(stroke.r, stroke.g, stroke.b, 64)
}
/// Rich fill for hero compositions (the OG card's vibrance).
pub fn fill_strong(stroke: Color) -> Color {
    Color::rgba(stroke.r, stroke.g, stroke.b, 104)
}

/// Handle lengths worth calling out: the values the system actually
/// reuses (all with high 2-adic valuation or stem-adjacent).
pub const FAVORED: [f64; 7] = [64.0, 96.0, 128.0, 160.0, 192.0, 224.0, 256.0];

// --- UFO outline loading, components resolved ----------------------------------------

#[derive(Clone, Copy, PartialEq)]
pub enum PtRole {
    Smooth,
    Corner,
    Off,
}

pub struct Outline {
    pub path: BezPath,
    pub points: Vec<(f64, f64, PtRole)>,
    pub handles: Vec<((f64, f64), (f64, f64))>, // on-curve anchor -> off-curve
    pub width: f64,
}

/// UFO3 glif filename: every uppercase letter gets an underscore suffix.
pub fn glif_name(glyph: &str) -> String {
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

/// Load a glyph's outline with components resolved recursively.
pub fn load_outline(glyphs_dir: &std::path::Path, name: &str) -> Outline {
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

// --- sfnt family-name reader ----------------------------------------------------------

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

/// Load the font into the renderer and return its Windows family name.
pub fn load_family(renderer: &mut Renderer, path: &str) -> String {
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

// --- the sheet -------------------------------------------------------------------------

pub struct Sheet<'a> {
    pub ctx: Canvas,
    pub renderer: &'a Renderer,
    pub mono: String,
}

pub fn new_sheet<'a>(renderer: &'a Renderer, mono: &str) -> Sheet<'a> {
    let mut sheet = Sheet {
        ctx: Canvas::new(W, H),
        renderer,
        mono: mono.to_string(),
    };
    sheet.ctx.background(bg());
    sheet
}

impl Sheet<'_> {
    pub fn mono_width(&self, txt: &str, size: f64) -> f64 {
        self.renderer.text_width(txt, Some(&self.mono), size, &[])
    }

    /// Mono label with its baseline at y. align: -1 left, 0 center, 1 right.
    pub fn label(&mut self, txt: &str, x: f64, y: f64, size: f64, color: Color, align: i8) {
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

    /// Label over a background patch so it stays legible on the grid.
    pub fn label_padded(&mut self, txt: &str, x: f64, y: f64, size: f64, color: Color, align: i8) {
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
    pub fn metric_tag(&mut self, txt: &str, x_edge: f64, y_line: f64, above: bool, align: i8) {
        let size = TAG_TEXT;
        let w = self.mono_width(txt, size);
        let box_w = ((w + 16.0) / 16.0).ceil() * 16.0;
        let box_h = 32.0;
        let x0 = if align < 0 { x_edge } else { x_edge - box_w };
        let y0 = if above { y_line + 16.0 } else { y_line - box_h - 16.0 };
        self.ctx.fill(bg()).stroke(blue()).stroke_width(PEN);
        self.ctx.rect(x0, y0, box_w, box_h);
        let baseline = y0 + (box_h - 0.73 * size) / 2.0;
        self.label(txt, x0 + box_w / 2.0, baseline, size, blue(), 0);
    }

    /// Diagonal hatching clipped to a rect (the sidebearing texture).
    pub fn hatch(&mut self, x0: f64, y0: f64, x1: f64, y1: f64, color: Color) {
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
    pub fn dim_h(&mut self, x0: f64, x1: f64, y: f64, txt: &str, color: Color) {
        self.ctx.no_fill().stroke(color).stroke_width(PEN);
        self.ctx.line(x0, y, x1, y);
        self.ctx.line(x0, y - 12.0, x0, y + 12.0);
        self.ctx.line(x1, y - 12.0, x1, y + 12.0);
        self.label_padded(txt, (x0 + x1) / 2.0, y + 18.0, DIM_TEXT, color, 0);
    }

    /// Vertical dimension: tick, line, tick, number beside it.
    pub fn dim_v(&mut self, x: f64, y0: f64, y1: f64, txt: &str, color: Color, label_right: bool) {
        self.ctx.no_fill().stroke(color).stroke_width(PEN);
        self.ctx.line(x, y0, x, y1);
        self.ctx.line(x - 12.0, y0, x + 12.0, y0);
        self.ctx.line(x - 12.0, y1, x + 12.0, y1);
        let (lx, align) = if label_right {
            (x + 22.0, -1)
        } else {
            (x - 22.0, 1)
        };
        self.label_padded(txt, lx, (y0 + y1) / 2.0 - 10.0, DIM_TEXT, color, align);
    }

    /// Header/footer rules and the four frame-text slots.
    pub fn frame(&mut self, title: &str, right: &str, caption: &str) {
        self.ctx.stroke(green()).stroke_width(PEN).no_fill();
        self.ctx.line(MARGIN, HEADER_RULE_Y, W - MARGIN, HEADER_RULE_Y);
        self.ctx.line(MARGIN, FOOTER_RULE_Y, W - MARGIN, FOOTER_RULE_Y);
        self.label(title, MARGIN, HEADER_RULE_Y + 24.0, FRAME_TEXT, green(), -1);
        self.label(right, W - MARGIN, HEADER_RULE_Y + 24.0, FRAME_TEXT, green(), 1);
        self.label(caption, MARGIN, 64.0, FRAME_TEXT, green(), -1);
        self.label(
            "github.com/eliheuer/virtua-grotesk",
            W - MARGIN,
            64.0,
            FRAME_TEXT,
            green(),
            1,
        );
    }

    pub fn save(&self, renderer: &Renderer, out: &std::path::Path) {
        std::fs::create_dir_all(out.parent().unwrap()).unwrap();
        renderer
            .render_to_png(&self.ctx, out.to_str().unwrap())
            .unwrap();
        println!("wrote {}", out.display());
    }
}

// --- coordinate frame + furniture --------------------------------------------------------

pub struct Frame {
    pub s: f64,
    pub x0: f64,
    pub baseline: f64,
}

impl Frame {
    pub fn x(&self, ux: f64) -> f64 {
        self.x0 + ux * self.s
    }
    pub fn y(&self, uy: f64) -> f64 {
        self.baseline + uy * self.s
    }
}

/// The 16-unit design grid across the content width.
pub fn draw_grid(sheet: &mut Sheet, f: &Frame, top_u: f64, bottom_u: f64) {
    let step = 16.0 * f.s;
    let (y0, y1) = (f.y(bottom_u), f.y(top_u));
    let ctx = &mut sheet.ctx;
    ctx.no_fill().stroke(grid()).stroke_width(PEN_LIGHT);
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

/// Blue vertical metrics, full width.
pub fn metric_lines(sheet: &mut Sheet, f: &Frame, solid: &[f64], dashed: &[f64]) {
    let ctx = &mut sheet.ctx;
    ctx.no_fill().stroke(blue()).stroke_width(PEN);
    ctx.line_dash(&[10.0, 10.0]);
    for &uy in dashed {
        ctx.line(MARGIN, f.y(uy), W - MARGIN, f.y(uy));
    }
    ctx.line_dash(&[]);
    for &uy in solid {
        ctx.line(MARGIN, f.y(uy), W - MARGIN, f.y(uy));
    }
}

/// Advance-width dimension zone: boundary ticks, hatched sidebearings,
/// ink width centered, bearings at the ends.
pub fn advance_row(
    sheet: &mut Sheet,
    f: &Frame,
    y: f64,
    glyphs: &[(f64, f64)], // (origin_ux, advance)
    inks: &[(f64, f64)],   // (ink_x0, ink_x1) glyph-local
) {
    for ((origin, adv), (ink0, ink1)) in glyphs.iter().zip(inks) {
        let (bx0, bx1) = (f.x(*origin), f.x(origin + adv));
        let (ix0, ix1) = (f.x(*ink0 + origin), f.x(*ink1 + origin));
        sheet.ctx.no_fill().stroke(gray()).stroke_width(PEN);
        sheet.ctx.line(bx0, y, bx1, y);
        sheet.ctx.line(bx0, y - 20.0, bx0, y + 20.0);
        sheet.ctx.line(bx1, y - 20.0, bx1, y + 20.0);
        sheet.hatch(bx0, y - 14.0, ix0, y + 14.0, red());
        sheet.hatch(ix1, y - 14.0, bx1, y + 14.0, red());
        sheet.label_padded(
            &format!("{}", (ink1 - ink0).round()),
            (ix0 + ix1) / 2.0,
            y - 10.0,
            DIM_TEXT,
            green(),
            0,
        );
        sheet.label(&format!("{}", ink0.round()), ix0 + 10.0, y + 26.0, LEGEND_TEXT, red(), -1);
        sheet.label(
            &format!("{}", (adv - ink1).round()),
            ix1 - 10.0,
            y + 26.0,
            LEGEND_TEXT,
            red(),
            1,
        );
    }
}

// --- the annotation engine -----------------------------------------------------------------

pub fn on8(x: f64, y: f64) -> bool {
    x.rem_euclid(8.0) == 0.0 && y.rem_euclid(8.0) == 0.0
}

/// Glyph body with the technical (translucent) fill.
pub fn draw_body(sheet: &mut Sheet, o: &Outline, s: f64, x0: f64, baseline: f64) {
    let place = Affine::new([s, 0.0, 0.0, s, x0, baseline]);
    sheet.ctx.fill(fill_of(red())).stroke(red()).stroke_width(PEN);
    sheet.ctx.draw_path(place * o.path.clone());
}

/// Glyph body in an arbitrary color with the rich hero fill.
pub fn draw_body_strong(sheet: &mut Sheet, o: &Outline, s: f64, x0: f64, baseline: f64, color: Color) {
    let place = Affine::new([s, 0.0, 0.0, s, x0, baseline]);
    sheet.ctx.fill(fill_strong(color)).stroke(color).stroke_width(PEN);
    sheet.ctx.draw_path(place * o.path.clone());
}

/// Handles + point markers colored by GRID LEVEL: green on the 8-unit
/// machine grid, red off 8 (the human's 2-unit optical corrections).
pub fn draw_points(sheet: &mut Sheet, o: &Outline, s: f64, x0: f64, baseline: f64) {
    sheet.ctx.no_fill().stroke(handle_color()).stroke_width(PEN_LIGHT);
    for ((ax, ay), (hx, hy)) in &o.handles {
        sheet.ctx.line(
            x0 + ax * s,
            baseline + ay * s,
            x0 + hx * s,
            baseline + hy * s,
        );
    }
    for (px, py, role) in &o.points {
        let color = if on8(*px, *py) { green() } else { red() };
        marker(sheet, x0 + px * s, baseline + py * s, *role, color);
    }
}

/// Handles + point markers in one color (for role-colored sheets like the
/// model review, where color means input/output/reference instead).
pub fn draw_points_mono(sheet: &mut Sheet, o: &Outline, s: f64, x0: f64, baseline: f64, color: Color) {
    sheet.ctx.no_fill().stroke(handle_color()).stroke_width(PEN_LIGHT);
    for ((ax, ay), (hx, hy)) in &o.handles {
        sheet.ctx.line(
            x0 + ax * s,
            baseline + ay * s,
            x0 + hx * s,
            baseline + hy * s,
        );
    }
    for (px, py, role) in &o.points {
        marker(sheet, x0 + px * s, baseline + py * s, *role, color);
    }
}

fn marker(sheet: &mut Sheet, cx: f64, cy: f64, role: PtRole, color: Color) {
    sheet.ctx.fill(bg()).stroke(color).stroke_width(PEN);
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

/// Purple labels on axis-aligned handles whose length is in the favored set.
pub fn handle_labels(sheet: &mut Sheet, o: &Outline, s: f64, x0: f64, baseline: f64) {
    for ((ax, ay), (hx, hy)) in &o.handles {
        let (dx, dy) = (hx - ax, hy - ay);
        let len = dx.abs().max(dy.abs());
        if !(dx == 0.0 || dy == 0.0) || !FAVORED.contains(&len) {
            continue;
        }
        let (mx, my) = (x0 + (ax + hx) / 2.0 * s, baseline + (ay + hy) / 2.0 * s);
        if dy == 0.0 {
            let ly = if *ay >= 440.0 { my - 30.0 } else { my + 12.0 };
            sheet.label_padded(&format!("{len}"), mx, ly, SMALL_TEXT, purple(), 0);
        } else {
            sheet.label_padded(&format!("{len}"), mx + 14.0, my - 7.0, SMALL_TEXT, purple(), -1);
        }
    }
}

/// Real coordinates on the on-curve extrema, gray, clamped to the margins.
pub fn extrema_labels(sheet: &mut Sheet, o: &Outline, s: f64, x0: f64, baseline: f64) {
    let on: Vec<(f64, f64)> = o
        .points
        .iter()
        .filter(|(_, _, r)| *r != PtRole::Off)
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
        let w = sheet.mono_width(&txt, SMALL_TEXT);
        let anchor = x0 + p.0 * s + dxy.0;
        let mut tx = match align {
            1 => anchor - w,
            0 => anchor - w / 2.0,
            _ => anchor,
        };
        tx = tx.clamp(MARGIN + 10.0, W - MARGIN - 10.0 - w);
        sheet.label_padded(&txt, tx, baseline + p.1 * s + dxy.1, SMALL_TEXT, gray(), -1);
    }
}

/// The full technical treatment: body, points, handle lengths, extrema.
pub fn annotate(sheet: &mut Sheet, o: &Outline, s: f64, x0: f64, baseline: f64) {
    draw_body(sheet, o, s, x0, baseline);
    draw_points(sheet, o, s, x0, baseline);
    handle_labels(sheet, o, s, x0, baseline);
    extrema_labels(sheet, o, s, x0, baseline);
}

/// The machine/hand legend, one row, right-aligned at (x_right, y baseline).
pub fn legend(sheet: &mut Sheet, x_right: f64, y: f64) {
    let size = LEGEND_TEXT;
    let t2 = "off 8, on 2 = the hand";
    let t1 = "on 8 = machine";
    let w2 = sheet.mono_width(t2, size);
    let w1 = sheet.mono_width(t1, size);
    let x2 = x_right - w2;
    let dot2 = x2 - 26.0;
    let x1 = dot2 - 40.0 - w1;
    let dot1 = x1 - 26.0;
    sheet.label(t2, x2, y, size, red(), -1);
    sheet.label(t1, x1, y, size, green(), -1);
    sheet.ctx.fill(bg()).stroke(red()).stroke_width(PEN);
    sheet.ctx.oval(dot2, y + 1.0, 15.0, 15.0);
    sheet.ctx.fill(bg()).stroke(green()).stroke_width(PEN);
    sheet.ctx.oval(dot1, y + 1.0, 15.0, 15.0);
}

/// A blue knockout node circle at a line intersection (the little circles
/// where cell dividers meet rules and dimension rows).
pub fn node(sheet: &mut Sheet, x: f64, y: f64, r: f64) {
    sheet.ctx.fill(bg()).stroke(blue()).stroke_width(PEN);
    sheet.ctx.oval(x - r, y - r, r * 2.0, r * 2.0);
}

/// Blue advance-boundary dividers: one vertical per boundary, spanning
/// y_top..y_bottom (canvas), with knockout nodes at both ends.
pub fn cell_dividers(sheet: &mut Sheet, xs: &[f64], y_top: f64, y_bottom: f64) {
    for &x in xs {
        sheet.ctx.no_fill().stroke(blue()).stroke_width(PEN);
        sheet.ctx.line(x, y_bottom, x, y_top);
        node(sheet, x, y_top, 7.0);
        node(sheet, x, y_bottom, 7.0);
    }
}

/// Red leader + label calling out one optical-correction point.
pub fn correction_callout(sheet: &mut Sheet, from: (f64, f64), text_at: (f64, f64), align: i8) {
    sheet.ctx.no_fill().stroke(red()).stroke_width(PEN);
    sheet.ctx.line(from.0, from.1, text_at.0, text_at.1 + 8.0);
    let dx = if align < 0 { 10.0 } else { -10.0 };
    sheet.label_padded(
        "off 8, on 2: optical correction",
        text_at.0 + dx,
        text_at.1,
        LEGEND_TEXT,
        red(),
        align,
    );
}

// --- PHASE 4 PLACEHOLDER: native social variants ---------------------------------------------
//
// Planned for the day after the post ships (promotion pass). The idea, so
// the door stays open in this design:
//
//   pub enum SocialFormat { Card, Square, Vertical }
//
//   - Card (2520x1320, 1.91:1) is what every figure renders today; masters
//     double as X/LinkedIn card images.
//   - Square (2048x2048) and Vertical (1080x1920 reels) get NATIVE
//     compositions, not letterboxes: same Sheet/annotation engine, a
//     taller Frame, fewer glyphs per sheet. The virtua-grotesk repo's
//     scripts/designbot/social/render.sh has the ffmpeg fit pipeline to
//     reuse for animated variants (CRF 16, lanczos, BT.709).
//   - export_social.py already numbers masters in post order and extracts
//     alt text; square/vertical exports slot in beside them.
