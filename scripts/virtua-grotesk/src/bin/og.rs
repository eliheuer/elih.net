//! OG / share card for the Virtua Grotesk post: the word "Grid" at 1:1
//! (one font unit = one canvas pixel at scale 1), on the family frame with
//! the full point language — green points on the 8-unit machine grid, red
//! points off 8 on the 2-unit human grid. Hero fill (fill_strong), the
//! design grid, vertical metrics with tags, and the advance/sidebearing
//! dimension zone below the baseline.
//!
//! REBUILD after editing (from this directory):
//!     cargo run --release --bin og
//!
//! Writes BOTH outputs:
//!     ../../src/content/blog/virtua-grotesk/share-card.png   (post hero)
//!     ../../public/og/virtua-grotesk.png                     (og:image)

use designbot_render::Renderer;
use virtua_grotesk_figures::*;

const GLYPHS: &[&str] = &["G", "r", "i", "d"];

fn ink(o: &Outline) -> (f64, f64) {
    let mut lo = f64::INFINITY;
    let mut hi = f64::NEG_INFINITY;
    for (x, _, _) in &o.points {
        lo = lo.min(*x);
        hi = hi.max(*x);
    }
    (lo, hi)
}

fn main() {
    let home = std::env::var("HOME").unwrap();
    let mono_path = format!("{home}/GH/repos/google-fonts/ofl/geistmono/GeistMono[wght].ttf");
    let glyphs_dir = std::path::PathBuf::from(&home)
        .join("GH/repos/virtua-grotesk/sources/VirtuaGrotesk-Regular.ufo/glyphs");

    let mut renderer = Renderer::new(W as u32, H as u32);
    let mono = load_family(&mut renderer, &mono_path);
    let mut sheet = new_sheet(&renderer, &mono);

    let outlines: Vec<Outline> = GLYPHS.iter().map(|g| load_outline(&glyphs_dir, g)).collect();
    let run: f64 = outlines.iter().map(|o| o.width).sum();

    const S: f64 = 1.0;
    let f = Frame {
        s: S,
        x0: MARGIN + (W - 2.0 * MARGIN - run * S) / 2.0,
        baseline: 375.0,
    };

    draw_grid(&mut sheet, &f, 784.0, -80.0);
    metric_lines(&mut sheet, &f, &[0.0, 576.0, 768.0], &[784.0, -16.0]);

    // advance-boundary cell dividers: each sort in its own cell, the
    // divider dropping from the top overshoot down to the deeper of the
    // two adjacent dimension rows, with knockout nodes at the ends
    let row_y = |j: i64| {
        if j % 2 == 0 {
            f.baseline - 188.0
        } else {
            f.baseline - 140.0
        }
    };
    {
        let mut bounds = vec![0.0];
        let mut acc = 0.0;
        for o in &outlines {
            acc += o.width;
            bounds.push(acc);
        }
        let n = outlines.len() as i64;
        for (i, &b) in bounds.iter().enumerate() {
            let i = i as i64;
            let deepest = row_y(i.min(n - 1)).min(row_y((i - 1).clamp(0, n - 1)));
            cell_dividers(&mut sheet, &[f.x(b)], f.y(784.0), deepest);
        }
    }

    // glyphs: hero fill + grid-level point language
    let mut ox = 0.0;
    for o in &outlines {
        draw_body_strong(&mut sheet, o, S, f.x(ox), f.baseline, red());
        draw_points(&mut sheet, o, S, f.x(ox), f.baseline);
        ox += o.width;
    }

    // advance / sidebearing dimension zone, staggered rows like a real sheet
    let mut ox = 0.0;
    for (j, o) in outlines.iter().enumerate() {
        let (i0, i1) = ink(o);
        advance_row(&mut sheet, &f, row_y(j as i64), &[(ox, o.width)], &[(i0, i1)]);
        ox += o.width;
    }

    // metric tags
    sheet.metric_tag("CAP 768", MARGIN, f.y(768.0), false, -1);
    sheet.metric_tag("X-HEIGHT 576", MARGIN, f.y(576.0), true, -1);
    sheet.metric_tag("BASELINE 0", MARGIN, f.y(0.0), true, -1);
    sheet.metric_tag("OVERSHOOT +16", W - MARGIN, f.y(784.0), false, 1);
    sheet.metric_tag("OVERSHOOT -16", W - MARGIN, f.y(-16.0), false, 1);

    legend(&mut sheet, W - MARGIN, f.baseline - 214.0);

    sheet.frame(
        "VIRTUA GROTESK",
        "POWERS OF TWO GRID / UPM 1024 = 2^10",
        "REGULAR 400 / SIL OPEN FONT LICENSE (OFL) VERSION 1.1",
    );

    let here = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let root = here.parent().unwrap().parent().unwrap();
    sheet.save(
        &renderer,
        &root.join("src/content/blog/virtua-grotesk/share-card.png"),
    );
    sheet.save(&renderer, &root.join("public/og/virtua-grotesk.png"));
}
