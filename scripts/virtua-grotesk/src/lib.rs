//! Shared drawing mechanics for every Virtua Grotesk post figure.
//!
//! Visual decisions live in `style.rs`: base color swatches, line and type
//! scales, and semantic role mappings. The reviewed section 03 technical
//! drawing language lives in `technical.rs`. This file owns lower-level
//! mechanics: UFO loading, frames, labels, dimensions, point markers, and
//! collision-aware annotation placement. Binaries under `src/bin/` own
//! content and layout.
//!
//! POINT LANGUAGE (the key innovation, drawn not told):
//!   - circle = smooth on-curve, square = corner, small circle = off-curve
//!   - GREEN = on the 8-unit machine grid
//!   - RED   = off 8, on the 2-unit human grid: an optical correction

use designbot::prelude::*;
use designbot_render::Renderer;
use kurbo::{Affine, BezPath, Point as KPoint, Shape};
use std::path::{Path, PathBuf};

pub mod inputs;
pub mod style;
pub mod technical;
pub use style::*;
pub use technical::*;

/// Handle lengths worth calling out: the values the system actually
/// reuses (all with high 2-adic valuation or stem-adjacent).
pub const FAVORED: [f64; 7] = [64.0, 96.0, 128.0, 160.0, 192.0, 224.0, 256.0];

// --- color-managed PNG output ------------------------------------------------------

// This is intentionally local to the blog-figure crate. If these renderers
// seed social assets in the upstream Virtua Grotesk repo, copy this output
// policy with them; do not copy only the drawing code and lose the guardrail.

/// Add explicit sRGB, gAMA, and cHRM chunks to a DesignBot-rendered PNG.
///
/// DesignBot currently emits valid pixel data but no color-space chunk. An
/// untagged RGB image is device-dependent, so a browser or social-media image
/// pipeline may guess differently when it decodes and recompresses the file.
/// These figures are authored in 8-bit sRGB; tagging that fact makes the first
/// conversion deterministic. Relative-colorimetric intent is appropriate for
/// flat graphic colors because in-gamut colors should remain unchanged. Do not
/// use the PNG "saturation" intent here: it explicitly permits sacrificing hue
/// and lightness to preserve saturation.
///
/// The fallback gAMA/cHRM values are the exact sRGB values recommended by the
/// PNG specification for decoders that do not understand the sRGB chunk:
/// https://www.w3.org/TR/png-3/#sRGB-chunk
fn tag_png_as_srgb(path: &Path) {
    const SIGNATURE: &[u8; 8] = b"\x89PNG\r\n\x1a\n";
    const SRGB_GAMMA: u32 = 45_455;
    const SRGB_CHROMATICITIES: [u32; 8] = [
        31_270, 32_900, // white point
        64_000, 33_000, // red
        30_000, 60_000, // green
        15_000, 6_000, // blue
    ];

    fn chunk(bytes: &[u8], name: &[u8; 4]) -> Option<(usize, usize)> {
        let mut offset = 8;
        while offset + 12 <= bytes.len() {
            let length = u32::from_be_bytes(bytes[offset..offset + 4].try_into().unwrap()) as usize;
            let end = offset + 12 + length;
            assert!(end <= bytes.len(), "truncated PNG chunk in generated image");
            if &bytes[offset + 4..offset + 8] == name {
                return Some((offset, end));
            }
            offset = end;
        }
        None
    }

    fn push_chunk(out: &mut Vec<u8>, name: &[u8; 4], data: &[u8]) {
        out.extend_from_slice(&(data.len() as u32).to_be_bytes());
        out.extend_from_slice(name);
        out.extend_from_slice(data);
        let mut crc = crc32fast::Hasher::new();
        crc.update(name);
        crc.update(data);
        out.extend_from_slice(&crc.finalize().to_be_bytes());
    }

    let bytes = std::fs::read(path).unwrap();
    assert_eq!(&bytes[..8], SIGNATURE, "renderer did not produce a PNG");
    assert!(
        chunk(&bytes, b"cICP").is_none() && chunk(&bytes, b"iCCP").is_none(),
        "generated PNG already declares a non-sRGB color profile"
    );
    if chunk(&bytes, b"sRGB").is_some() {
        return;
    }

    let (_, ihdr_end) = chunk(&bytes, b"IHDR").expect("generated PNG has no IHDR chunk");
    let mut tagged = Vec::with_capacity(bytes.len() + 96);
    tagged.extend_from_slice(&bytes[..ihdr_end]);
    push_chunk(&mut tagged, b"sRGB", &[1]); // relative colorimetric
    push_chunk(&mut tagged, b"gAMA", &SRGB_GAMMA.to_be_bytes());
    let chromaticities: Vec<u8> = SRGB_CHROMATICITIES
        .into_iter()
        .flat_map(u32::to_be_bytes)
        .collect();
    push_chunk(&mut tagged, b"cHRM", &chromaticities);
    tagged.extend_from_slice(&bytes[ihdr_end..]);
    std::fs::write(path, tagged).unwrap();
}

/// Render a PNG and normalize its color metadata. All figure binaries should
/// use this function, directly or through `Sheet::save`.
pub fn write_png(renderer: &Renderer, context: &Canvas, out: &Path) {
    std::fs::create_dir_all(out.parent().unwrap()).unwrap();
    renderer
        .render_to_png(context, out.to_str().unwrap())
        .unwrap();
    tag_png_as_srgb(out);
    println!("wrote {}", out.display());
}

// --- output ownership ---------------------------------------------------------------

/// Output paths for a figure binary.
///
/// Renders update the working-tree blog asset by default so Astro can show
/// them in context immediately. Git does not store those revisions until they
/// are explicitly committed. Pass `--scratch` for an ignored standalone render.
pub struct OutputPaths {
    repo_root: PathBuf,
    preview_root: PathBuf,
    scratch: bool,
}

impl OutputPaths {
    pub fn from_args() -> Self {
        let mut scratch = false;
        for arg in std::env::args().skip(1) {
            match arg.as_str() {
                "--site" | "--publish" => scratch = false,
                "--scratch" | "--preview" => scratch = true,
                _ => panic!("unknown argument {arg:?}; use --site or --scratch"),
            }
        }

        let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let repo_root = manifest
            .parent()
            .and_then(Path::parent)
            .expect("figure crate must live at scripts/virtua-grotesk")
            .to_path_buf();
        let preview_root = manifest.join("build/preview");
        println!(
            "output mode: {}",
            if scratch {
                "scratch preview (ignored build directory)"
            } else {
                "site preview (working-tree asset; not committed automatically)"
            }
        );

        Self {
            repo_root,
            preview_root,
            scratch,
        }
    }

    pub fn blog(&self, filename: &str) -> PathBuf {
        if !self.scratch {
            self.repo_root
                .join("src/content/blog/virtua-grotesk")
                .join(filename)
        } else {
            self.preview_root.join("blog").join(filename)
        }
    }

    pub fn og(&self, filename: &str) -> PathBuf {
        if !self.scratch {
            self.repo_root.join("public/og").join(filename)
        } else {
            self.preview_root.join("og").join(filename)
        }
    }
}

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

fn push_on_curve(
    points: &mut Vec<(f64, f64, PtRole)>,
    p: &norad::ContourPoint,
    k: usize,
    n: usize,
) {
    if k == n {
        return;
    }
    let role = if p.smooth {
        PtRole::Smooth
    } else {
        PtRole::Corner
    };
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
        let role = if sp.smooth {
            PtRole::Smooth
        } else {
            PtRole::Corner
        };
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
    sheet.ctx.background(role::canvas::background());
    sheet
}

impl Sheet<'_> {
    pub fn mono_width(&self, txt: &str, size: f64) -> f64 {
        self.renderer.text_width(txt, Some(&self.mono), size, &[])
    }

    pub fn mono_width_weighted(&self, txt: &str, size: f64, weight: f32) -> f64 {
        self.renderer.text_width(
            txt,
            Some(&self.mono),
            size,
            &[(u32::from_be_bytes(*b"wght"), weight)],
        )
    }

    /// Mono label with its baseline at y. align: -1 left, 0 center, 1 right.
    pub fn label(&mut self, txt: &str, x: f64, y: f64, size: f64, color: Color, align: i8) {
        self.label_weighted(txt, x, y, size, color, align, 400.0);
    }

    pub fn label_weighted(
        &mut self,
        txt: &str,
        x: f64,
        y: f64,
        size: f64,
        color: Color,
        align: i8,
        weight: f32,
    ) {
        let w = self.mono_width_weighted(txt, size, weight);
        let x = match align {
            -1 => x,
            0 => x - w / 2.0,
            _ => x - w,
        };
        self.ctx
            .font(&self.mono)
            .clear_font_variations()
            .font_variation("wght", weight)
            .font_size(size)
            .fill(color)
            .text_align(TextAlign::Left)
            .text(txt, x, y);
    }

    /// Label over a background patch so it stays legible on the grid.
    pub fn label_padded(&mut self, txt: &str, x: f64, y: f64, size: f64, color: Color, align: i8) {
        self.label_padded_on(txt, x, y, size, color, align, role::canvas::background());
    }

    /// Padded label with an explicit knockout color for figure-specific palettes.
    pub fn label_padded_on(
        &mut self,
        txt: &str,
        x: f64,
        y: f64,
        size: f64,
        color: Color,
        align: i8,
        background: Color,
    ) {
        self.label_padded_weighted_on(txt, x, y, size, color, align, background, 400.0);
    }

    pub fn label_padded_weighted_on(
        &mut self,
        txt: &str,
        x: f64,
        y: f64,
        size: f64,
        color: Color,
        align: i8,
        background: Color,
        weight: f32,
    ) {
        let w = self.mono_width_weighted(txt, size, weight);
        let pad = 8.0;
        let x0 = match align {
            -1 => x,
            0 => x - w / 2.0,
            _ => x - w,
        };
        self.ctx.fill(background).no_stroke();
        self.ctx
            .rect(x0 - pad, y - 0.28 * size, w + 2.0 * pad, 1.3 * size);
        self.label_weighted(txt, x0, y, size, color, -1, weight);
    }

    /// Metric-line tag docked on a line. align: -1 left edge, 1 right edge.
    pub fn metric_tag(&mut self, txt: &str, x_edge: f64, y_line: f64, above: bool, align: i8) {
        let size = TAG_TEXT;
        let w = self.mono_width(txt, size);
        let box_w = ((w + 16.0) / 16.0).ceil() * 16.0;
        let box_h = 32.0;
        let x0 = if align < 0 { x_edge } else { x_edge - box_w };
        let y0 = if above {
            y_line + 16.0
        } else {
            y_line - box_h - 16.0
        };
        self.ctx
            .fill(role::canvas::background())
            .stroke(blue())
            .stroke_width(PEN);
        self.ctx.rect(x0, y0, box_w, box_h);
        let baseline = y0 + (box_h - 0.73 * size) / 2.0;
        self.label(txt, x0 + box_w / 2.0, baseline, size, blue(), 0);
    }

    /// Diagonal hatching clipped to a rect (the sidebearing texture).
    pub fn hatch(&mut self, x0: f64, y0: f64, x1: f64, y1: f64, color: Color) {
        self.hatch_with_width(x0, y0, x1, y1, color, line::MEDIUM);
    }

    pub fn hatch_with_width(
        &mut self,
        x0: f64,
        y0: f64,
        x1: f64,
        y1: f64,
        color: Color,
        width: f64,
    ) {
        let h = y1 - y0;
        self.ctx.stroke(color).stroke_width(width).no_fill();
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

    /// In-viewport title, top-left, knockout over whatever is beneath.
    pub fn hud_title(&mut self, lines: &[&str]) {
        // first line's cap tops sit exactly on the top margin
        let mut y = H - MARGIN - 0.74 * FRAME_TEXT;
        for line in lines {
            self.label_padded(line, MARGIN + 2.0, y, FRAME_TEXT, green(), -1);
            y -= FRAME_TEXT + 14.0;
        }
    }

    /// Attribution, bottom-right, knockout. `extra` adds a line above the
    /// standard one (model name, license, ...).
    pub fn attribution(&mut self, extra: Option<&str>) {
        let mut y = MARGIN + 4.0;
        self.label_padded(
            "Virtua Grotesk / elih.net/blog/virtua-grotesk",
            W - MARGIN - 2.0,
            y,
            SMALL_TEXT,
            green(),
            1,
        );
        if let Some(line) = extra {
            y += SMALL_TEXT + 14.0;
            self.label_padded(line, W - MARGIN - 2.0, y, SMALL_TEXT, green(), 1);
        }
    }

    pub fn save(&self, renderer: &Renderer, out: &std::path::Path) {
        write_png(renderer, &self.ctx, out);
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
    ctx.no_fill()
        .stroke(role::grid::standard())
        .stroke_width(PEN_LIGHT);
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
    advance_row_colored(
        sheet,
        f,
        y,
        glyphs,
        inks,
        gray(),
        red(),
        green(),
        red(),
        role::canvas::background(),
        PEN,
        line::MEDIUM,
        DIM_TEXT,
        400.0,
    );
}

/// Advance-width dimension zone with explicit colors for figure palettes.
pub fn advance_row_colored(
    sheet: &mut Sheet,
    f: &Frame,
    y: f64,
    glyphs: &[(f64, f64)],
    inks: &[(f64, f64)],
    line_color: Color,
    hatch_color: Color,
    value_color: Color,
    bearing_color: Color,
    background: Color,
    stroke_width: f64,
    hatch_width: f64,
    text_size: f64,
    text_weight: f32,
) {
    for ((origin, adv), (ink0, ink1)) in glyphs.iter().zip(inks) {
        let (bx0, bx1) = (f.x(*origin), f.x(origin + adv));
        let (ix0, ix1) = (f.x(*ink0 + origin), f.x(*ink1 + origin));
        sheet
            .ctx
            .no_fill()
            .stroke(line_color)
            .stroke_width(stroke_width);
        sheet.ctx.line(bx0, y, bx1, y);
        sheet.ctx.line(bx0, y - 20.0, bx0, y + 20.0);
        sheet.ctx.line(bx1, y - 20.0, bx1, y + 20.0);
        sheet.hatch_with_width(bx0, y - 14.0, ix0, y + 14.0, hatch_color, hatch_width);
        sheet.hatch_with_width(ix1, y - 14.0, bx1, y + 14.0, hatch_color, hatch_width);
        sheet.label_padded_weighted_on(
            &format!("{}", (ink1 - ink0).round()),
            (ix0 + ix1) / 2.0,
            y - 10.0,
            text_size,
            value_color,
            0,
            background,
            text_weight,
        );
        sheet.label_padded_weighted_on(
            &format!("{}", ink0.round()),
            (bx0 + ix0) / 2.0,
            y + 26.0,
            text_size,
            bearing_color,
            0,
            background,
            text_weight,
        );
        sheet.label_padded_weighted_on(
            &format!("{}", (adv - ink1).round()),
            (ix1 + bx1) / 2.0,
            y + 26.0,
            text_size,
            bearing_color,
            0,
            background,
            text_weight,
        );
    }
}

// --- the annotation engine -----------------------------------------------------------------

pub fn on8(x: f64, y: f64) -> bool {
    x.rem_euclid(8.0) == 0.0 && y.rem_euclid(8.0) == 0.0
}

#[derive(Clone, Copy)]
pub struct PointStyle {
    pub smooth_size: f64,
    pub corner_size: f64,
    pub off_curve_size: f64,
    pub correction_filled: bool,
    pub stroke_width: f64,
}

pub const DEFAULT_POINT_STYLE: PointStyle = PointStyle {
    smooth_size: 16.0,
    corner_size: 14.0,
    off_curve_size: 11.0,
    correction_filled: true,
    stroke_width: PEN_LIGHT,
};

/// Large, uniform point language for the simplified inline figures. Point
/// roles still use circle/square geometry, but every marker has equal visual
/// weight and remains legible when the figure is shown as a small card.
pub const FIGURE_POINT_STYLE: PointStyle = PointStyle {
    smooth_size: 20.0,
    corner_size: 20.0,
    off_curve_size: 20.0,
    correction_filled: false,
    stroke_width: line::HERO,
};

/// Draw an opaque, OG-style glyph with one near-black pen color and the two
/// light-gray point tiers. The fill color is the only figure-specific choice.
pub fn draw_figure_glyph(
    sheet: &mut Sheet,
    o: &Outline,
    s: f64,
    x0: f64,
    baseline: f64,
    fill: Color,
) {
    draw_body_styled(
        sheet,
        o,
        s,
        x0,
        baseline,
        fill,
        255,
        role::figure::pen(),
        line::HERO,
    );
    draw_points_styled(
        sheet,
        o,
        s,
        x0,
        baseline,
        role::figure::pen(),
        role::figure::pen(),
        role::figure::pen(),
        role::figure::point_fill(),
        role::figure::correction_point_fill(),
        FIGURE_POINT_STYLE,
    );
}

/// Glyph body with the technical (translucent) fill: light gray, so the
/// semantic point colors (green machine / red hand) stay legible on top.
pub fn draw_body(sheet: &mut Sheet, o: &Outline, s: f64, x0: f64, baseline: f64) {
    let place = Affine::new([s, 0.0, 0.0, s, x0, baseline]);
    sheet
        .ctx
        .fill(with_alpha(gray_200(), 36))
        .stroke(gray_200())
        .stroke_width(PEN);
    sheet.ctx.draw_path(place * o.path.clone());
}

/// Glyph body in an arbitrary color with the rich hero fill.
pub fn draw_body_strong(
    sheet: &mut Sheet,
    o: &Outline,
    s: f64,
    x0: f64,
    baseline: f64,
    color: Color,
) {
    draw_body_strong_with_width(sheet, o, s, x0, baseline, color, PEN);
}

pub fn draw_body_strong_with_width(
    sheet: &mut Sheet,
    o: &Outline,
    s: f64,
    x0: f64,
    baseline: f64,
    color: Color,
    stroke_width: f64,
) {
    draw_body_with_alpha_with_width(sheet, o, s, x0, baseline, color, 104, stroke_width);
}

pub fn draw_body_with_alpha_with_width(
    sheet: &mut Sheet,
    o: &Outline,
    s: f64,
    x0: f64,
    baseline: f64,
    color: Color,
    fill_alpha: u8,
    stroke_width: f64,
) {
    draw_body_styled(
        sheet,
        o,
        s,
        x0,
        baseline,
        color,
        fill_alpha,
        color,
        stroke_width,
    );
}

/// Glyph body with independently editable fill and outline colors.
pub fn draw_body_styled(
    sheet: &mut Sheet,
    o: &Outline,
    s: f64,
    x0: f64,
    baseline: f64,
    fill_color: Color,
    fill_alpha: u8,
    stroke_color: Color,
    stroke_width: f64,
) {
    let place = Affine::new([s, 0.0, 0.0, s, x0, baseline]);
    sheet
        .ctx
        .fill(with_alpha(fill_color, fill_alpha))
        .stroke(stroke_color)
        .stroke_width(stroke_width);
    sheet.ctx.draw_path(place * o.path.clone());
}

/// Handles + point markers colored by GRID LEVEL: green on the 8-unit
/// machine grid, red off 8 (the human's 2-unit optical corrections).
pub fn draw_points(sheet: &mut Sheet, o: &Outline, s: f64, x0: f64, baseline: f64) {
    draw_points_handle_stroke(sheet, o, s, x0, baseline, role::bezier::handles());
}

/// draw_points with a chosen handle-line color (e.g. purple to match
/// handle-length labels).
pub fn draw_points_handle_stroke(
    sheet: &mut Sheet,
    o: &Outline,
    s: f64,
    x0: f64,
    baseline: f64,
    stroke: Color,
) {
    draw_points_colored(sheet, o, s, x0, baseline, stroke, green(), red());
}

/// Full control over the point language colors: handle stroke, on-8
/// marker color, off-8 marker color.
pub fn draw_points_colored(
    sheet: &mut Sheet,
    o: &Outline,
    s: f64,
    x0: f64,
    baseline: f64,
    stroke: Color,
    on8_col: Color,
    off8_col: Color,
) {
    draw_points_colored_on(
        sheet,
        o,
        s,
        x0,
        baseline,
        stroke,
        on8_col,
        off8_col,
        role::canvas::background(),
    );
}

/// Point language with an explicit knockout color for figure palettes.
pub fn draw_points_colored_on(
    sheet: &mut Sheet,
    o: &Outline,
    s: f64,
    x0: f64,
    baseline: f64,
    stroke: Color,
    on8_col: Color,
    off8_col: Color,
    background: Color,
) {
    draw_points_styled(
        sheet,
        o,
        s,
        x0,
        baseline,
        stroke,
        on8_col,
        off8_col,
        background,
        background,
        DEFAULT_POINT_STYLE,
    );
}

pub fn draw_points_styled(
    sheet: &mut Sheet,
    o: &Outline,
    s: f64,
    x0: f64,
    baseline: f64,
    stroke: Color,
    on8_col: Color,
    off8_col: Color,
    on8_fill: Color,
    off8_fill: Color,
    style: PointStyle,
) {
    sheet
        .ctx
        .no_fill()
        .stroke(stroke)
        .stroke_width(style.stroke_width);
    for ((ax, ay), (hx, hy)) in &o.handles {
        sheet.ctx.line(
            x0 + ax * s,
            baseline + ay * s,
            x0 + hx * s,
            baseline + hy * s,
        );
    }
    for (px, py, role) in &o.points {
        let (color, fill) = if on8(*px, *py) {
            (on8_col, on8_fill)
        } else if style.correction_filled {
            (off8_col, off8_col)
        } else {
            (off8_col, off8_fill)
        };
        let size = match role {
            PtRole::Smooth => style.smooth_size,
            PtRole::Corner => style.corner_size,
            PtRole::Off => style.off_curve_size,
        };
        marker_with_fill_sized(
            sheet,
            x0 + px * s,
            baseline + py * s,
            *role,
            color,
            fill,
            size,
            style.stroke_width,
        );
    }
}

/// Handles + point markers in one color (for role-colored sheets like the
/// model review, where color means input/output/reference instead).
pub fn draw_points_mono(
    sheet: &mut Sheet,
    o: &Outline,
    s: f64,
    x0: f64,
    baseline: f64,
    color: Color,
) {
    sheet
        .ctx
        .no_fill()
        .stroke(role::bezier::handles())
        .stroke_width(PEN_LIGHT);
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
    marker_with_fill(sheet, cx, cy, role, color, role::canvas::background());
}

pub fn marker_with_fill(
    sheet: &mut Sheet,
    cx: f64,
    cy: f64,
    role: PtRole,
    color: Color,
    fill: Color,
) {
    let size = match role {
        PtRole::Smooth => 16.0,
        PtRole::Corner => 14.0,
        PtRole::Off => 11.0,
    };
    marker_with_fill_sized(sheet, cx, cy, role, color, fill, size, PEN);
}

pub fn marker_with_fill_sized(
    sheet: &mut Sheet,
    cx: f64,
    cy: f64,
    role: PtRole,
    color: Color,
    fill: Color,
    size: f64,
    stroke_width: f64,
) {
    let half = size / 2.0;
    sheet
        .ctx
        .fill(fill)
        .stroke(color)
        .stroke_width(stroke_width);
    match role {
        PtRole::Smooth => {
            sheet.ctx.oval(cx - half, cy - half, size, size);
        }
        PtRole::Corner => {
            sheet.ctx.rect(cx - half, cy - half, size, size);
        }
        PtRole::Off => {
            sheet.ctx.oval(cx - half, cy - half, size, size);
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
            sheet.label_padded(
                &format!("{len}"),
                mx + 14.0,
                my - 7.0,
                SMALL_TEXT,
                purple(),
                -1,
            );
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
        let ty = (baseline + p.1 * s + dxy.1).clamp(MARGIN + 8.0, H - MARGIN - SMALL_TEXT);
        sheet.label_padded(&txt, tx, ty, SMALL_TEXT, gray(), -1);
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
    let t2 = "off 8, on 2 = correction";
    let t1 = "on 8 = structure";
    let w2 = sheet.mono_width(t2, size);
    let w1 = sheet.mono_width(t1, size);
    let x2 = x_right - w2;
    let dot2 = x2 - 26.0;
    let x1 = dot2 - 40.0 - w1;
    let dot1 = x1 - 26.0;
    sheet.label(t2, x2, y, size, red(), -1);
    sheet.label(t1, x1, y, size, green(), -1);
    sheet
        .ctx
        .fill(role::canvas::background())
        .stroke(red())
        .stroke_width(PEN);
    sheet.ctx.oval(dot2, y + 1.0, 15.0, 15.0);
    sheet
        .ctx
        .fill(role::canvas::background())
        .stroke(green())
        .stroke_width(PEN);
    sheet.ctx.oval(dot1, y + 1.0, 15.0, 15.0);
}

/// A blue knockout node circle at a line intersection (the little circles
/// where cell dividers meet rules and dimension rows).
pub fn node(sheet: &mut Sheet, x: f64, y: f64, r: f64) {
    sheet
        .ctx
        .fill(role::canvas::background())
        .stroke(blue())
        .stroke_width(PEN);
    sheet.ctx.oval(x - r, y - r, r * 2.0, r * 2.0);
}

/// Blue advance-boundary dividers: one vertical per boundary, spanning
/// y_top..y_bottom (canvas), with knockout nodes at both ends.
pub fn cell_dividers(sheet: &mut Sheet, xs: &[f64], y_top: f64, y_bottom: f64) {
    cell_dividers_colored(
        sheet,
        xs,
        y_top,
        y_bottom,
        blue(),
        role::canvas::background(),
        PEN,
    );
}

/// Advance-boundary dividers with explicit colors for figure palettes.
pub fn cell_dividers_colored(
    sheet: &mut Sheet,
    xs: &[f64],
    y_top: f64,
    y_bottom: f64,
    color: Color,
    background: Color,
    stroke_width: f64,
) {
    for &x in xs {
        sheet.ctx.no_fill().stroke(color).stroke_width(stroke_width);
        sheet.ctx.line(x, y_bottom, x, y_top);
        sheet
            .ctx
            .fill(background)
            .stroke(color)
            .stroke_width(stroke_width);
        sheet.ctx.oval(x - 7.0, y_top - 7.0, 14.0, 14.0);
        sheet.ctx.oval(x - 7.0, y_bottom - 7.0, 14.0, 14.0);
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

// --- label placement engine --------------------------------------------------
// Collision-aware label placement, shared by the dimension-sheet figures
// (reference usage: src/bin/interpn.rs). Register fixed obstacles, ink
// paths (canvas coordinates), and anchor points; then place labels.

/// 96 -> "64+32": a value as its descending powers-of-two sum.
pub fn p2sum(v: i64) -> String {
    let mut parts = Vec::new();
    let mut bit = 1i64 << 62;
    while bit > 0 {
        if v & bit != 0 {
            parts.push(bit.to_string());
        }
        bit >>= 1;
    }
    parts.join("+")
}

/// "96 (64+32)"; pure powers stay bare ("128", not "128 (128)").
pub fn fmt_val(v: i64) -> String {
    if v.count_ones() <= 1 {
        v.to_string()
    } else {
        format!("{v} ({})", p2sum(v))
    }
}

/// Unit vector pointing away from the ink around a boundary point.
/// Probes 16 directions at radius r; averages the ink-free ones.
pub fn outward_dir(path: &BezPath, x: f64, y: f64, r: f64) -> (f64, f64) {
    let (mut sx, mut sy) = (0.0f64, 0.0f64);
    for k in 0..16 {
        let th = k as f64 * std::f64::consts::TAU / 16.0;
        let (dx, dy) = (th.cos(), th.sin());
        if !path.contains(KPoint::new(x + dx * r, y + dy * r)) {
            sx += dx;
            sy += dy;
        }
    }
    let n = (sx * sx + sy * sy).sqrt();
    if n < 1e-6 {
        (0.0, -1.0)
    } else {
        (sx / n, sy / n)
    }
}

pub struct Labeler {
    placed: Vec<(f64, f64, f64, f64)>,
    ink: Vec<BezPath>,
    anchors: Vec<(f64, f64)>,
    markers: Vec<(f64, f64)>,
    queued: Vec<(f64, f64, (f64, f64), String, Color, bool, usize)>,
}

impl Default for Labeler {
    fn default() -> Self {
        Self::new()
    }
}

impl Labeler {
    pub fn new() -> Self {
        Labeler {
            placed: Vec::new(),
            ink: Vec::new(),
            anchors: Vec::new(),
            markers: Vec::new(),
            queued: Vec::new(),
        }
    }

    fn text_w(txt: &str) -> f64 {
        txt.len() as f64 * SMALL_TEXT * 0.62 + 18.0
    }

    /// Register a fixed rectangle labels must avoid.
    pub fn obstacle(&mut self, x0: f64, y0: f64, x1: f64, y1: f64) {
        self.placed.push((x0, y0, x1, y1));
    }

    /// Register already-drawn centered text as an obstacle.
    pub fn obstacle_text(&mut self, cx: f64, y: f64, size: f64, txt: &str) {
        let w = txt.len() as f64 * size * 0.62 + 18.0;
        self.placed
            .push((cx - w / 2.0, y - 10.0, cx + w / 2.0, y + size + 12.0));
    }

    /// Register a glyph outline (canvas coordinates) labels must not cover.
    pub fn ink(&mut self, path: BezPath) {
        self.ink.push(path);
    }

    /// Register an anchor point (used for the rival-ambiguity cost and
    /// as a marker no label may cover).
    pub fn anchor(&mut self, x: f64, y: f64) {
        self.anchors.push((x, y));
        self.markers.push((x, y));
    }

    /// Register a drawn marker (e.g. an off-curve dot) no label may cover.
    pub fn marker(&mut self, x: f64, y: f64) {
        self.markers.push((x, y));
    }

    fn overlaps(&self, r: (f64, f64, f64, f64)) -> bool {
        self.placed
            .iter()
            .any(|q| r.0 < q.2 && q.0 < r.2 && r.1 < q.3 && q.1 < r.3)
    }

    fn ink_hit(&self, r: (f64, f64, f64, f64)) -> bool {
        let probes = [
            KPoint::new(r.0, r.1),
            KPoint::new(r.2, r.1),
            KPoint::new(r.0, r.3),
            KPoint::new(r.2, r.3),
            KPoint::new((r.0 + r.2) / 2.0, (r.1 + r.3) / 2.0),
        ];
        self.ink
            .iter()
            .any(|p| probes.iter().any(|pt| p.contains(*pt)))
    }

    /// Queue a label for simultaneous placement (see `place_all`).
    /// `dir` is the preferred outward unit vector, `avoid_ink` forbids
    /// positions over the registered outlines (use for coordinate labels;
    /// handle/segment labels may sit on the glyph fill).
    /// `owner` is the index of the ink path this label belongs to (order
    /// of `ink()` calls); labels pay a heavy cost for candidates inside a
    /// DIFFERENT glyph's bounding box, so they stay in their own territory.
    pub fn queue(
        &mut self,
        px: f64,
        py: f64,
        dir: (f64, f64),
        txt: String,
        color: Color,
        avoid_ink: bool,
        owner: usize,
    ) {
        self.queued
            .push((px, py, dir, txt, color, avoid_ink, owner));
    }

    /// Place every queued label at once by simulated annealing over
    /// discrete candidate positions (8 octants x 5 gaps per label), after
    /// Christensen, Marks & Shieber, "An Empirical Study of Algorithms for
    /// Point-Feature Label Placement" (ACM TOG 1995). The objective sums a
    /// per-candidate preference cost (stay close, stay outward, stay
    /// unambiguous) and a large penalty per overlapping label pair.
    pub fn place_all(&mut self, sheet: &mut Sheet) {
        let reqs = std::mem::take(&mut self.queued);
        if reqs.is_empty() {
            return;
        }
        struct Cand {
            x: f64,
            y: f64,
            rect: (f64, f64, f64, f64),
            cost: f64,
        }
        let bboxes: Vec<kurbo::Rect> = self.ink.iter().map(|p| p.bounding_box()).collect();
        let mut cands: Vec<Vec<Cand>> = Vec::with_capacity(reqs.len());
        for (px, py, dir, txt, _, avoid_ink, owner) in reqs.iter() {
            let w = Self::text_w(txt);
            // snap the outward direction to the nearest octant so the
            // candidate fan is always the 8 clean cartographic positions
            let base = (dir.1.atan2(dir.0) / std::f64::consts::FRAC_PI_4).round()
                * std::f64::consts::FRAC_PI_4;
            let mut list = Vec::new();
            for k in 0..8i32 {
                // angular octant distance from the outward direction
                let ang_steps = k.min(8 - k) as f64;
                let a = base + k as f64 * std::f64::consts::FRAC_PI_4;
                let u = (a.cos(), a.sin());
                for (gi, gap) in [12.0f64, 26.0].iter().enumerate() {
                    // near or nothing, at a UNIFORM visual distance: offset
                    // each axis by half-extent + gap, so on diagonals the box
                    // corner hugs the point exactly like the axis placements
                    let (hw, hh) = (w / 2.0, 18.0);
                    let m = u.0.abs().max(u.1.abs());
                    let ex = (u.0 / m).round();
                    let ey = (u.1 / m).round();
                    let cx = px + ex * (hw + gap);
                    let cy = py + ey * (hh + gap) - 8.0;
                    let r = (cx - w / 2.0, cy - 10.0, cx + w / 2.0, cy + 26.0);
                    let dbg = std::env::var("DEBUG_LABELS").is_ok() && txt == "64,16" && gi < 2;
                    if r.0 < 6.0 || r.2 > W - 6.0 || r.1 < 6.0 || r.3 > H - 6.0 {
                        if dbg {
                            eprintln!("  rej canvas k={k} gi={gi} c=({cx:.0},{cy:.0})");
                        }
                        continue;
                    }
                    if self.overlaps(r) {
                        if dbg {
                            eprintln!("  rej fixed  k={k} gi={gi} c=({cx:.0},{cy:.0})");
                        }
                        continue; // fixed obstacles are hard constraints
                    }
                    if *avoid_ink && self.ink_hit(r) {
                        if dbg {
                            eprintln!("  rej ink    k={k} gi={gi} c=({cx:.0},{cy:.0})");
                        }
                        continue;
                    }
                    // hard rule: a label box never covers any drawn marker
                    // (its own point is exempt; the clearance geometry
                    // already keeps the box off it)
                    if self.markers.iter().any(|(mx, my)| {
                        ((mx - px).abs() > 0.1 || (my - py).abs() > 0.1)
                            && *mx > r.0 - 10.0
                            && *mx < r.2 + 10.0
                            && *my > r.1 - 10.0
                            && *my < r.3 + 10.0
                    }) {
                        if std::env::var("DEBUG_LABELS").is_ok() && txt == "64,16" && gi < 2 {
                            eprintln!("  rej marker k={k} gi={gi} c=({cx:.0},{cy:.0})");
                        }
                        continue;
                    }
                    let d_own = ((cx - px).powi(2) + (cy + 8.0 - py).powi(2)).sqrt();
                    let mut cost = ang_steps * 18.0 + gi as f64 * 20.0 + d_own * 0.02;
                    // territory: penalize only candidates sitting in ANOTHER
                    // glyph's column while outside their own; the open canvas
                    // margins outside every column are free ground
                    let in_own_col = bboxes
                        .get(*owner)
                        .is_none_or(|ob| cx > ob.x0 - 44.0 && cx < ob.x1 + 44.0);
                    for (bi, bb) in bboxes.iter().enumerate() {
                        if bi == *owner {
                            continue;
                        }
                        let in_other_col = cx > bb.x0 - 44.0 && cx < bb.x1 + 44.0;
                        if in_other_col && !in_own_col {
                            cost += 400.0;
                        }
                        if cx > bb.x0 && cx < bb.x1 && cy + 8.0 > bb.y0 && cy + 8.0 < bb.y1 {
                            cost += 500.0;
                        }
                    }
                    for (qx, qy) in self.anchors.iter() {
                        if (qx - px).abs() < 0.1 && (qy - py).abs() < 0.1 {
                            continue;
                        }
                        let d_riv = ((cx - qx).powi(2) + (cy + 8.0 - qy).powi(2)).sqrt();
                        if d_riv < d_own {
                            cost += 140.0;
                        }
                    }
                    list.push(Cand {
                        x: cx,
                        y: cy,
                        rect: r,
                        cost,
                    });
                }
            }
            if std::env::var("DEBUG_LABELS").is_ok() && txt == "64,16" {
                eprintln!("cands for 64,16 at ({px:.0},{py:.0}): {}", list.len());
                for c in list.iter().take(6) {
                    eprintln!("  ({:>5.0},{:>5.0}) cost {:>6.1}", c.x, c.y, c.cost);
                }
            }
            {}
            cands.push(list);
        }

        // initial assignment: cheapest candidate each
        let n = reqs.len();
        let mut pick: Vec<Option<usize>> = cands
            .iter()
            .map(|l| (0..l.len()).min_by(|a, b| l[*a].cost.total_cmp(&l[*b].cost)))
            .collect();
        let isect = |a: (f64, f64, f64, f64), b: (f64, f64, f64, f64)| {
            a.0 < b.2 && b.0 < a.2 && a.1 < b.3 && b.1 < a.3
        };
        const W_OVL: f64 = 3000.0;
        let pair_energy = |i: usize, ci: usize, pick: &Vec<Option<usize>>| {
            let mut e = 0.0;
            let ri = cands[i][ci].rect;
            for (j, pj) in pick.iter().enumerate() {
                if j == i {
                    continue;
                }
                if let Some(cj) = pj {
                    if isect(ri, cands[j][*cj].rect) {
                        e += W_OVL;
                    }
                }
            }
            e
        };

        // simulated annealing, deterministic LCG
        let mut rng: u64 = 0x9E3779B97F4A7C15;
        let mut next = |m: usize| -> usize {
            rng = rng
                .wrapping_mul(6364136223846793005)
                .wrapping_add(1442695040888963407);
            ((rng >> 33) as usize) % m.max(1)
        };
        let mut temp = 2600.0f64; // hot enough to hop over one overlap
        for _sweep in 0..420 {
            for _ in 0..n {
                let i = next(n);
                if cands[i].is_empty() {
                    continue;
                }
                let cur = match pick[i] {
                    Some(c) => c,
                    None => continue,
                };
                let alt = next(cands[i].len());
                if alt == cur {
                    continue;
                }
                let e_cur = cands[i][cur].cost + pair_energy(i, cur, &pick);
                let e_alt = cands[i][alt].cost + pair_energy(i, alt, &pick);
                let de = e_alt - e_cur;
                let accept = de < 0.0 || {
                    let p = (-de / temp).exp();
                    (next(1_000_000) as f64) / 1_000_000.0 < p
                };
                if accept {
                    pick[i] = Some(alt);
                }
            }
            temp *= 0.97;
        }

        // repair pass: keep labels greedily; a label overlapping the kept
        // set retries every candidate before it is dropped
        let mut kept: Vec<(usize, usize)> = Vec::new();
        let mut dropped = 0usize;
        let mut order: Vec<usize> = (0..n).collect();
        order.sort_by(|a, b| {
            let ca = pick[*a].map(|c| cands[*a][c].cost).unwrap_or(f64::MAX);
            let cb = pick[*b].map(|c| cands[*b][c].cost).unwrap_or(f64::MAX);
            ca.total_cmp(&cb)
        });
        for i in order {
            let free_vs_kept = |ci: usize, kept: &Vec<(usize, usize)>| {
                kept.iter()
                    .all(|(j, cj)| !isect(cands[i][ci].rect, cands[*j][*cj].rect))
            };
            let chosen = match pick[i] {
                Some(ci) if free_vs_kept(ci, &kept) => Some(ci),
                _ => {
                    let mut order_c: Vec<usize> = (0..cands[i].len()).collect();
                    order_c.sort_by(|a, b| cands[i][*a].cost.total_cmp(&cands[i][*b].cost));
                    order_c.into_iter().find(|ci| free_vs_kept(*ci, &kept))
                }
            };
            match chosen {
                Some(ci) => kept.push((i, ci)),
                None => dropped += 1,
            }
        }
        for (i, ci) in kept {
            let c = &cands[i][ci];
            sheet.label_padded(&reqs[i].3, c.x, c.y, SMALL_TEXT, reqs[i].4, 0);
            self.placed.push(c.rect);
        }
        if dropped > 0 {
            eprintln!("labeler: {dropped} label(s) dropped (no non-overlapping spot)");
        }
        if std::env::var("DEBUG_LABELS").is_ok() {
            for i in 0..n {
                let chosen = pick[i].map(|c| cands[i][c].cost);
                let best = cands[i].iter().map(|c| c.cost).fold(f64::MAX, f64::min);
                if let Some(ch) = chosen {
                    if ch > best + 60.0 {
                        eprintln!(
                            "label {:>14} at ({:>6.0},{:>6.0}): chosen cost {:>6.1}, best static {:>6.1}, cands {}",
                            reqs[i].3, reqs[i].0, reqs[i].1, ch, best, cands[i].len()
                        );
                    }
                }
            }
        }
    }
}

/// Arrowed dimension: line stops short of what it measures, arrowheads
/// point outward, the value label registers as a Labeler obstacle.
pub fn dim_arrow(
    sheet: &mut Sheet,
    lab: &mut Labeler,
    x0: f64,
    x1: f64,
    y: f64,
    txt: &str,
    color: Color,
) {
    let inset = 6.0;
    let (a, b) = (x0 + inset, x1 - inset);
    sheet.ctx.no_fill().stroke(color).stroke_width(PEN);
    sheet.ctx.line(a, y, b, y);
    for (tip, dir) in [(a, 1.0), (b, -1.0)] {
        sheet.ctx.line(tip, y, tip + dir * 14.0, y + 7.0);
        sheet.ctx.line(tip, y, tip + dir * 14.0, y - 7.0);
    }
    sheet.label_padded(txt, (x0 + x1) / 2.0, y + 16.0, SMALL_TEXT, color, 0);
    lab.obstacle_text((x0 + x1) / 2.0, y + 16.0, SMALL_TEXT, txt);
}
