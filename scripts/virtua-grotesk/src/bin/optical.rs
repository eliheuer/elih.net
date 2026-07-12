//! Optical-correction illustration for the Virtua Grotesk post, §03.
//!
//! A single kurbo cubic arch drawn in the Runebender point language (on-curve
//! = green ring, off-curve handle = purple ring, orange center dot) over the
//! powers-of-two grid. Every point lands on the 8-unit grid except one
//! off-curve handle, pulled 4 units off it (onto the 2-unit grid) and called
//! out in red as an optical correction: the one place the eye overrules the
//! machine grid. Dark OG palette, matching og.rs / the img2bez OG card.
//!
//! Run from this directory:
//!
//!     cargo run --release --bin optical
//!
//! Writes ../../src/content/blog/virtua-grotesk/fig-optical-correction.png.

use designbot::prelude::*;
use designbot_render::Renderer;
use kurbo::{Affine, BezPath};

const W: f64 = 2400.0; // same frame as the OG card
const H: f64 = 1260.0;
const UNIT: f64 = 14.0; // canvas px per font unit (8-unit cell = 112px)
const ORIGIN_Y: f64 = 140.0; // canvas y of unit y=0 (y-up)

// palette
fn bg() -> Color {
    Color::rgb(0x10, 0x10, 0x10)
}
fn grid_8() -> Color {
    Color::rgb(0x32, 0x32, 0x32)
}
fn grid_2() -> Color {
    Color::rgb(0x1e, 0x1e, 0x1e)
}
fn curve() -> Color {
    Color::rgb(230, 230, 230)
}
fn handle() -> Color {
    Color::rgb(120, 120, 120)
}
fn green() -> Color {
    Color::rgb(24, 184, 111)
}
fn purple() -> Color {
    Color::rgb(140, 108, 255)
}
fn orange() -> Color {
    Color::rgb(255, 152, 15)
}
fn red() -> Color {
    Color::rgb(0xff, 0x45, 0x35)
}

fn origin_x() -> f64 {
    ((W - 128.0 * UNIT) / 2.0).round()
}

fn cx(ux: f64) -> f64 {
    origin_x() + ux * UNIT
}
fn cy(uy: f64) -> f64 {
    ORIGIN_Y + uy * UNIT
}

/// A Runebender-style node: background-knockout ring plus an orange center dot.
fn node(ctx: &mut Canvas, x: f64, y: f64, ring: Color, r: f64, w: f64, dot: f64) {
    ctx.fill(bg()).stroke(ring).stroke_width(w);
    ctx.oval(x - r, y - r, r * 2.0, r * 2.0);
    ctx.no_stroke().fill(orange());
    ctx.oval(x - dot, y - dot, dot * 2.0, dot * 2.0);
}

fn main() {
    let home = std::env::var("HOME").unwrap();
    let mono_path = format!("{home}/GH/repos/google-fonts/ofl/geistmono/GeistMono[wght].ttf");

    let mut renderer = Renderer::new(W as u32, H as u32);
    let mono = load_family(&mut renderer, &mono_path);

    let mut ctx = Canvas::new(W, H);
    ctx.background(bg());

    // ── powers-of-two grid: faint 2-unit lattice, brighter 8-unit lattice ──
    let ox = origin_x();
    let u_lo = ((0.0 - ox) / UNIT).floor() as i64;
    let u_hi = ((W - ox) / UNIT).ceil() as i64;
    let mut u = u_lo - u_lo.rem_euclid(2);
    while u <= u_hi {
        let x = ox + u as f64 * UNIT;
        if u.rem_euclid(8) == 0 {
            ctx.stroke(grid_8()).stroke_width(2.0);
        } else {
            ctx.stroke(grid_2()).stroke_width(1.5);
        }
        ctx.line(x, 0.0, x, H);
        u += 2;
    }
    let v_lo = ((0.0 - ORIGIN_Y) / UNIT).floor() as i64;
    let v_hi = ((H - ORIGIN_Y) / UNIT).ceil() as i64;
    let mut v = v_lo - v_lo.rem_euclid(2);
    while v <= v_hi {
        let y = ORIGIN_Y + v as f64 * UNIT;
        if v.rem_euclid(8) == 0 {
            ctx.stroke(grid_8()).stroke_width(2.0);
        } else {
            ctx.stroke(grid_2()).stroke_width(1.5);
        }
        ctx.line(0.0, y, W, y);
        v += 2;
    }

    // ── the arch, in font units (y-up); every coordinate is a multiple of 8
    //    except the apex on-curve point, overshot +4 off the 8-grid (its
    //    extremum handles rise with it so the tangent stays horizontal) ──
    let l = (0.0, 0.0);
    let t = (64.0, 68.0); // grid intent is 64; +4 overshoot, the correction
    let r = (128.0, 0.0);
    let l_up = (0.0, 40.0);
    let t_left = (24.0, 68.0);
    let t_right = (104.0, 68.0);
    let r_up = (128.0, 40.0);

    let mut path = BezPath::new();
    path.move_to(l);
    path.curve_to(l_up, t_left, t);
    path.curve_to(t_right, r_up, r);
    let to_canvas = Affine::translate((ox, ORIGIN_Y)) * Affine::scale(UNIT);

    // handles (thin gray lines)
    for (on, off) in [(l, l_up), (t, t_left), (t, t_right), (r, r_up)] {
        ctx.stroke(handle()).stroke_width(2.5);
        ctx.line(cx(on.0), cy(on.1), cx(off.0), cy(off.1));
    }

    // the curve
    ctx.no_fill().stroke(curve()).stroke_width(5.0);
    ctx.draw_path(to_canvas * path);

    // off-curve handles (purple), on-curve base points (green), corrected apex (red)
    for off in [l_up, t_left, t_right, r_up] {
        node(&mut ctx, cx(off.0), cy(off.1), purple(), 19.0, 6.0, 6.0);
    }
    for on in [l, r] {
        node(&mut ctx, cx(on.0), cy(on.1), green(), 24.0, 6.0, 7.0);
    }
    node(&mut ctx, cx(t.0), cy(t.1), red(), 26.0, 6.5, 7.5);

    // ── the callout, under the point (in the arch's open interior) ──
    let px = cx(t.0);
    let py = cy(t.1);
    ctx.stroke(red()).stroke_width(2.5);
    ctx.line(px, py - 27.0, px, py - 96.0);

    label(&mut ctx, &renderer, &mono, "OPTICAL CORRECTION", px, py - 150.0, 42.0, red());
    label(&mut ctx, &renderer, &mono, "4 UNITS OFF THE 8-GRID", px, py - 198.0, 30.0, handle());

    let here = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let repo = here.parent().unwrap().parent().unwrap();
    let out = repo.join("src/content/blog/virtua-grotesk/fig-optical-correction.png");
    std::fs::create_dir_all(out.parent().unwrap()).unwrap();
    renderer.render_to_png(&ctx, out.to_str().unwrap()).unwrap();
    println!("wrote {}", out.display());
}

/// Center a Geist Mono label with its baseline at y.
fn label(ctx: &mut Canvas, renderer: &Renderer, mono: &str, txt: &str, x: f64, y: f64, size: f64, color: Color) {
    let w = renderer.text_width(txt, Some(mono), size, &[]);
    ctx.font(mono)
        .clear_font_variations()
        .font_size(size)
        .fill(color)
        .text_align(TextAlign::Left)
        .text(txt, x - w / 2.0, y);
}

// --- minimal sfnt reader (family name for ctx.font()), from og.rs ------------

fn read_u16(data: &[u8], offset: usize) -> u16 {
    u16::from_be_bytes([data[offset], data[offset + 1]])
}
fn read_u32(data: &[u8], offset: usize) -> u32 {
    u32::from_be_bytes([data[offset], data[offset + 1], data[offset + 2], data[offset + 3]])
}
fn find_table(data: &[u8], tag: &[u8; 4]) -> Option<usize> {
    let num_tables = read_u16(data, 4) as usize;
    (0..num_tables)
        .map(|i| 12 + i * 16)
        .find(|&rec| &data[rec..rec + 4] == tag)
        .map(|rec| read_u32(data, rec + 8) as usize)
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
    panic!("no Windows family name record in {path}");
}
