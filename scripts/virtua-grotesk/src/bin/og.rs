//! OG / share card for the Virtua Grotesk post, in the manner of Norm's
//! Replica specimen dimension sheets, translated to dark mode with the
//! runebender-web theme palette: the word "Grid" from the Regular master
//! filled in mark-red over the 16-unit design grid, blue vertical-metric
//! lines with values, per-glyph advance widths and hatched side bearings
//! in staggered dimension rows below. 2400x1260 (2x of 1200x630).
//!
//! At this size one font unit = one canvas pixel, so the drawing grid IS
//! the design grid. Reads glyphs from the sibling checkout
//! ~/GH/repos/virtua-grotesk. Run from scripts/virtua-grotesk:
//!
//!     cargo run --release --bin og
//!
//! Writes ../../src/content/blog/virtua-grotesk/share-card.png and
//! ../../public/og/virtua-grotesk.png.

use designbot::prelude::*;
use designbot_render::Renderer;
use kurbo::{Affine, BezPath, Shape};

const W: f64 = 2400.0;
const H: f64 = 1260.0;

const MARGIN: f64 = 96.0; // content runs MARGIN..W-MARGIN
const BASELINE_Y: f64 = 936.0; // canvas y of font y=0
const GRID_TOP: f64 = 152.0; // font y = 784 (cap overshoot line)
const GRID_BOTTOM: f64 = 1016.0; // font y = -80
const HEADER_RULE_Y: f64 = 110.0;
const FOOTER_RULE_Y: f64 = 1204.0;

const GLYPHS: &[&str] = &["G_", "r", "i", "d"];

// runebender-web theme tokens (themeTokens.ts / runebender.json)
fn bg() -> Color {
    Color::rgb(0x10, 0x10, 0x10)
}
fn grid_minor() -> Color {
    Color::rgb(0x1f, 0x1f, 0x1f)
}
fn grid_major() -> Color {
    Color::rgb(0x2d, 0x2d, 0x2d)
}
fn rule() -> Color {
    Color::rgb(0x40, 0x40, 0x40)
}
fn diag() -> Color {
    Color::rgb(0x38, 0x38, 0x38)
}
fn text_primary() -> Color {
    Color::rgb(0x90, 0x90, 0x90)
}
fn text_bright() -> Color {
    Color::rgb(0xf0, 0xf0, 0xf0)
}
fn text_dim() -> Color {
    Color::rgb(0x70, 0x70, 0x70)
}
fn subdued() -> Color {
    Color::rgb(0x50, 0x50, 0x50)
}
fn red() -> Color {
    Color::rgb(0xff, 0x4a, 0x3d)
}
fn blue() -> Color {
    Color::rgb(0x45, 0x6f, 0xff)
}
fn green() -> Color {
    Color::rgb(0x18, 0xb8, 0x6f)
}

// --- minimal sfnt readers (family name + baseline metrics) -----------------

fn read_u16(data: &[u8], offset: usize) -> u16 {
    u16::from_be_bytes([data[offset], data[offset + 1]])
}

fn read_i16(data: &[u8], offset: usize) -> i16 {
    i16::from_be_bytes([data[offset], data[offset + 1]])
}

fn read_u32(data: &[u8], offset: usize) -> u32 {
    u32::from_be_bytes([
        data[offset],
        data[offset + 1],
        data[offset + 2],
        data[offset + 3],
    ])
}

fn find_table(data: &[u8], tag: &[u8; 4]) -> Option<usize> {
    let num_tables = read_u16(data, 4) as usize;
    (0..num_tables)
        .map(|i| 12 + i * 16)
        .find(|&rec| &data[rec..rec + 4] == tag)
        .map(|rec| read_u32(data, rec + 8) as usize)
}

/// Windows-platform UTF-16 name string (id 16 falling back to 1).
fn family_name(data: &[u8]) -> String {
    let name = find_table(data, b"name").expect("no name table");
    let count = read_u16(data, name + 2) as usize;
    let string_off = name + read_u16(data, name + 4) as usize;
    for want in [16u16, 1] {
        for i in 0..count {
            let rec = name + 6 + i * 12;
            if read_u16(data, rec) == 3 && read_u16(data, rec + 6) == want {
                let len = read_u16(data, rec + 8) as usize;
                let off = string_off + read_u16(data, rec + 10) as usize;
                let units: Vec<u16> = data[off..off + len]
                    .chunks_exact(2)
                    .map(|c| u16::from_be_bytes([c[0], c[1]]))
                    .collect();
                return String::from_utf16_lossy(&units);
            }
        }
    }
    panic!("no Windows family name record");
}

struct LabelFont {
    family: String,
    asc: f64,  // em fraction
    desc: f64, // em fraction, positive down
}

impl LabelFont {
    fn load(renderer: &mut Renderer, path: &str) -> LabelFont {
        let data = std::fs::read(path).unwrap_or_else(|e| panic!("read {path}: {e}"));
        renderer
            .load_font(path)
            .unwrap_or_else(|e| panic!("load {path}: {e:?}"));
        let hhea = find_table(&data, b"hhea").expect("no hhea");
        let head = find_table(&data, b"head").expect("no head");
        let upm = read_u16(&data, head + 18) as f64;
        LabelFont {
            family: family_name(&data),
            asc: read_i16(&data, hhea + 4) as f64 / upm,
            desc: -read_i16(&data, hhea + 6) as f64 / upm,
        }
    }

    /// Distance from designbot's text() y (top of line) down to the first
    /// baseline, matching parley's rounding (see social_images.rs).
    fn baseline_offset(&self, size: f64) -> f64 {
        let asc = (self.asc * size).round();
        let desc = (self.desc * size).round();
        let lh = size.round();
        (asc + (lh - asc - desc) * 0.5).round()
    }
}

// --- UFO outlines -----------------------------------------------------------

struct Outline {
    path: BezPath, // font units, y-up
    width: f64,
    lsb: f64,
    rsb: f64,
}

fn load_outline(glif: &std::path::Path) -> Outline {
    let glyph = norad::Glyph::load(glif).expect("failed to load glif");
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
    let bounds = path.bounding_box();
    Outline {
        lsb: bounds.x0,
        rsb: glyph.width - bounds.x1,
        path,
        width: glyph.width,
    }
}

// --- drawing helpers ----------------------------------------------------------

struct Sheet<'a> {
    ctx: Canvas,
    renderer: &'a Renderer,
    mono: &'a LabelFont,
}

impl Sheet<'_> {
    fn mono_width(&self, txt: &str, size: f64) -> f64 {
        self.renderer
            .text_width(txt, Some(&self.mono.family), size, &[])
    }

    /// Mono label with its BASELINE at y. align: -1 left, 0 center, 1 right.
    fn label(&mut self, txt: &str, x: f64, y: f64, size: f64, color: Color, align: i8) {
        let w = self.mono_width(txt, size);
        let x = match align {
            -1 => x,
            0 => x - w / 2.0,
            _ => x - w,
        };
        self.ctx
            .font(&self.mono.family)
            .clear_font_variations()
            .font_size(size)
            .fill(color)
            .text_align(TextAlign::Left)
            .text(txt, x, y - self.mono.baseline_offset(size));
    }

    /// Label over a background patch so it stays legible on the grid.
    /// Returns the padded background width.
    fn label_padded(&mut self, txt: &str, x: f64, y: f64, size: f64, color: Color, align: i8) {
        let w = self.mono_width(txt, size);
        let pad = 10.0;
        let x0 = match align {
            -1 => x,
            0 => x - w / 2.0,
            _ => x - w,
        };
        self.ctx.no_stroke().fill(bg());
        self.ctx
            .rect(x0 - pad, y - size * 0.86, w + pad * 2.0, size * 1.14);
        self.label(txt, x0, y, size, color, -1);
    }

    /// 45-degree hatching clipped to a rect, Replica side-bearing style.
    fn hatch(&mut self, x0: f64, y0: f64, x1: f64, y1: f64, color: Color) {
        let h = y1 - y0;
        self.ctx.stroke(color).stroke_width(1.6).no_fill();
        let step = 6.0;
        let mut t = x0 - h;
        while t < x1 {
            // segment from (t, y1) to (t + h, y0), clipped to [x0, x1]
            let sx = t.max(x0);
            let ex = (t + h).min(x1);
            if ex > sx {
                self.ctx.line(sx, y1 - (sx - t), ex, y1 - (ex - t));
            }
            t += step;
        }
    }

    /// Small open circle node, Replica corner-marker style.
    fn node(&mut self, x: f64, y: f64, r: f64) {
        self.ctx.no_fill().stroke(text_primary()).stroke_width(1.6);
        self.ctx.oval(x - r, y - r, r * 2.0, r * 2.0);
    }
}

fn main() {
    let home = std::env::var("HOME").unwrap();
    let glyphs_dir = std::path::PathBuf::from(&home)
        .join("GH/repos/virtua-grotesk/sources/VirtuaGrotesk-Regular.ufo/glyphs");
    let vf_path = format!("{home}/GH/repos/virtua-grotesk/fonts/variable/VirtuaGrotesk[wght].ttf");
    let mono_path = format!("{home}/GH/repos/google-fonts/ofl/geistmono/GeistMono[wght].ttf");

    let outlines: Vec<Outline> = GLYPHS
        .iter()
        .map(|name| load_outline(&glyphs_dir.join(format!("{name}.glif"))))
        .collect();
    let total_advance: f64 = outlines.iter().map(|o| o.width).sum();

    let mut renderer = Renderer::new(W as u32, H as u32);
    let mono = LabelFont::load(&mut renderer, &mono_path);
    let virtua = LabelFont::load(&mut renderer, &vf_path);

    let mut sheet = Sheet {
        ctx: Canvas::new(W, H),
        renderer: &renderer,
        mono: &mono,
    };
    sheet.ctx.background(bg());

    // one font unit = one pixel; center the run
    let x0 = ((W - total_advance) / 2.0).round();
    // advance boundaries, canvas x
    let mut bounds = vec![x0];
    let mut cursor = x0;
    for o in &outlines {
        cursor += o.width;
        bounds.push(cursor);
    }

    // ── the 16-unit design grid, aligned to the glyph origin ──
    {
        let ctx = &mut sheet.ctx;
        ctx.no_fill();
        for major in [false, true] {
            let (color, step) = if major {
                (grid_major(), 64.0)
            } else {
                (grid_minor(), 16.0)
            };
            ctx.stroke(color).stroke_width(1.0);
            let mut x = x0 - (((x0 - MARGIN) / step).floor()) * step;
            while x <= W - MARGIN {
                ctx.line(x, GRID_TOP, x, GRID_BOTTOM);
                x += step;
            }
            let mut y = BASELINE_Y - (((BASELINE_Y - GRID_TOP) / step).floor()) * step;
            while y <= GRID_BOTTOM {
                ctx.line(MARGIN, y, W - MARGIN, y);
                y += step;
            }
        }
    }

    // ── per-glyph cells: boundary verticals, corner-to-corner diagonals ──
    {
        let ctx = &mut sheet.ctx;
        ctx.stroke(diag()).stroke_width(1.4).no_fill();
        for w in bounds.windows(2) {
            let (bx0, bx1) = (w[0], w[1]);
            ctx.line(bx0, GRID_TOP, bx1, GRID_BOTTOM);
            ctx.line(bx0, GRID_BOTTOM, bx1, GRID_TOP);
        }
        ctx.stroke(rule()).stroke_width(1.6);
        for &b in &bounds {
            ctx.line(b, GRID_TOP, b, GRID_BOTTOM);
        }
    }

    // ── glyphs, mark-red ──
    for (o, w) in outlines.iter().zip(bounds.windows(2)) {
        let to_canvas = Affine::new([1.0, 0.0, 0.0, -1.0, w[0], BASELINE_Y]);
        sheet.ctx.no_stroke().fill(red());
        sheet.ctx.draw_path(to_canvas * o.path.clone());
    }

    // ── vertical metrics: overshoots dashed, cap/x-height/baseline solid ──
    {
        let ctx = &mut sheet.ctx;
        ctx.stroke(blue()).stroke_width(1.6).no_fill();
        for y in [BASELINE_Y - 784.0, BASELINE_Y + 16.0] {
            let (dash, gap) = (10.0, 10.0);
            let mut x = MARGIN;
            while x < W - MARGIN {
                ctx.line(x, y, (x + dash).min(W - MARGIN), y);
                x += dash + gap;
            }
        }
        ctx.stroke_width(2.2);
        for y in [768.0, 576.0, 0.0] {
            ctx.line(MARGIN, BASELINE_Y - y, W - MARGIN, BASELINE_Y - y);
        }
    }

    // ── cell corner nodes over the glyphs ──
    for &b in &bounds {
        sheet.node(b, GRID_TOP, 6.0);
        sheet.node(b, GRID_BOTTOM, 6.0);
    }

    // ── metric labels, left, over background patches ──
    for (txt, y) in [
        ("CAP 768", BASELINE_Y - 768.0 + 34.0),
        ("X-HEIGHT 576", BASELINE_Y - 576.0 - 12.0),
        ("BASELINE 0", BASELINE_Y - 12.0),
    ] {
        sheet.label_padded(txt, MARGIN + 8.0, y, 26.0, blue(), -1);
    }
    sheet.label_padded(
        "OVERSHOOT +16",
        W - MARGIN - 8.0,
        BASELINE_Y - 784.0 - 10.0,
        26.0,
        blue(),
        1,
    );
    sheet.label_padded(
        "OVERSHOOT -16",
        W - MARGIN - 8.0,
        BASELINE_Y + 16.0 + 34.0,
        26.0,
        blue(),
        1,
    );

    // ── dimension zone: boundary ticks + staggered width / side-bearing rows ──
    let tick_bottom = 1168.0;
    {
        let ctx = &mut sheet.ctx;
        ctx.stroke(subdued()).stroke_width(1.6).no_fill();
        for &b in &bounds {
            ctx.line(b, GRID_BOTTOM, b, tick_bottom);
        }
    }
    for (j, (o, w)) in outlines.iter().zip(bounds.windows(2)).enumerate() {
        let (bx0, bx1) = (w[0], w[1]);
        let row_y = if j % 2 == 0 { 1064.0 } else { 1132.0 };
        let ink0 = bx0 + o.lsb;
        let ink1 = bx1 - o.rsb;

        // dim line across the cell, hatched side-bearing blocks at the ends
        sheet
            .ctx
            .stroke(subdued())
            .stroke_width(1.6)
            .no_fill()
            .line(bx0, row_y, bx1, row_y);
        sheet.hatch(bx0, row_y - 14.0, ink0, row_y + 14.0, red());
        sheet.hatch(ink1, row_y - 14.0, bx1, row_y + 14.0, red());

        // ink width, bright, centered; side bearings, red, tucked at the ends
        sheet.label_padded(
            &format!("{}", (ink1 - ink0).round()),
            (ink0 + ink1) / 2.0,
            row_y + 11.0,
            32.0,
            text_bright(),
            0,
        );
        sheet.label(
            &format!("{}", o.lsb.round()),
            ink0 + 10.0,
            row_y - 22.0,
            24.0,
            red(),
            -1,
        );
        sheet.label(
            &format!("{}", o.rsb.round()),
            ink1 - 10.0,
            row_y - 22.0,
            24.0,
            red(),
            1,
        );
    }

    // ── header: Replica legend + section badge ──
    {
        let base_y = 78.0;
        let size = 30.0;
        let mut x = MARGIN;
        let segments: [(&str, Color, Option<Color>); 4] = [
            ("POWERS-OF-TWO GRID", text_bright(), None),
            ("WIDTH", text_primary(), Some(text_bright())),
            ("SIDE BEARINGS", text_primary(), Some(red())),
            ("METRICS", text_primary(), Some(blue())),
        ];
        for (i, (txt, color, square)) in segments.iter().enumerate() {
            if i > 0 {
                sheet.label("/", x, base_y, size, subdued(), -1);
                x += sheet.mono_width("/", size) + 6.0;
            }
            if let Some(sq) = square {
                sheet.ctx.no_stroke().fill(*sq);
                sheet.ctx.rect(x, base_y - 20.0, 20.0, 20.0);
                x += 30.0;
            }
            sheet.label(txt, x, base_y, size, *color, -1);
            x += sheet.mono_width(txt, size) + 22.0;
        }

        // badge: green box with a Virtua "a", sheet number beside it
        let box_s = 56.0;
        let bx = W - MARGIN - box_s - 66.0;
        let by = base_y - 44.0;
        sheet.ctx.no_stroke().fill(green());
        sheet.ctx.rect(bx, by, box_s, box_s);
        sheet
            .ctx
            .font(&virtua.family)
            .clear_font_variations()
            .font_variation("wght", 500.0)
            .font_size(40.0)
            .fill(bg())
            .text_align(TextAlign::Center)
            .text(
                "a",
                bx + box_s / 2.0,
                by + box_s / 2.0 + 15.0 - virtua.baseline_offset(40.0),
            );
        sheet.ctx.text_align(TextAlign::Left);
        sheet.label("01", W - MARGIN, base_y, size, text_primary(), 1);
    }

    // ── rules and footer captions ──
    {
        let ctx = &mut sheet.ctx;
        ctx.stroke(rule()).stroke_width(2.0).no_fill();
        ctx.line(MARGIN, HEADER_RULE_Y, W - MARGIN, HEADER_RULE_Y);
        ctx.line(MARGIN, FOOTER_RULE_Y, W - MARGIN, FOOTER_RULE_Y);
    }
    sheet.label(
        "VIRTUA GROTESK / REGULAR 400 / UPM 1024",
        MARGIN,
        1240.0,
        24.0,
        text_dim(),
        -1,
    );
    sheet.label(
        "github.com/eliheuer/virtua-grotesk",
        W - MARGIN,
        1240.0,
        24.0,
        text_dim(),
        1,
    );

    let here = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let repo = here.parent().unwrap().parent().unwrap();
    for out in [
        repo.join("src/content/blog/virtua-grotesk/share-card.png"),
        repo.join("public/og/virtua-grotesk.png"),
    ] {
        std::fs::create_dir_all(out.parent().unwrap()).unwrap();
        renderer
            .render_to_png(&sheet.ctx, out.to_str().unwrap())
            .unwrap();
        println!("wrote {}", out.display());
    }
}
