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
//!     ~/GH/repos/font-garden-lab/runs/vNN/pred.ufo   (newest run, auto-detected)
//!     ~/GH/repos/google-fonts/ofl/geistmono/GeistMono[wght].ttf

use designbot::prelude::*;
use designbot_render::Renderer;
use kurbo::Affine;
#[allow(unused_imports)]
use virtua_grotesk_figures::*;

/// Newest font-garden-lab run that produced a pred.ufo, e.g. runs/v07 ->
/// (".../runs/v07/pred.ufo", "VIRTUA-12M-0.7"). Keeps the figures pointed at
/// the latest model with no code edit per run.
fn latest_pred(home: &str) -> (std::path::PathBuf, String) {
    let runs = std::path::PathBuf::from(home).join("GH/repos/font-garden-lab/runs");
    let mut best: Option<(u32, std::path::PathBuf)> = None;
    for entry in std::fs::read_dir(&runs).unwrap().flatten() {
        let name = entry.file_name().to_string_lossy().to_string();
        if let Some(num) = name.strip_prefix('v').and_then(|s| s.parse::<u32>().ok()) {
            let pred = entry.path().join("pred.ufo");
            if pred.is_dir() && best.as_ref().is_none_or(|(b, _)| num > *b) {
                best = Some((num, pred));
            }
        }
    }
    let (num, pred) = best.expect("no runs/vNN/pred.ufo under font-garden-lab");
    // a run can pin its exact public name (e.g. the param count changed):
    // echo "VIRTUA-25M-0.9" > runs/vNN/model-name.txt
    let name = std::fs::read_to_string(pred.parent().unwrap().join("model-name.txt"))
        .map(|s| s.trim().to_uppercase())
        .unwrap_or_else(|_| format!("VIRTUA-12M-{}.{}", num / 10, num % 10));
    (pred, name)
}

// --- fig-model-review -----------------------------------------------------------

fn fig_review(
    renderer: &Renderer,
    mono: &str,
    model_name: &str,
    reg: &std::path::Path,
    bold: &std::path::Path,
    pred: &std::path::Path,
    out: &std::path::Path,
) {
    let mut sheet = new_sheet(renderer, mono);

    const GLYPHS: [&str; 7] = ["K", "E", "M", "n", "b", "c", "a"];
    const S: f64 = 0.32;

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
            label: "01 input / human Regular".into(),
            color: green(),
            dir: reg,
            skip_a: false,
        },
        Row {
            label: format!("02 output / {model_name}"),
            color: red(),
            dir: pred,
            skip_a: false,
        },
        Row {
            label: "03 reference / human Bold".into(),
            color: gray(),
            dir: bold,
            skip_a: true,
        },
    ];

    let slot_w = (W - 2.0 * MARGIN) / GLYPHS.len() as f64;

    for (i, row) in rows.iter().enumerate() {
        let band_top = HEADER_RULE_Y - i as f64 * band_h;
        let band_bottom = band_top - band_h;
        let baseline = band_bottom + 28.0;

        // baseline, house blue, behind the glyphs
        sheet.ctx.no_fill().stroke(blue()).stroke_width(2.0);
        sheet.ctx.line(MARGIN, baseline, W - MARGIN, baseline);

        sheet.label(&row.label, MARGIN, band_top - 28.0, 26.0, row.color, -1);

        for (j, name) in GLYPHS.iter().enumerate() {
            let slot_center = MARGIN + (j as f64 + 0.5) * slot_w;
            if row.skip_a && *name == "a" {
                sheet.label_padded(
                    "never boldened",
                    slot_center,
                    baseline + 74.0,
                    24.0,
                    dim_color(),
                    0,
                );
                continue;
            }
            let glif = row.dir.join("glyphs").join(glif_name(name));
            if !glif.is_file() {
                sheet.label_padded("not in run", slot_center, baseline + 74.0, 24.0, dim_color(), 0);
                continue;
            }
            let o = load_outline(&row.dir.join("glyphs"), name);
            let x = slot_center - o.width * S / 2.0;
            draw_body_strong(&mut sheet, &o, S, x, baseline, row.color);
        }
    }

    sheet.attribution(Some(&format!("held-out review / model: {model_name}")));
    sheet.save(renderer, out);
}

// --- fig-model-bolden-a ----------------------------------------------------------

fn fig_bolden_a(
    renderer: &Renderer,
    mono: &str,
    model_name: &str,
    reg: &std::path::Path,
    bold: &std::path::Path,
    pred: &std::path::Path,
    out: &std::path::Path,
) {
    let mut sheet = new_sheet(renderer, mono);

    const S: f64 = 1.45;
    const BASELINE: f64 = 242.0;
    let grid_bottom = MARGIN;
    let grid_top = H - MARGIN;

    // the Regular a from the sources; the Bold a from the detected run's
    // pred.ufo when it has one (keeps the label truthful), else the
    // committed draft in the Bold sources
    let o_reg = load_outline(&reg.join("glyphs"), "a");
    let pred_a = pred.join("glyphs/a.glif");
    let o_bold = if pred_a.is_file() {
        load_outline(&pred.join("glyphs"), "a")
    } else {
        load_outline(&bold.join("glyphs"), "a")
    };

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
        ctx.no_fill().stroke(blue()).stroke_width(PEN);
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
    draw_body_strong(&mut sheet, &o_reg, S, x_reg, BASELINE, green());
    draw_body_strong(&mut sheet, &o_bold, S, x_bold, BASELINE, red());
    draw_points_mono(&mut sheet, &o_reg, S, x_reg, BASELINE, green());
    draw_points_mono(&mut sheet, &o_bold, S, x_bold, BASELINE, red());

    // arrow between, at half x-height
    {
        let y = BASELINE + 288.0 * S;
        let x0 = x_reg + o_reg.width * S + 70.0;
        let x1 = x_bold - 70.0;
        let ctx = &mut sheet.ctx;
        ctx.no_fill().stroke(gray()).stroke_width(PEN);
        ctx.line(x0, y, x1, y);
        ctx.line(x1 - 22.0, y + 14.0, x1, y);
        ctx.line(x1 - 22.0, y - 14.0, x1, y);
    }

    // metric tags, docked at the left margin
    sheet.metric_tag("x-height 576", MARGIN, BASELINE + 576.0 * S, true, -1);
    sheet.metric_tag("baseline 0", MARGIN, BASELINE, true, -1);

    // captions under the run, centered per glyph
    let label_y = 132.0;
    sheet.label(
        "Regular / drawn by hand",
        x_reg + o_reg.width * S / 2.0,
        label_y,
        26.0,
        green(),
        0,
    );
    sheet.label(
        &format!("Bold / drawn by {model_name}"),
        x_bold + o_bold.width * S / 2.0,
        label_y,
        26.0,
        red(),
        0,
    );

    sheet.hud_title(&[
        "Weight transfer / the first Bold a",
        "the Bold master never had a real a; the model drew this one",
    ]);
    sheet.attribution(Some(&format!("model: {model_name}")));
    sheet.save(renderer, out);
}

// --- main ------------------------------------------------------------------------

fn main() {
    let home = std::env::var("HOME").unwrap();
    let mono_path = format!("{home}/GH/repos/google-fonts/ofl/geistmono/GeistMono[wght].ttf");
    let sources = std::path::PathBuf::from(&home).join("GH/repos/virtua-grotesk/sources");
    let reg = sources.join("VirtuaGrotesk-Regular.ufo");
    let bold = sources.join("VirtuaGrotesk-Bold.ufo");
    let (pred, model_name) = latest_pred(&home);
    println!("model: {model_name} ({})", pred.display());

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
        &model_name,
        &reg,
        &bold,
        &pred,
        &post.join("fig-model-review.png"),
    );
    fig_bolden_a(
        &renderer,
        &mono,
        &model_name,
        &reg,
        &bold,
        &pred,
        &post.join("fig-model-bolden-a.png"),
    );
}
