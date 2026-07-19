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
//!     ~/GH/repos/font-garden-lab/runs/v08/pred.ufo   (pinned in inputs.rs)
//!     ~/GH/repos/google-fonts/ofl/geistmono/GeistMono[wght].ttf

use designbot::prelude::*;
use designbot_render::Renderer;
#[allow(unused_imports)]
use virtua_grotesk_figures::*;

// --- fig-model-review -----------------------------------------------------------

fn fig_review(
    renderer: &Renderer,
    mono: &str,
    _model_name: &str,
    reg: &std::path::Path,
    bold: &std::path::Path,
    pred: &std::path::Path,
    out: &std::path::Path,
) {
    let mut sheet = new_sheet(renderer, mono);

    const GLYPHS: [&str; 6] = ["K", "E", "M", "n", "b", "c"];
    const S: f64 = 0.43;

    // three bands between the rules, top to bottom
    let band_h = (HEADER_RULE_Y - FOOTER_RULE_Y) / 3.0;
    struct Row<'a> {
        color: Color,
        dir: &'a std::path::Path,
    }
    let rows = [
        Row {
            color: role::figure::red(),
            dir: reg,
        },
        Row {
            color: role::figure::orange(),
            dir: pred,
        },
        Row {
            color: role::figure::green(),
            dir: bold,
        },
    ];

    let slot_w = (W - 2.0 * MARGIN) / GLYPHS.len() as f64;

    for (i, row) in rows.iter().enumerate() {
        let band_top = HEADER_RULE_Y - i as f64 * band_h;
        let band_bottom = band_top - band_h;
        let baseline = band_bottom + 44.0;

        // baseline, house blue, behind the glyphs
        sheet
            .ctx
            .no_fill()
            .stroke(role::figure::pen())
            .stroke_width(line::THIN);
        sheet.ctx.line(MARGIN, baseline, W - MARGIN, baseline);

        for (j, name) in GLYPHS.iter().enumerate() {
            let slot_center = MARGIN + (j as f64 + 0.5) * slot_w;
            let glif = row.dir.join("glyphs").join(glif_name(name));
            if !glif.is_file() {
                sheet.label_padded(
                    "not in run",
                    slot_center,
                    baseline + 74.0,
                    24.0,
                    role::figure::pen(),
                    0,
                );
                continue;
            }
            let o = load_outline(&row.dir.join("glyphs"), name);
            let x = slot_center - o.width * S / 2.0;
            draw_body_styled(
                &mut sheet,
                &o,
                S,
                x,
                baseline,
                row.color,
                255,
                role::figure::pen(),
                line::HERO,
            );
        }
    }
    sheet.save(renderer, out);
}

// --- fig-model-bolden-a ----------------------------------------------------------

fn fig_bolden_a(
    renderer: &Renderer,
    mono: &str,
    _model_name: &str,
    reg: &std::path::Path,
    bold: &std::path::Path,
    pred: &std::path::Path,
    out: &std::path::Path,
) {
    let mut sheet = new_sheet(renderer, mono);

    const S: f64 = 1.62;
    const BASELINE: f64 = 214.0;

    // the Regular a from the sources; the Bold a from the detected run's
    // pred.ufo when it has one (keeps the label truthful), else the
    // committed draft in the Bold sources
    let o_reg = load_outline(&reg.join("glyphs"), "n");
    let pred_n = pred.join("glyphs/n.glif");
    let o_bold = if pred_n.is_file() {
        load_outline(&pred.join("glyphs"), "n")
    } else {
        load_outline(&bold.join("glyphs"), "n")
    };

    // run layout: centered between the margins
    let gap = 260.0;
    let run_w = o_reg.width * S + gap + o_bold.width * S;
    let x_reg = MARGIN + (W - 2.0 * MARGIN - run_w) / 2.0;
    let x_bold = x_reg + o_reg.width * S + gap;

    // vertical metrics: solid baseline + x-height, dashed overshoots
    {
        let ctx = &mut sheet.ctx;
        ctx.no_fill().stroke(blue()).stroke_width(line::HERO);
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
    draw_figure_glyph(&mut sheet, &o_reg, S, x_reg, BASELINE, role::figure::red());
    draw_figure_glyph(
        &mut sheet,
        &o_bold,
        S,
        x_bold,
        BASELINE,
        role::figure::green(),
    );

    // arrow between, at half x-height
    {
        let y = BASELINE + 288.0 * S;
        let x0 = x_reg + o_reg.width * S + 70.0;
        let x1 = x_bold - 70.0;
        let ctx = &mut sheet.ctx;
        ctx.no_fill().stroke(role::figure::pen()).stroke_width(10.0);
        ctx.line(x0, y, x1, y);
        ctx.line(x1 - 32.0, y + 22.0, x1, y);
        ctx.line(x1 - 32.0, y - 22.0, x1, y);
    }
    sheet.save(renderer, out);
}

// --- main ------------------------------------------------------------------------

fn main() {
    let mono_path = inputs::geist_mono();
    let sources = inputs::virtua_sources();
    let reg = sources.join("VirtuaGrotesk-Regular.ufo");
    let bold = sources.join("VirtuaGrotesk-Bold.ufo");
    let pred = inputs::model_prediction();
    let model_name = inputs::MODEL_NAME.to_string();
    assert!(
        pred.is_dir(),
        "missing pinned model input: {}",
        pred.display()
    );
    println!("model: {model_name} ({})", pred.display());

    let mut renderer = Renderer::new(W as u32, H as u32);
    let mono = load_family(&mut renderer, mono_path.to_str().unwrap());

    let outputs = OutputPaths::from_args();

    fig_review(
        &renderer,
        &mono,
        &model_name,
        &reg,
        &bold,
        &pred,
        &outputs.blog("fig-model-review.png"),
    );
    fig_bolden_a(
        &renderer,
        &mono,
        &model_name,
        &reg,
        &bold,
        &pred,
        &outputs.blog("fig-model-bolden-n.png"),
    );
}
