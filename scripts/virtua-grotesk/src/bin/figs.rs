//! Comparison figures for the Virtua Grotesk post, drawn with designbot in the
//! same dark dimension-sheet language as `og`:
//!
//!   fig-complete-two : glyph completion (reference / 40% given / model)
//!   fig-bolden-b     : weight transfer  (Regular input / Bold predicted / Bold actual)
//!
//! The geometry comes from the font-garden-lab eval harness, which writes each
//! figure as a three-panel SVG in font units (y-down, baseline at y=900, panels
//! offset by the label x). We parse those paths, flip to y-up, and re-render
//! them here so the post's figures match the OG card instead of the harness's
//! plain white sheets. Re-run after a fresh eval to refresh:
//!
//!     cd scripts/virtua-grotesk && cargo run --release --bin figs
//!
//! Inputs (sibling checkout); repoint the glyphs in main() at a fresh run:
//!     ~/GH/repos/font-garden-lab/runs/v02/complete-R.svg
//!     ~/GH/repos/font-garden-lab/runs/night1/bolden-g.svg
//!     ~/GH/repos/google-fonts/ofl/geistmono/GeistMono[wght].ttf

use designbot::prelude::*;
use designbot_render::Renderer;
use kurbo::{Affine, BezPath, Shape};

const W: f64 = 2520.0;
const H: f64 = 1320.0;
const MARGIN: f64 = 64.0;
const GAP: f64 = 96.0;
/// Per-figure content geometry, so descender-deep glyphs (the g) get a
/// deeper grid and cap-height figures keep the cap line. Baselines are set
/// so the content block sits equidistant from the two rules.
struct Geom {
    baseline: f64,   // canvas y of font y=0
    top: f64,        // grid top, font units above the baseline
    bottom: f64,     // grid bottom, font units (negative = below baseline)
    cap: bool,       // draw the cap-height line + tag
    descender: bool, // draw the descender line + tag
}
const HEADER_RULE_Y: f64 = 1210.0;
const FOOTER_RULE_Y: f64 = 110.0;
const SVG_BASELINE: f64 = 900.0; // font-garden-lab SVGs put the baseline here

// Theme tokens, shared with og.rs.
fn bg() -> Color {
    Color::rgb(0x10, 0x10, 0x10)
}
fn grid() -> Color {
    Color::rgb(0x2a, 0x2a, 0x2a)
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

/// What a panel is: ground truth, the given input, or the model's output. The
/// role decides the glyph's color; it's read from the harness SVG's fill.
#[derive(Clone, Copy)]
enum Role {
    Reference, // #ccc  ground truth
    Given,     // #000  the seed handed to the model
    Model,     // #ff4e00  the model's output
}

impl Role {
    fn from_fill(fill: &str) -> Role {
        let f = fill.to_ascii_lowercase();
        if f.contains("ff4e00") || f.contains("ff4535") {
            Role::Model
        } else if f == "#000" || f == "#000000" || f == "black" {
            Role::Given
        } else {
            Role::Reference
        }
    }
    /// (fill, stroke), fills kept ~40% so the grid reads through, like og.rs.
    fn colors(self) -> (Color, Color) {
        match self {
            Role::Reference => (Color::rgba(0x8a, 0x8a, 0x8a, 90), gray()),
            Role::Given => (Color::rgba(0x15, 0xc4, 0x74, 110), green()),
            Role::Model => (Color::rgba(0xff, 0x45, 0x35, 104), red()),
        }
    }
}

struct Panel {
    label: String,
    path: BezPath, // font units, y-up, advance origin at x=0, baseline at y=0
    role: Role,
}

// --- SVG parsing ------------------------------------------------------------

fn attr(tag: &str, key: &str) -> Option<String> {
    let pat = format!("{key}=\"");
    let i = tag.find(&pat)? + pat.len();
    let j = tag[i..].find('"')? + i;
    Some(tag[i..j].to_string())
}

/// Parse the harness's three-panel SVG into panels in y-up font coordinates.
fn parse_svg(path: &std::path::Path) -> Vec<Panel> {
    let svg = std::fs::read_to_string(path).unwrap_or_else(|e| panic!("read {path:?}: {e}"));

    // labels + their x (the panel's advance origin), in document order
    let mut labels: Vec<(f64, String)> = Vec::new();
    for (start, _) in svg.match_indices("<text") {
        let open_len = svg[start..].find('>').expect("unterminated <text>");
        let open = &svg[start..start + open_len];
        let x: f64 = attr(open, "x").and_then(|s| s.parse().ok()).unwrap_or(0.0);
        let text_start = start + open_len + 1;
        let text_len = svg[text_start..].find("</text>").expect("no </text>");
        labels.push((x, svg[text_start..text_start + text_len].to_string()));
    }

    // path d + fill, in document order
    let mut paths: Vec<(String, String)> = Vec::new();
    for (start, _) in svg.match_indices("<path") {
        let end = svg[start..].find("/>").expect("unterminated <path>");
        let elem = &svg[start..start + end];
        let d = attr(elem, "d").expect("path without d");
        let fill = attr(elem, "fill").unwrap_or_default();
        paths.push((d, fill));
    }

    assert_eq!(labels.len(), paths.len(), "label/path count mismatch in {path:?}");

    labels
        .into_iter()
        .zip(paths)
        .map(|((offset, label), (d, fill))| {
            let raw = BezPath::from_svg(&d).unwrap_or_else(|e| panic!("bad path in {path:?}: {e}"));
            // SVG (y-down, origin at panel offset) -> font (y-up, origin at 0):
            //   x' = x - offset,  y' = SVG_BASELINE - y
            let to_font = Affine::new([1.0, 0.0, 0.0, -1.0, -offset, SVG_BASELINE]);
            Panel {
                label: label.trim().to_uppercase(),
                path: to_font * raw,
                role: Role::from_fill(&fill),
            }
        })
        .collect()
}

// --- sfnt family-name reader (same as og.rs) --------------------------------

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

// --- drawing ----------------------------------------------------------------

struct Sheet<'a> {
    ctx: Canvas,
    renderer: &'a Renderer,
    mono: String,
}

impl Sheet<'_> {
    fn label(&mut self, txt: &str, x: f64, y: f64, size: f64, color: Color, align: i8) {
        let w = self.renderer.text_width(txt, Some(&self.mono), size, &[]);
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

    /// Metric-line tag docked on a grid line, same look as og.rs.
    fn metric_tag(&mut self, txt: &str, x_edge: f64, y_line: f64, above: bool) {
        let size = 30.0;
        let w = self.renderer.text_width(txt, Some(&self.mono), size, &[]);
        let box_w = ((w + 16.0) / 16.0).ceil() * 16.0;
        let box_h = 32.0;
        let x0 = x_edge;
        let y0 = if above { y_line + 16.0 } else { y_line - box_h - 16.0 };
        self.ctx.fill(bg()).stroke(blue()).stroke_width(2.5);
        self.ctx.rect(x0, y0, box_w, box_h);
        let baseline = y0 + (box_h - 0.73 * size) / 2.0;
        self.label(txt, x0 + box_w / 2.0, baseline, size, blue(), 0);
    }
}

fn render_figure(
    renderer: &Renderer,
    mono: &str,
    panels: &[Panel],
    geom: &Geom,
    title: &str,
    right: &str,
    caption: &str,
    out: &std::path::Path,
) {
    let baseline_y = geom.baseline;
    let grid_top = baseline_y + geom.top;
    let grid_bottom = baseline_y + geom.bottom;
    let mut sheet = Sheet {
        ctx: Canvas::new(W, H),
        renderer,
        mono: mono.to_string(),
    };
    sheet.ctx.background(bg());

    let n = panels.len();
    let slot = (W - 2.0 * MARGIN - (n as f64 - 1.0) * GAP) / n as f64;
    let cell_left = |i: usize| MARGIN + i as f64 * (slot + GAP);

    // Align every panel to a shared glyph origin, centered on the reference
    // (panel 0) so the 40% seed and the model output land where they belong.
    let ref_bb = panels[0].path.bounding_box();
    let inset_x = (slot - ref_bb.width()) / 2.0 - ref_bb.x0;

    // ── 16-unit design grid across the sheet ──
    let step = 16.0;
    let grid_left = MARGIN - ((MARGIN % step + step) % step);
    {
        let ctx = &mut sheet.ctx;
        ctx.no_fill().stroke(grid()).stroke_width(2.0);
        let mut x = grid_left;
        while x <= W - MARGIN {
            ctx.line(x, grid_bottom, x, grid_top);
            x += step;
        }
        let mut y = grid_bottom;
        while y <= grid_top {
            ctx.line(MARGIN, y, W - MARGIN, y);
            y += step;
        }
    }

    // ── vertical metrics, full width behind the glyphs ──
    {
        let ctx = &mut sheet.ctx;
        ctx.stroke(blue()).stroke_width(2.5).no_fill();
        ctx.line_dash(&[10.0, 10.0]);
        let mut dashed = vec![-16.0];
        if geom.cap {
            dashed.push(784.0);
        }
        for y in dashed {
            ctx.line(MARGIN, baseline_y + y, W - MARGIN, baseline_y + y);
        }
        ctx.line_dash(&[]);
        let mut solid = vec![576.0, 0.0];
        if geom.cap {
            solid.push(768.0);
        }
        if geom.descender {
            solid.push(-256.0);
        }
        for y in solid {
            ctx.line(MARGIN, baseline_y + y, W - MARGIN, baseline_y + y);
        }
    }

    // ── per-cell dividers ──
    {
        let ctx = &mut sheet.ctx;
        ctx.stroke(blue()).stroke_width(2.5).no_fill();
        for i in 0..n {
            let x = cell_left(i) + inset_x; // the shared advance origin
            ctx.line(x, grid_bottom, x, grid_top);
        }
    }

    // ── glyphs, role-colored, grid reading through the ~40% fill ──
    for (i, p) in panels.iter().enumerate() {
        let (fill, stroke) = p.role.colors();
        let place = Affine::translate((cell_left(i) + inset_x, baseline_y));
        sheet.ctx.fill(fill).stroke(stroke).stroke_width(2.5);
        sheet.ctx.draw_path(place * p.path.clone());
    }

    // ── mask glyph spill at the grid box (no clip API; bg is solid) ──
    {
        let ctx = &mut sheet.ctx;
        ctx.fill(bg()).no_stroke();
        ctx.rect(0.0, 0.0, W, grid_bottom);
        ctx.rect(0.0, grid_top, W, H - grid_top);
        ctx.rect(0.0, 0.0, MARGIN, H);
        ctx.rect(W - MARGIN, 0.0, MARGIN, H);
    }

    // ── panel labels, numbered and role-colored, above the cap line ──
    for (i, p) in panels.iter().enumerate() {
        let (_, color) = p.role.colors();
        let txt = format!("{:02} {}", i + 1, p.label);
        sheet.label(&txt, cell_left(i), grid_top + 44.0, 30.0, color, -1);
    }

    // ── left-edge metric tags ──
    if geom.cap {
        sheet.metric_tag("CAP 768", grid_left, baseline_y + 768.0, false);
    }
    sheet.metric_tag("X-HEIGHT 576", grid_left, baseline_y + 576.0, true);
    sheet.metric_tag("BASELINE 0", grid_left, baseline_y, true);
    if geom.descender {
        sheet.metric_tag("DESCENDER -256", grid_left, baseline_y - 256.0, true);
    }

    // ── header / footer rules + captions ──
    {
        let ctx = &mut sheet.ctx;
        ctx.stroke(green()).stroke_width(2.5).no_fill();
        ctx.line(MARGIN, HEADER_RULE_Y, W - MARGIN, HEADER_RULE_Y);
        ctx.line(MARGIN, FOOTER_RULE_Y, W - MARGIN, FOOTER_RULE_Y);
    }
    sheet.label(title, MARGIN, HEADER_RULE_Y + 24.0, 30.0, green(), -1);
    sheet.label(right, W - MARGIN, HEADER_RULE_Y + 24.0, 30.0, green(), 1);
    sheet.label(caption, MARGIN, 64.0, 30.0, green(), -1);
    sheet.label(
        "GITHUB.COM/ELIHEUER/VIRTUA-GROTESK",
        W - MARGIN,
        64.0,
        30.0,
        green(),
        1,
    );

    std::fs::create_dir_all(out.parent().unwrap()).unwrap();
    renderer.render_to_png(&sheet.ctx, out.to_str().unwrap()).unwrap();
    println!("wrote {}", out.display());
}

fn main() {
    let home = std::env::var("HOME").unwrap();
    let lab = std::path::PathBuf::from(&home).join("GH/repos/font-garden-lab");
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

    // (eval SVG under font-garden-lab, output PNG, header, footer caption).
    // The model panels come from whichever eval run last covered the glyph;
    // repoint these at a fresh run to refresh.
    let cap_geom = Geom {
        baseline: 276.0,
        top: 784.0,
        bottom: -80.0,
        cap: true,
        descender: false,
    };
    let desc_geom = Geom {
        baseline: 459.0,
        top: 608.0,
        bottom: -272.0,
        cap: false,
        descender: true,
    };
    let figures = [
        (
            "runs/v02/complete-R.svg",
            "fig-complete-r.png",
            &cap_geom,
            "GLYPH COMPLETION",
            "MODEL: VIRTUA-12M-0.2",
            "MODEL FINISHES A HELD-OUT GLYPH FROM 40% OF ITS OUTLINE",
        ),
        (
            "runs/night1/bolden-g.svg",
            "fig-bolden-g.png",
            &desc_geom,
            "WEIGHT TRANSFER",
            "MODEL: VIRTUA-12M (NIGHT 1)",
            "MODEL PREDICTS THE BOLD WEIGHT FROM THE REGULAR",
        ),
    ];

    for (svg, png, geom, title, right, caption) in figures {
        let panels = parse_svg(&lab.join(svg));
        render_figure(&renderer, &mono, &panels, geom, title, right, caption, &post.join(png));
    }
}
