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
//! Inputs are pinned in inputs.rs:
//!     ~/GH/repos/font-garden-lab/runs/v08/complete-R.svg
//!     ~/GH/repos/font-garden-lab/runs/night1/bolden-g.svg
//!     ~/GH/repos/google-fonts/ofl/geistmono/GeistMono[wght].ttf

use designbot::prelude::*;
use designbot_render::Renderer;
use kurbo::{Affine, BezPath, Shape};
#[allow(unused_imports)]
use virtua_grotesk_figures::*;

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
const SVG_BASELINE: f64 = 900.0; // font-garden-lab SVGs put the baseline here

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
            Role::Reference => (with_alpha(gray(), 90), gray()),
            Role::Given => (with_alpha(green(), 110), green()),
            Role::Model => (with_alpha(red(), 104), red()),
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

    assert_eq!(
        labels.len(),
        paths.len(),
        "label/path count mismatch in {path:?}"
    );

    labels
        .into_iter()
        .zip(paths)
        .map(|((offset, label), (d, fill))| {
            let raw = BezPath::from_svg(&d).unwrap_or_else(|e| panic!("bad path in {path:?}: {e}"));
            // SVG (y-down, origin at panel offset) -> font (y-up, origin at 0):
            //   x' = x - offset,  y' = SVG_BASELINE - y
            let to_font = Affine::new([1.0, 0.0, 0.0, -1.0, -offset, SVG_BASELINE]);
            Panel {
                label: label.trim().to_string(),
                path: to_font * raw,
                role: Role::from_fill(&fill),
            }
        })
        .collect()
}

// --- drawing ----------------------------------------------------------------

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
    let mut sheet = new_sheet(renderer, mono);

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
        ctx.no_fill()
            .stroke(role::grid::standard())
            .stroke_width(line::THIN);
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
        ctx.stroke(blue()).stroke_width(PEN).no_fill();
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
        ctx.stroke(blue()).stroke_width(PEN).no_fill();
        for i in 0..n {
            let x = cell_left(i) + inset_x; // the shared advance origin
            ctx.line(x, grid_bottom, x, grid_top);
        }
    }

    // ── glyphs, role-colored, grid reading through the ~40% fill ──
    for (i, p) in panels.iter().enumerate() {
        let (fill, stroke) = p.role.colors();
        let place = Affine::translate((cell_left(i) + inset_x, baseline_y));
        sheet.ctx.fill(fill).stroke(stroke).stroke_width(PEN);
        sheet.ctx.draw_path(place * p.path.clone());
    }

    // ── mask glyph spill at the grid box (no clip API; bg is solid) ──
    {
        let ctx = &mut sheet.ctx;
        ctx.fill(role::canvas::background()).no_stroke();
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
        sheet.metric_tag("cap 768", grid_left, baseline_y + 768.0, false, -1);
    }
    sheet.metric_tag("x-height 576", grid_left, baseline_y + 576.0, true, -1);
    sheet.metric_tag("baseline 0", grid_left, baseline_y, true, -1);
    if geom.descender {
        sheet.metric_tag("descender -256", grid_left, baseline_y - 256.0, true, -1);
    }

    sheet.label_padded(
        title,
        MARGIN + 2.0,
        MARGIN + 4.0 + SMALL_TEXT + 14.0,
        FRAME_TEXT,
        green(),
        -1,
    );
    sheet.label_padded(caption, MARGIN + 2.0, MARGIN + 4.0, SMALL_TEXT, green(), -1);
    sheet.attribution(Some(right));

    write_png(renderer, &sheet.ctx, out);
}

fn main() {
    let lab = inputs::font_garden();
    let mono_path = inputs::geist_mono();

    let mut renderer = Renderer::new(W as u32, H as u32);
    let mono = load_family(&mut renderer, mono_path.to_str().unwrap());

    let outputs = OutputPaths::from_args();

    // (eval SVG under font-garden-lab, output PNG, header, footer caption).
    // The model panels come from whichever eval run last covered the glyph;
    // repoint these at a fresh run to refresh.
    let cap_geom = Geom {
        baseline: 276.0,
        top: 912.0,
        bottom: -208.0,
        cap: true,
        descender: false,
    };
    let desc_geom = Geom {
        baseline: 459.0,
        top: 728.0,
        bottom: -392.0,
        cap: false,
        descender: true,
    };
    let figures = [
        (
            inputs::COMPLETION_SVG,
            "fig-complete-r.png",
            &cap_geom,
            "Glyph completion",
            "model: Virtua-12M-v0.1",
            "the model finishes a held-out glyph from 40% of its outline",
        ),
        (
            inputs::BOLDEN_SVG,
            "fig-bolden-g.png",
            &desc_geom,
            "Weight transfer",
            "model: Virtua-12M (night 1)",
            "the model predicts the Bold weight from the Regular",
        ),
    ];

    for (svg, png, geom, title, right, caption) in figures {
        let panels = parse_svg(&lab.join(svg));
        render_figure(
            &renderer,
            &mono,
            &panels,
            geom,
            title,
            right,
            caption,
            &outputs.blog(png),
        );
    }
}
