//! Model-demo figures for the Virtua Grotesk post, §07, in the green
//! dimension-sheet house style (same family as og.rs / figs.rs):
//!
//!   fig-model-review.png   : held-out review sheet, 3 rows x 7 glyphs —
//!                            human Regular (green), model Bold (red),
//!                            human Bold reference (gray; the a was never
//!                            boldened, so its cell says so)
//!   fig-model-bolden-a.png : the hero — Regular a (green, hand) to
//!                            Bold a (red, drawn by Virtua-12M-0.7)
//!
//! Margin discipline: 96px margins on all sides; header/footer rules at
//! 1224 / 112; all content centered between the rules.
//!
//! Run from this directory:
//!
//!     cargo run --release --bin model
//!
//! Inputs read at render time, from sibling checkouts:
//!     ~/GH/repos/virtua-grotesk/sources/VirtuaGrotesk-{Regular,Bold}.ufo
//!     ~/GH/repos/font-garden-lab/runs/v07/pred.ufo   (model outputs)
//!     ~/GH/repos/google-fonts/ofl/geistmono/GeistMono[wght].ttf

use designbot::prelude::*;
use designbot_render::Renderer;
use kurbo::{Affine, BezPath, Shape};

const W: f64 = 2520.0;
const H: f64 = 1320.0;
const MARGIN: f64 = 96.0;
const HEADER_RULE_Y: f64 = 1224.0;
const FOOTER_RULE_Y: f64 = 112.0;

const MODEL_NAME: &str = "VIRTUA-12M-0.7";

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
fn blue() -> Color {
    Color::rgb(0x4a, 0x78, 0xff)
}
fn gray() -> Color {
    Color::rgb(0x8a, 0x8a, 0x8a)
}
fn dim() -> Color {
    Color::rgb(0x5a, 0x5a, 0x5a)
}
fn handle() -> Color {
    Color::rgb(0x6a, 0x6a, 0x6a)
}

fn fill_of(stroke: Color) -> Color {
    Color::rgba(stroke.r, stroke.g, stroke.b, 100)
}

// --- UFO outline loading (same as og.rs) --------------------------------------

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
    width: f64,
}

fn push_on_curve(points: &mut Vec<(f64, f64, PtRole)>, p: &norad::ContourPoint, k: usize, n: usize) {
    if k == n {
        return;
    }
    let role = if p.smooth { PtRole::Smooth } else { PtRole::Corner };
    points.push((p.x, p.y, role));
}

fn load_outline(glif: &std::path::Path) -> Outline {
    let glyph = norad::Glyph::load(glif).unwrap_or_else(|e| panic!("load {glif:?}: {e}"));
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
    Outline {
        path,
        points,
        handles,
        width: glyph.width,
    }
}

/// UFO3 glif filename: uppercase names get an underscore suffix.
fn glif_name(glyph: &str) -> String {
    if glyph.chars().next().is_some_and(|c| c.is_uppercase()) {
        format!("{glyph}_.glif")
    } else {
        format!("{glyph}.glif")
    }
}

// --- sfnt family-name reader (same as og.rs) ----------------------------------

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

// --- drawing -------------------------------------------------------------------

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

    /// Metric-line tag docked on a grid line, og.rs style.
    fn metric_tag(&mut self, txt: &str, x_edge: f64, y_line: f64, above: bool) {
        let size = 30.0;
        let w = self.mono_width(txt, size);
        let box_w = ((w + 16.0) / 16.0).ceil() * 16.0;
        let box_h = 32.0;
        let y0 = if above { y_line + 16.0 } else { y_line - box_h - 16.0 };
        self.ctx.fill(bg()).stroke(blue()).stroke_width(2.5);
        self.ctx.rect(x_edge, y0, box_w, box_h);
        let baseline = y0 + (box_h - 0.73 * size) / 2.0;
        self.label(txt, x_edge + box_w / 2.0, baseline, size, blue(), 0);
    }

    fn frame(&mut self, title: &str, right: &str, caption: &str) {
        self.ctx.stroke(green()).stroke_width(2.5).no_fill();
        self.ctx.line(MARGIN, HEADER_RULE_Y, W - MARGIN, HEADER_RULE_Y);
        self.ctx.line(MARGIN, FOOTER_RULE_Y, W - MARGIN, FOOTER_RULE_Y);
        self.label(title, MARGIN, HEADER_RULE_Y + 42.0, 30.0, green(), -1);
        self.label(right, W - MARGIN, HEADER_RULE_Y + 42.0, 30.0, green(), 1);
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

fn draw_glyph(sheet: &mut Sheet, o: &Outline, x: f64, baseline: f64, s: f64, color: Color) {
    let place = Affine::new([s, 0.0, 0.0, s, x, baseline]);
    sheet.ctx.fill(fill_of(color)).stroke(color).stroke_width(2.5);
    sheet.ctx.draw_path(place * o.path.clone());
}

fn draw_points(sheet: &mut Sheet, o: &Outline, x: f64, baseline: f64, s: f64, color: Color) {
    sheet.ctx.no_fill().stroke(handle()).stroke_width(2.0);
    for ((x1, y1), (x2, y2)) in &o.handles {
        sheet
            .ctx
            .line(x + x1 * s, baseline + y1 * s, x + x2 * s, baseline + y2 * s);
    }
    for (px, py, role) in &o.points {
        let (cx, cy) = (x + px * s, baseline + py * s);
        sheet.ctx.fill(bg()).stroke(color).stroke_width(2.5);
        match role {
            PtRole::Smooth => {
                sheet.ctx.oval(cx - 8.0, cy - 8.0, 16.0, 16.0);
            }
            PtRole::Corner => {
                sheet.ctx.rect(cx - 7.0, cy - 7.0, 14.0, 14.0);
            }
            PtRole::Off => {
                sheet.ctx.oval(cx - 6.0, cy - 6.0, 12.0, 12.0);
            }
        }
    }
}

// --- fig-model-review -----------------------------------------------------------

fn fig_review(
    renderer: &Renderer,
    mono: &str,
    reg: &std::path::Path,
    bold: &std::path::Path,
    pred: &std::path::Path,
    out: &std::path::Path,
) {
    let mut sheet = Sheet {
        ctx: Canvas::new(W, H),
        renderer,
        mono: mono.to_string(),
    };
    sheet.ctx.background(bg());

    const GLYPHS: [&str; 7] = ["K", "E", "M", "n", "b", "c", "a"];
    const S: f64 = 0.27;

    // three bands between the rules, top to bottom
    let band_h = (HEADER_RULE_Y - FOOTER_RULE_Y) / 3.0;
    struct Row<'a> {
        label: String,
        color: Color,
        dir: &'a std::path::Path,
        skip_a: bool,
    }
    let rows = [
        Row {
            label: "01 INPUT / HUMAN REGULAR".into(),
            color: green(),
            dir: reg,
            skip_a: false,
        },
        Row {
            label: format!("02 OUTPUT / {MODEL_NAME}"),
            color: red(),
            dir: pred,
            skip_a: false,
        },
        Row {
            label: "03 REFERENCE / HUMAN BOLD".into(),
            color: gray(),
            dir: bold,
            skip_a: true,
        },
    ];

    let slot_w = (W - 2.0 * MARGIN) / GLYPHS.len() as f64;

    for (i, row) in rows.iter().enumerate() {
        let band_top = HEADER_RULE_Y - i as f64 * band_h;
        let band_bottom = band_top - band_h;
        let baseline = band_bottom + 88.0;

        // baseline, house blue, behind the glyphs
        sheet.ctx.no_fill().stroke(blue()).stroke_width(2.0);
        sheet.ctx.line(MARGIN, baseline, W - MARGIN, baseline);

        sheet.label(&row.label, MARGIN, band_top - 42.0, 26.0, row.color, -1);

        for (j, name) in GLYPHS.iter().enumerate() {
            let slot_center = MARGIN + (j as f64 + 0.5) * slot_w;
            if row.skip_a && *name == "a" {
                sheet.label_padded(
                    "NEVER BOLDENED",
                    slot_center,
                    baseline + 74.0,
                    24.0,
                    dim(),
                    0,
                );
                continue;
            }
            let o = load_outline(&row.dir.join("glyphs").join(glif_name(name)));
            let x = slot_center - o.width * S / 2.0;
            draw_glyph(&mut sheet, &o, x, baseline, S, row.color);
        }
    }

    sheet.frame(
        "WEIGHT TRANSFER / HELD-OUT REVIEW",
        &format!("MODEL: {MODEL_NAME}"),
        "THE MODEL NEVER SAW THESE BOLDS; THE BOLD a HAS NO HUMAN REFERENCE",
    );
    sheet.save(renderer, out);
}

// --- fig-model-bolden-a ----------------------------------------------------------

fn fig_bolden_a(
    renderer: &Renderer,
    mono: &str,
    reg: &std::path::Path,
    bold: &std::path::Path,
    out: &std::path::Path,
) {
    let mut sheet = Sheet {
        ctx: Canvas::new(W, H),
        renderer,
        mono: mono.to_string(),
    };
    sheet.ctx.background(bg());

    const S: f64 = 1.3;
    const BASELINE: f64 = 296.0;
    let grid_bottom = BASELINE - 64.0 * S; // 212.8
    let grid_top = BASELINE + 640.0 * S; // 1128.8

    // the a from each master; the Bold a in sources is the model's draft
    let o_reg = load_outline(&reg.join("glyphs/a.glif"));
    let o_bold = load_outline(&bold.join("glyphs/a.glif"));

    // run layout: centered between the margins
    let gap = 320.0;
    let run_w = o_reg.width * S + gap + o_bold.width * S;
    let x_reg = MARGIN + (W - 2.0 * MARGIN - run_w) / 2.0;
    let x_bold = x_reg + o_reg.width * S + gap;

    // 16-unit design grid, anchored to the Regular's origin
    {
        let step = 16.0 * S;
        let ctx = &mut sheet.ctx;
        ctx.no_fill().stroke(grid()).stroke_width(2.0);
        let mut x = x_reg - (((x_reg - MARGIN) / step).floor()) * step;
        while x <= W - MARGIN {
            ctx.line(x, grid_bottom, x, grid_top);
            x += step;
        }
        let mut y = BASELINE - (((BASELINE - grid_bottom) / step).floor()) * step;
        while y <= grid_top {
            ctx.line(MARGIN, y, W - MARGIN, y);
            y += step;
        }
    }

    // vertical metrics: solid baseline + x-height, dashed overshoots
    {
        let ctx = &mut sheet.ctx;
        ctx.no_fill().stroke(blue()).stroke_width(2.5);
        ctx.line_dash(&[10.0, 10.0]);
        for uy in [-16.0, 592.0] {
            ctx.line(MARGIN, BASELINE + uy * S, W - MARGIN, BASELINE + uy * S);
        }
        ctx.line_dash(&[]);
        for uy in [0.0, 576.0] {
            ctx.line(MARGIN, BASELINE + uy * S, W - MARGIN, BASELINE + uy * S);
        }
    }

    // glyphs with the Runebender point language on top
    draw_glyph(&mut sheet, &o_reg, x_reg, BASELINE, S, green());
    draw_glyph(&mut sheet, &o_bold, x_bold, BASELINE, S, red());
    draw_points(&mut sheet, &o_reg, x_reg, BASELINE, S, green());
    draw_points(&mut sheet, &o_bold, x_bold, BASELINE, S, red());

    // arrow between, at half x-height
    {
        let y = BASELINE + 288.0 * S;
        let x0 = x_reg + o_reg.width * S + 70.0;
        let x1 = x_bold - 70.0;
        let ctx = &mut sheet.ctx;
        ctx.no_fill().stroke(gray()).stroke_width(2.5);
        ctx.line(x0, y, x1, y);
        ctx.line(x1 - 22.0, y + 14.0, x1, y);
        ctx.line(x1 - 22.0, y - 14.0, x1, y);
    }

    // metric tags, docked at the left margin
    sheet.metric_tag("X-HEIGHT 576", MARGIN, BASELINE + 576.0 * S, true);
    sheet.metric_tag("BASELINE 0", MARGIN, BASELINE, true);

    // captions under the run, centered per glyph
    let label_y = 150.0;
    sheet.label(
        "REGULAR / DRAWN BY HAND",
        x_reg + o_reg.width * S / 2.0,
        label_y,
        26.0,
        green(),
        0,
    );
    sheet.label(
        &format!("BOLD / DRAWN BY {MODEL_NAME}"),
        x_bold + o_bold.width * S / 2.0,
        label_y,
        26.0,
        red(),
        0,
    );

    sheet.frame(
        "WEIGHT TRANSFER / THE FIRST BOLD a",
        &format!("MODEL: {MODEL_NAME}"),
        "THE BOLD MASTER NEVER HAD A REAL a; THE MODEL DREW THIS ONE",
    );
    sheet.save(renderer, out);
}

// --- main ------------------------------------------------------------------------

fn main() {
    let home = std::env::var("HOME").unwrap();
    let mono_path = format!("{home}/GH/repos/google-fonts/ofl/geistmono/GeistMono[wght].ttf");
    let sources = std::path::PathBuf::from(&home).join("GH/repos/virtua-grotesk/sources");
    let reg = sources.join("VirtuaGrotesk-Regular.ufo");
    let bold = sources.join("VirtuaGrotesk-Bold.ufo");
    let pred = std::path::PathBuf::from(&home).join("GH/repos/font-garden-lab/runs/v07/pred.ufo");

    let mut renderer = Renderer::new(W as u32, H as u32);
    let mono = load_family(&mut renderer, &mono_path);

    let here = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let post = here
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("src/content/blog/virtua-grotesk");

    fig_review(
        &renderer,
        &mono,
        &reg,
        &bold,
        &pred,
        &post.join("fig-model-review.png"),
    );
    fig_bolden_a(&renderer, &mono, &reg, &bold, &post.join("fig-model-bolden-a.png"));
}
