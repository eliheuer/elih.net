//! Flat-vs-nested grid comparison for the Virtua Grotesk post, §10.
//!
//! Two panels, both showing the SAME zoomed crop of the lowercase a's lower
//! bowl from the Regular UFO — the counter whose left extremum is pulled to
//! x=116, on the 2-unit grid but off the 8-unit grid (three points: one
//! smooth anchor, two off-curve handles).
//!
//!   Panel 01, flat grid (Replica's scheme): one lattice, one color; every
//!   point renders identical gray, and the off-grid cluster is tagged
//!   "correction or mistake?" — the data can't say.
//!
//!   Panel 02, nested grid (Virtua): the 2 / 8 / 64 levels drawn at three
//!   intensities; on-8 points green (machine structure), the x=116 cluster
//!   red (the hand) — the level a point sits on labels the decision.
//!
//! Run from this directory:
//!
//!     cargo run --release --bin grids
//!
//! Writes ../../src/content/blog/virtua-grotesk/fig-grid-labels.png.
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
const GAP: f64 = 96.0;
const SLOT: f64 = (W - 2.0 * MARGIN - GAP) / 2.0; // 1116
const HEADER_RULE_Y: f64 = 1224.0;
const FOOTER_RULE_Y: f64 = 96.0;

// The crop, in font units, and its scale on the canvas.
const UX0: f64 = 64.0;
const UX1: f64 = 464.0;
const UY0: f64 = -16.0;
const UY1: f64 = 304.0;
const S: f64 = 2.7; // canvas px per font unit -> 1080 x 864 panel
const PANEL_BOTTOM: f64 = 195.0;
const PANEL_TOP: f64 = PANEL_BOTTOM + (UY1 - UY0) * S;
const INSET_X: f64 = (SLOT - (UX1 - UX0) * S) / 2.0;

// Theme tokens, shared with og.rs / figs.rs.
fn bg() -> Color {
    Color::rgb(0x10, 0x10, 0x10)
}
fn grid_flat() -> Color {
    Color::rgb(0x30, 0x30, 0x30)
}
fn grid_2() -> Color {
    Color::rgb(0x1a, 0x1a, 0x1a)
}
fn grid_8() -> Color {
    Color::rgb(0x2c, 0x2c, 0x2c)
}
fn grid_64() -> Color {
    Color::rgb(0x42, 0x42, 0x42)
}
fn border() -> Color {
    Color::rgb(0x4a, 0x4a, 0x4a)
}
fn curve() -> Color {
    Color::rgb(230, 230, 230)
}
fn curve_fill() -> Color {
    Color::rgba(230, 230, 230, 20)
}
fn handle() -> Color {
    Color::rgb(0x6a, 0x6a, 0x6a)
}
fn green() -> Color {
    Color::rgb(0x15, 0xc4, 0x74)
}
fn red() -> Color {
    Color::rgb(0xff, 0x45, 0x35)
}
fn blue() -> Color {
    Color::rgb(0x4a, 0x78, 0xff)
}
fn gray() -> Color {
    Color::rgb(0x8a, 0x8a, 0x8a)
}

// --- UFO outline loading (same as og.rs) -------------------------------------

#[derive(Clone, Copy)]
enum PtRole {
    Smooth,
    Corner,
    Off,
}

struct Outline {
    path: BezPath,
    points: Vec<(f64, f64, PtRole)>,
    handles: Vec<((f64, f64), (f64, f64))>,
}

fn push_on_curve(points: &mut Vec<(f64, f64, PtRole)>, p: &norad::ContourPoint, k: usize, n: usize) {
    if k == n {
        return;
    }
    let role = if p.smooth { PtRole::Smooth } else { PtRole::Corner };
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
    Outline { path, points, handles }
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
        self.ctx.fill(bg()).no_stroke();
        self.ctx
            .rect(x0 - pad, y - 0.28 * size, w + 2.0 * pad, 1.3 * size);
        self.label(txt, x0, y, size, color, -1);
    }
}

/// Panel-local transform: font units -> canvas.
fn cx(panel_left: f64, ux: f64) -> f64 {
    panel_left + (ux - UX0) * S
}
fn cy(uy: f64) -> f64 {
    PANEL_BOTTOM + (uy - UY0) * S
}

fn on8(v: f64) -> bool {
    v.rem_euclid(8.0) == 0.0
}

fn main() {
    let home = std::env::var("HOME").unwrap();
    let mono_path = format!("{home}/GH/repos/google-fonts/ofl/geistmono/GeistMono[wght].ttf");
    let glif = std::path::PathBuf::from(&home)
        .join("GH/repos/virtua-grotesk/sources/VirtuaGrotesk-Regular.ufo/glyphs/a.glif");

    let mut renderer = Renderer::new(W as u32, H as u32);
    let mono = load_family(&mut renderer, &mono_path);
    let outline = load_outline(&glif);

    let here = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let out = here
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("src/content/blog/virtua-grotesk/fig-grid-labels.png");

    let mut sheet = Sheet {
        ctx: Canvas::new(W, H),
        renderer: &renderer,
        mono: mono.clone(),
    };
    sheet.ctx.background(bg());

    let panel_left = |i: usize| MARGIN + i as f64 * (SLOT + GAP) + INSET_X;
    let panel_w = (UX1 - UX0) * S;

    // ── grids, per panel ──
    for i in 0..2 {
        let pl = panel_left(i);
        let draw_lattice = |step: f64, color: Color, width: f64, ctx: &mut Canvas| {
            ctx.no_fill().stroke(color).stroke_width(width);
            let mut ux = UX0;
            while ux <= UX1 {
                ctx.line(cx(pl, ux), cy(UY0), cx(pl, ux), cy(UY1));
                ux += step;
            }
            let mut uy = (UY0 / step).ceil() * step;
            while uy <= UY1 {
                ctx.line(cx(pl, UX0), cy(uy), cx(pl, UX1), cy(uy));
                uy += step;
            }
        };
        if i == 0 {
            // flat: one lattice, one color
            draw_lattice(8.0, grid_flat(), 2.0, &mut sheet.ctx);
        } else {
            // nested: three levels at three intensities
            draw_lattice(2.0, grid_2(), 1.0, &mut sheet.ctx);
            draw_lattice(8.0, grid_8(), 2.0, &mut sheet.ctx);
            draw_lattice(64.0, grid_64(), 2.5, &mut sheet.ctx);
        }
        // baseline, house blue
        sheet.ctx.no_fill().stroke(blue()).stroke_width(2.5);
        sheet.ctx.line(cx(pl, UX0), cy(0.0), cx(pl, UX1), cy(0.0));
    }

    // ── the outline, identical in both panels ──
    for i in 0..2 {
        let pl = panel_left(i);
        let place = Affine::new([S, 0.0, 0.0, S, pl - UX0 * S, PANEL_BOTTOM - UY0 * S]);
        sheet.ctx.fill(curve_fill()).stroke(curve()).stroke_width(2.5);
        sheet.ctx.draw_path(place * outline.path.clone());
    }

    // ── mask the crop spill (no clip API; bg is solid) ──
    {
        let ctx = &mut sheet.ctx;
        ctx.fill(bg()).no_stroke();
        ctx.rect(0.0, 0.0, W, PANEL_BOTTOM); // below
        ctx.rect(0.0, PANEL_TOP, W, H - PANEL_TOP); // above
        ctx.rect(0.0, 0.0, panel_left(0), H); // left of panel 01
        let gap_x0 = panel_left(0) + panel_w;
        ctx.rect(gap_x0, 0.0, panel_left(1) - gap_x0, H); // between panels
        let right_x0 = panel_left(1) + panel_w;
        ctx.rect(right_x0, 0.0, W - right_x0, H); // right of panel 02
    }

    // ── panel borders ──
    for i in 0..2 {
        let pl = panel_left(i);
        sheet.ctx.no_fill().stroke(border()).stroke_width(2.5);
        sheet.ctx.rect(pl, PANEL_BOTTOM, panel_w, PANEL_TOP - PANEL_BOTTOM);
    }

    // ── handles + point markers, colored by panel scheme ──
    let in_window = |x: f64, y: f64| {
        (UX0 - 4.0..=UX1 + 4.0).contains(&x) && (UY0 - 4.0..=UY1 + 4.0).contains(&y)
    };
    for i in 0..2 {
        let pl = panel_left(i);
        // handle lines
        for ((x1, y1), (x2, y2)) in &outline.handles {
            if !(in_window(*x1, *y1) && in_window(*x2, *y2)) {
                continue;
            }
            let correction = !on8(*x2) || !on8(*y2);
            let color = if i == 0 {
                handle()
            } else if correction {
                red()
            } else {
                handle()
            };
            sheet.ctx.no_fill().stroke(color).stroke_width(2.5);
            sheet.ctx.line(cx(pl, *x1), cy(*y1), cx(pl, *x2), cy(*y2));
        }
        // markers, knocked out with the background color
        for (x, y, role) in &outline.points {
            if !in_window(*x, *y) {
                continue;
            }
            let correction = !on8(*x) || !on8(*y);
            let color = if i == 0 {
                gray()
            } else if correction {
                red()
            } else {
                green()
            };
            sheet.ctx.fill(bg()).stroke(color).stroke_width(2.5);
            let (px, py) = (cx(pl, *x), cy(*y));
            match role {
                PtRole::Smooth => {
                    sheet.ctx.oval(px - 9.0, py - 9.0, 18.0, 18.0);
                }
                PtRole::Corner => {
                    sheet.ctx.rect(px - 8.0, py - 8.0, 16.0, 16.0);
                }
                PtRole::Off => {
                    sheet.ctx.oval(px - 7.0, py - 7.0, 14.0, 14.0);
                }
            }
        }
    }

    // ── callouts: leader from the text to the x=116 anchor ──
    for i in 0..2 {
        let pl = panel_left(i);
        let anchor = (cx(pl, 116.0), cy(160.0));
        let text_x = cx(pl, 250.0);
        let text_y = cy(160.0) - 10.0;
        let color = if i == 0 { gray() } else { red() };
        sheet.ctx.no_fill().stroke(color).stroke_width(2.5);
        sheet.ctx.line(anchor.0 + 16.0, anchor.1, text_x - 14.0, text_y + 8.0);
        let (l1, l2) = if i == 0 {
            ("X=116: OFF GRID", "CORRECTION OR MISTAKE?")
        } else {
            ("X=116: ON 2, OFF 8", "OPTICAL CORRECTION")
        };
        sheet.label_padded(l1, text_x, text_y + 20.0, 26.0, color, -1);
        sheet.label_padded(l2, text_x, text_y - 22.0, 26.0, color, -1);
    }

    // ── baseline tags ──
    for i in 0..2 {
        let pl = panel_left(i);
        sheet.label_padded("BASELINE 0", pl + 14.0, cy(0.0) + 12.0, 26.0, blue(), -1);
    }

    // ── panel titles + legends ──
    let title_y = PANEL_TOP + 44.0;
    sheet.label("01 FLAT GRID / ONE LEVEL", panel_left(0), title_y, 30.0, green(), -1);
    sheet.label(
        "ALL POINTS EQUALLY LEGAL",
        panel_left(0) + panel_w,
        title_y,
        26.0,
        gray(),
        1,
    );
    sheet.label(
        "02 NESTED GRID / 64 \u{b7} 8 \u{b7} 2",
        panel_left(1),
        title_y,
        30.0,
        green(),
        -1,
    );
    {
        // right-aligned legend: [green oval] ON 8   [red oval] ON 2, OFF 8
        let size = 26.0;
        let t2 = "ON 2, OFF 8";
        let t1 = "ON 8";
        let w2 = sheet.mono_width(t2, size);
        let w1 = sheet.mono_width(t1, size);
        let right = panel_left(1) + panel_w;
        let x2 = right - w2;
        let dot2 = x2 - 26.0;
        let x1 = dot2 - 36.0 - w1;
        let dot1 = x1 - 26.0;
        sheet.label(t2, x2, title_y, size, red(), -1);
        sheet.label(t1, x1, title_y, size, green(), -1);
        sheet.ctx.fill(bg()).stroke(red()).stroke_width(2.5);
        sheet.ctx.oval(dot2, title_y + 1.0, 16.0, 16.0);
        sheet.ctx.fill(bg()).stroke(green()).stroke_width(2.5);
        sheet.ctx.oval(dot1, title_y + 1.0, 16.0, 16.0);
    }

    // ── header / footer rules + captions ──
    {
        let ctx = &mut sheet.ctx;
        ctx.stroke(green()).stroke_width(2.5).no_fill();
        ctx.line(MARGIN, HEADER_RULE_Y, W - MARGIN, HEADER_RULE_Y);
        ctx.line(MARGIN, FOOTER_RULE_Y, W - MARGIN, FOOTER_RULE_Y);
    }
    sheet.label(
        "GRID AS LABELING FUNCTION",
        MARGIN,
        HEADER_RULE_Y + 24.0,
        30.0,
        green(),
        -1,
    );
    sheet.label(
        "VIRTUA GROTESK / EM 1024 = 2^10",
        W - MARGIN,
        HEADER_RULE_Y + 24.0,
        30.0,
        green(),
        1,
    );
    sheet.label(
        "LOWER BOWL OF a, REGULAR. SAME OUTLINE BOTH PANELS; ONLY THE NESTED GRID LABELS THE HAND",
        MARGIN,
        50.0,
        30.0,
        green(),
        -1,
    );
    sheet.label(
        "GITHUB.COM/ELIHEUER/VIRTUA-GROTESK",
        W - MARGIN,
        50.0,
        30.0,
        green(),
        1,
    );

    std::fs::create_dir_all(out.parent().unwrap()).unwrap();
    renderer
        .render_to_png(&sheet.ctx, out.to_str().unwrap())
        .unwrap();
    println!("wrote {}", out.display());
}
