//! Model-demo figures for the Virtua Grotesk post, §07, in the green
//! dimension-sheet house style (same family as og.rs / figs.rs):
//!
//!   fig-model-review.png   : held-out review sheet, 3 rows x 6 glyphs —
//!                            human Regular, model Bold, and human Bold
//!                            reference
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
use kurbo::Shape;
#[allow(unused_imports)]
use virtua_grotesk_figures::*;

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
    sheet.ctx.line_cap("round");

    const GLYPHS: [&str; 6] = ["K", "E", "M", "n", "b", "c"];
    const S: f64 = 0.40;
    const COLUMN_GAP: f64 = 166.0;
    const LABEL_TOP: f64 = H - 89.0;
    const ROW_STEP: f64 = 400.0;
    const LABEL_GAP: f64 = 44.0;

    struct Row<'a> {
        color: Color,
        dir: &'a std::path::Path,
        label: String,
    }
    let rows = [
        Row {
            color: role::figure::red(),
            dir: reg,
            label: "INPUT / MANUAL DRAWING / REGULAR".to_string(),
        },
        Row {
            color: role::figure::orange(),
            dir: pred,
            label: format!("OUTPUT / {model_name} / BOLD"),
        },
        Row {
            color: role::figure::green(),
            dir: bold,
            label: "REFERENCE / MANUAL DRAWING / BOLD".to_string(),
        },
    ];

    // Establish one shared six-column grid from the widest version of each
    // glyph. Every row uses these same optical centers, so the comparison is
    // stable vertically while the gaps remain visually balanced.
    let row_outlines: Vec<Vec<Option<Outline>>> = rows
        .iter()
        .map(|row| {
            GLYPHS
                .iter()
                .map(|name| {
                    let glif = row.dir.join("glyphs").join(glif_name(name));
                    glif.is_file()
                        .then(|| load_outline(&row.dir.join("glyphs"), name))
                })
                .collect()
        })
        .collect();
    let column_widths: Vec<f64> = (0..GLYPHS.len())
        .map(|column| {
            row_outlines
                .iter()
                .filter_map(|outlines| outlines[column].as_ref())
                .map(|o| o.path.bounding_box().width() * S)
                .fold(0.0, f64::max)
                .max(180.0)
        })
        .collect();
    let run_width =
        column_widths.iter().sum::<f64>() + COLUMN_GAP * (GLYPHS.len().saturating_sub(1)) as f64;
    let run_left = (W - run_width) / 2.0;
    let mut column_left = run_left;
    let column_centers: Vec<f64> = column_widths
        .iter()
        .map(|width| {
            let center = column_left + width / 2.0;
            column_left += width + COLUMN_GAP;
            center
        })
        .collect();

    for (row_index, (row, outlines)) in rows.iter().zip(&row_outlines).enumerate() {
        let label_y = LABEL_TOP - row_index as f64 * ROW_STEP;
        let row_top_units = outlines
            .iter()
            .filter_map(|outline| outline.as_ref())
            .map(|o| o.path.bounding_box().y1)
            .fold(0.0, f64::max);
        let baseline = label_y - LABEL_GAP - row_top_units * S;
        sheet.label_weighted(
            &row.label,
            run_left,
            label_y,
            type_size::MD,
            role::figure::pen(),
            -1,
            650.0,
        );

        for (outline, column_center) in outlines.iter().zip(&column_centers) {
            let Some(o) = outline.as_ref() else {
                sheet.label_padded(
                    "not in run",
                    *column_center,
                    baseline + 74.0,
                    24.0,
                    role::figure::pen(),
                    0,
                );
                continue;
            };
            let bounds = o.path.bounding_box();
            let ink_center = (bounds.x0 + bounds.x1) / 2.0;
            let x = column_center - ink_center * S;
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
    sheet.ctx.line_cap("round");

    const S: f64 = 1.72;
    const BASELINE: f64 = 154.0;

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
    let gap = 230.0;
    let run_w = o_reg.width * S + gap + o_bold.width * S;
    let x_reg = MARGIN + (W - 2.0 * MARGIN - run_w) / 2.0;
    let x_bold = x_reg + o_reg.width * S + gap;

    // Each half uses the real 8-unit source grid. Keeping the grids local
    // preserves their relationship to the two independent glyph origins.
    let panel_grid = |sheet: &mut Sheet, left: f64, right: f64, origin: f64| {
        let step = 8.0 * S;
        sheet
            .ctx
            .no_fill()
            .stroke(role::grid::faint())
            .stroke_width(line::FINE);
        let mut x = origin;
        while x > left {
            x -= step;
        }
        while x <= right {
            sheet.ctx.line(x, 0.0, x, H);
            x += step;
        }
        let mut y = BASELINE;
        while y > 0.0 {
            y -= step;
        }
        while y <= H {
            sheet.ctx.line(left, y, right, y);
            y += step;
        }
    };
    let divider = (x_reg + o_reg.width * S + x_bold) / 2.0;
    panel_grid(&mut sheet, 0.0, divider - 70.0, x_reg);
    panel_grid(&mut sheet, divider + 70.0, W, x_bold);

    // Shared font metrics use the same dark pen as every other construction.
    {
        let ctx = &mut sheet.ctx;
        ctx.no_fill()
            .stroke(role::figure::pen())
            .stroke_width(line::HERO);
        ctx.line_dash(&[14.0, 14.0]);
        for uy in [-16.0, 592.0] {
            ctx.line(0.0, BASELINE + uy * S, W, BASELINE + uy * S);
        }
        ctx.line_dash(&[]);
        for uy in [0.0, 576.0] {
            ctx.line(0.0, BASELINE + uy * S, W, BASELINE + uy * S);
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
        ctx.no_fill().stroke(role::figure::pen()).stroke_width(14.0);
        ctx.line(x0, y, x1, y);
        ctx.line(x1 - 38.0, y + 28.0, x1, y);
        ctx.line(x1 - 38.0, y - 28.0, x1, y);
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
