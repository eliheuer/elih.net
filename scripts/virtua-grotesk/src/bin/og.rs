//! OG / share card for the Virtua Grotesk post on elih.net: 
//
//! Coordinates are DrawBot's (y-up, origin bottom-left), which at this size
//! makes one font unit = one canvas pixel with the baseline at y=324: every
//! font-space coordinate is just BASELINE_Y + value, and the UFO outlines
//! draw with a plain translate. text() anchors the BASELINE at y;
//! rect()/oval() anchor at their bottom-left corner.
//!
//! REBUILD after editing this file (from the elih.net repo root):
//!     cd scripts/virtua-grotesk && cargo run --release --bin og
//!
//! That one command recompiles and overwrites BOTH outputs:
//!     src/content/blog/virtua-grotesk/share-card.png   (post hero)
//!     public/og/virtua-grotesk.png                     (og:image)
//! 
//! Rebuilds take about a second once deps are compiled; reload the post in
//! the browser to see the new card (the Astro dev server serves it as a
//! static asset, no restart needed).
//!
//! Inputs read at render time, from sibling checkouts:
//!     ~/GH/repos/virtua-grotesk/sources/VirtuaGrotesk-Regular.ufo
//!     ~/GH/repos/virtua-grotesk/fonts/variable/VirtuaGrotesk[wght].ttf
//!     ~/GH/repos/google-fonts/ofl/geistmono/GeistMono[wght].ttf

use designbot::prelude::*;
use designbot_render::Renderer;
use kurbo::{Affine, BezPath, Shape};

const W: f64 = 2400.0;
const H: f64 = 1260.0;

const MARGIN: f64 = 96.0; // content runs MARGIN...W-MARGIN
const BASELINE_Y: f64 = 308.0; // canvas y of font y=0
const GRID_TOP: f64 = BASELINE_Y + 784.0; // cap overshoot line
const GRID_BOTTOM: f64 = BASELINE_Y - 80.0;
const HEADER_RULE_Y: f64 = 1160.0;
const FOOTER_RULE_Y: f64 = 112.0;

const GLYPHS: &[&str] = &["G_", "r", "i", "d"];

// Theme tokens
fn bg() -> Color {
    Color::rgb(0x10, 0x10, 0x10)
}
fn grid() -> Color {
    // dark gray, so the graph paper sits well behind the drawing
    Color::rgb(0x32, 0x32, 0x32)
}
fn rule() -> Color {
    Color::rgb(0x15, 0xc4, 0x74)
}
fn text_bright() -> Color {
    Color::rgb(0x15, 0xc4, 0x74)
}
fn subdued() -> Color {
    Color::rgb(0x15, 0xc4, 0x74)
}
fn red() -> Color {
    Color::rgb(0xff, 0x45, 0x35)
}
fn blue() -> Color {
    Color::rgb(0x4a, 0x78, 0xff)
}
fn red_fill() -> Color {
    // the mark red at ~40%, so grid and construction lines read through
    Color::rgba(0xff, 0x45, 0x35, 104)
}

// --- minimal sfnt reader (family name for ctx.font()) ----------------------

fn read_u16(data: &[u8], offset: usize) -> u16 {
    u16::from_be_bytes([data[offset], data[offset + 1]])
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

/// Load the font into the renderer and return its Windows-platform family
/// name (id 16 falling back to 1) for ctx.font().
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
    panic!("no Windows family name record in {path}");
}

// --- UFO outlines -----------------------------------------------------------

/// Point roles, drawn as red-outlined markers knocked out with the background
/// color: smooth = circle, corner = square, off-curve = small circle.
#[derive(Clone, Copy)]
enum Role {
    Smooth,
    Corner,
    Off,
}

struct Outline {
    path: BezPath, // font units, y-up, same as the canvas
    points: Vec<(f64, f64, Role)>,
    handles: Vec<((f64, f64), (f64, f64))>, // on-curve anchor -> off-curve
    width: f64,
    lsb: f64,
    rsb: f64,
}

fn push_on_curve(points: &mut Vec<(f64, f64, Role)>, p: &norad::ContourPoint, k: usize, n: usize) {
    if k == n {
        return; // the closing point is the start point, already recorded
    }
    let role = if p.smooth { Role::Smooth } else { Role::Corner };
    points.push((p.x, p.y, role));
}

fn load_outline(glif: &std::path::Path) -> Outline {
    let glyph = norad::Glyph::load(glif).expect("failed to load glif");
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
        let role = if sp.smooth { Role::Smooth } else { Role::Corner };
        points.push((sp.x, sp.y, role));
        let mut prev_on = (sp.x, sp.y);
        let mut pending: Vec<(f64, f64)> = Vec::new();
        for k in 1..=n {
            let p = &pts[(start + k) % n];
            match p.typ {
                OffCurve => {
                    pending.push((p.x, p.y));
                    points.push((p.x, p.y, Role::Off));
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
    let bounds = path.bounding_box();
    Outline {
        lsb: bounds.x0,
        rsb: glyph.width - bounds.x1,
        path,
        points,
        handles,
        width: glyph.width,
    }
}

// --- drawing helpers ----------------------------------------------------------

struct Sheet<'a> {
    ctx: Canvas,
    renderer: &'a Renderer,
    mono: String,
}

impl Sheet<'_> {
    fn mono_width(&self, txt: &str, size: f64) -> f64 {
        self.renderer.text_width(txt, Some(&self.mono), size, &[])
    }

    /// Mono label with its baseline at y. align: -1 left, 0 center, 1 right.
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

    /// Label over a background patch so it stays legible on the grid.
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
            .rect(x0 - pad, y - size * 0.28, w + pad * 2.0, size * 1.14);
        self.label(txt, x0, y, size, color, -1);
    }

    /// Metric-line tag: a background-filled, blue-outlined box snapped to the
    /// 16-unit grid (32 high, width in whole cells), floating one grid unit
    /// off the line at y_line (above or below it), with the text optically
    /// centered. x_edge must be a grid line; align -1 grows the box
    /// rightward, 1 leftward.
    fn metric_tag(&mut self, txt: &str, x_edge: f64, y_line: f64, above: bool, align: i8) {
        let size = 30.0;
        let w = self.mono_width(txt, size);
        let box_w = ((w + 16.0) / 16.0).ceil() * 16.0;
        let box_h = 32.0;
        let x0 = if align < 0 { x_edge } else { x_edge - box_w };
        let y0 = if above {
            y_line + 16.0
        } else {
            y_line - box_h - 16.0
        };
        self.ctx.fill(bg()).stroke(blue()).stroke_width(2.5);
        self.ctx.rect(x0, y0, box_w, box_h);
        // Geist Mono caps/figures are ~0.73 em tall; center that ink box
        let baseline = y0 + (box_h - 0.73 * size) / 2.0;
        self.label(txt, x0 + box_w / 2.0, baseline, size, blue(), 0);
    }

    /// 45-degree hatching clipped to a rect, Replica side-bearing style.
    fn hatch(&mut self, x0: f64, y0: f64, x1: f64, y1: f64, color: Color) {
        let h = y1 - y0;
        self.ctx.stroke(color).stroke_width(2.5).no_fill();
        let step = 6.0;
        let mut t = x0 - h;
        while t < x1 {
            // segment from (t, y0) to (t + h, y1), clipped to [x0, x1]
            let sx = t.max(x0);
            let ex = (t + h).min(x1);
            if ex > sx {
                self.ctx.line(sx, y0 + (sx - t), ex, y0 + (ex - t));
            }
            t += step;
        }
    }

    /// Small circle node at a blue-line crossing, knocked out with the
    /// background color like the point markers.
    fn node(&mut self, x: f64, y: f64, r: f64) {
        self.ctx.fill(bg()).stroke(blue()).stroke_width(2.5);
        self.ctx.oval(x - r, y - r, r * 2.0, r * 2.0);
    }
}

fn main() {
    let home = std::env::var("HOME").unwrap();
    let glyphs_dir = std::path::PathBuf::from(&home)
        .join("GH/repos/virtua-grotesk/sources/VirtuaGrotesk-Regular.ufo/glyphs");
    let mono_path = format!("{home}/GH/repos/google-fonts/ofl/geistmono/GeistMono[wght].ttf");

    let outlines: Vec<Outline> = GLYPHS
        .iter()
        .map(|name| load_outline(&glyphs_dir.join(format!("{name}.glif"))))
        .collect();
    let total_advance: f64 = outlines.iter().map(|o| o.width).sum();

    let mut renderer = Renderer::new(W as u32, H as u32);
    let mono = load_family(&mut renderer, &mono_path);

    let mut sheet = Sheet {
        ctx: Canvas::new(W, H),
        renderer: &renderer,
        mono,
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

    // ── the 16-unit design grid, aligned to the glyph origin and snapped to
    //    whole cells: the box starts and ends exactly on grid lines ──
    let step = 16.0;
    let grid_left = x0 - (((x0 - MARGIN) / step).floor()) * step;
    let grid_right = grid_left + (((W - MARGIN - grid_left) / step).floor()) * step;
    {
        let ctx = &mut sheet.ctx;
        ctx.no_fill();
        ctx.stroke(grid()).stroke_width(2.5);
        let mut x = grid_left;
        while x <= grid_right {
            ctx.line(x, GRID_BOTTOM, x, GRID_TOP);
            x += step;
        }
        let mut y = GRID_BOTTOM;
        while y <= GRID_TOP {
            ctx.line(grid_left, y, grid_right, y);
            y += step;
        }
    }

    // ── per-glyph cells: advance-boundary verticals ──
    {
        let ctx = &mut sheet.ctx;
        ctx.stroke(blue()).stroke_width(2.5).no_fill();
        for &b in &bounds {
            ctx.line(b, GRID_BOTTOM, b, GRID_TOP);
        }
    }

    // ── vertical metrics, behind the glyphs:
    //    overshoots dashed, cap/x-height/baseline solid ──
    {
        let ctx = &mut sheet.ctx;
        ctx.stroke(blue()).stroke_width(2.5).no_fill();
        ctx.line_dash(&[10.0, 10.0]);
        for y in [784.0, -16.0] {
            ctx.line(grid_left, BASELINE_Y + y, grid_right, BASELINE_Y + y);
        }
        ctx.line_dash(&[]);
        for y in [768.0, 576.0, 0.0] {
            ctx.line(grid_left, BASELINE_Y + y, grid_right, BASELINE_Y + y);
        }
    }

    // ── glyphs: half-transparent mark-red fill with a solid contour stroke,
    //    Replica gauge-ball-page style; canvas and font agree on y-up, so
    //    placement is a plain translate ──
    for (o, w) in outlines.iter().zip(bounds.windows(2)) {
        sheet.ctx.fill(red_fill()).stroke(red()).stroke_width(2.5);
        sheet
            .ctx
            .draw_path(Affine::translate((w[0], BASELINE_Y)) * o.path.clone());
    }

    // ── bezier handles and points over everything, Runebender palette ──
    for (o, w) in outlines.iter().zip(bounds.windows(2)) {
        let (gx, gy) = (w[0], BASELINE_Y);
        sheet.ctx.stroke(red()).stroke_width(2.5).no_fill();
        for ((x1, y1), (x2, y2)) in &o.handles {
            sheet.ctx.line(gx + x1, gy + y1, gx + x2, gy + y2);
        }
        // red markers knocked out with the background color so they stay
        // readable over the fill and grid
        sheet.ctx.fill(bg()).stroke(red()).stroke_width(2.5);
        for (x, y, role) in &o.points {
            let (px, py) = (gx + x, gy + y);
            match role {
                Role::Smooth => {
                    sheet.ctx.oval(px - 7.0, py - 7.0, 14.0, 14.0);
                }
                Role::Corner => {
                    sheet.ctx.rect(px - 6.0, py - 6.0, 12.0, 12.0);
                }
                Role::Off => {
                    sheet.ctx.oval(px - 7.0, py - 7.0, 14.0, 14.0);
                }
            }
        }
    }

    // ── metric tags, snapped to the grid and docked onto their lines ──
    sheet.metric_tag("CAP 768", grid_left, BASELINE_Y + 768.0, false, -1);
    sheet.metric_tag("X-HEIGHT 576", grid_left, BASELINE_Y + 576.0, true, -1);
    sheet.metric_tag("BASELINE 0", grid_left, BASELINE_Y, true, -1);
    sheet.metric_tag("OVERSHOOT +16", grid_right, BASELINE_Y + 784.0, true, 1);
    sheet.metric_tag("OVERSHOOT -16", grid_right, BASELINE_Y - 16.0, false, 1);

    // ── dimension zone: staggered width / side-bearing rows, then boundary
    //    ticks whose end nodes land on the bottom corner of the deepest
    //    adjacent hatch block ──
    fn row_y(j: usize) -> f64 {
        if j % 2 == 0 {
            188.0
        } else {
            152.0
        }
    }
    for (j, (o, w)) in outlines.iter().zip(bounds.windows(2)).enumerate() {
        let (bx0, bx1) = (w[0], w[1]);
        let row_y = row_y(j);
        let ink0 = bx0 + o.lsb;
        let ink1 = bx1 - o.rsb;

        // dim line across the cell, hatched side-bearing blocks at the ends
        sheet
            .ctx
            .stroke(subdued())
            .stroke_width(2.5)
            .no_fill()
            .line(bx0, row_y, bx1, row_y);
        sheet.hatch(bx0, row_y - 14.0, ink0, row_y + 14.0, red());
        sheet.hatch(ink1, row_y - 14.0, bx1, row_y + 14.0, red());

        // ink width, bright, centered; side bearings, red, tucked at the ends
        sheet.label_padded(
            &format!("{}", (ink1 - ink0).round()),
            (ink0 + ink1) / 2.0,
            row_y - 11.0,
            30.0,
            text_bright(),
            0,
        );
        sheet.label(
            &format!("{}", o.lsb.round()),
            ink0 + 10.0,
            row_y + 22.0,
            30.0,
            red(),
            -1,
        );
        sheet.label(
            &format!("{}", o.rsb.round()),
            ink1 - 10.0,
            row_y + 22.0,
            30.0,
            red(),
            1,
        );
    }

    // ── boundary ticks + nodes, over the hatches ──
    for (i, &b) in bounds.iter().enumerate() {
        // deepest hatch touching this boundary: the cells to its left/right
        let mut tick_end = f64::INFINITY;
        if i > 0 {
            tick_end = tick_end.min(row_y(i - 1) - 14.0);
        }
        if i < outlines.len() {
            tick_end = tick_end.min(row_y(i) - 14.0);
        }
        sheet
            .ctx
            .stroke(blue())
            .stroke_width(2.5)
            .no_fill()
            .line(b, GRID_BOTTOM, b, tick_end);
        sheet.node(b, tick_end, 6.0);
        // nodes where the boundary crosses the blue metric lines
        for y in [784.0, 768.0, 576.0, 0.0, -16.0] {
            sheet.node(b, BASELINE_Y + y, 6.0);
        }
    }

    // ── header: Replica legend + section badge ──
    {
        let base_y = 1182.0;
        let size = 30.0;
        sheet.label("VIRTUA GROTESK", MARGIN, base_y, size, text_bright(), -1);

        sheet.label(
            "SIL OPEN FONT LICENSE (OFL) VERSION 1.1",
            W - MARGIN,
            base_y,
            size,
            text_bright(),
            1,
        );
    }

    // ── rules and footer captions ──
    {
        let ctx = &mut sheet.ctx;
        ctx.stroke(rule()).stroke_width(2.5).no_fill();
        ctx.line(MARGIN, HEADER_RULE_Y, W - MARGIN, HEADER_RULE_Y);
        ctx.line(MARGIN, FOOTER_RULE_Y, W - MARGIN, FOOTER_RULE_Y);
    }
    sheet.label(
        "POWERS OF TWO GRID / REGULAR 400 / UPM 1024",
        MARGIN,
        64.0,
        30.0,
        text_bright(),
        -1,
    );
    sheet.label(
        "GITHUB.COM/ELIHEUER/VIRTUA-GROTESK",
        W - MARGIN,
        64.0,
        30.0,
        text_bright(),
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

    // social variant for posting: sRGB-tagged, saturation pre-compensated
    // for 4:2:0 chroma subsampling, coarse grain, PNG-passthrough pixel
    let social = format!("{home}/Desktop/virtua-grotesk-og-social.png");
    renderer.render_to_png_social(&sheet.ctx, &social).unwrap();
    println!("wrote {social}");
}
