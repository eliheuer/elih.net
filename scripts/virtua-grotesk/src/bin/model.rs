//! Model-demo figure for the Virtua Grotesk post, §07, in the shared
//! technical-drawing house style:
//!
//!   fig-model-bolden-n.png : manually drawn Regular n beside the Bold n
//!                            drawn by the pinned model run
//!
//! Run from this directory:
//!
//!     cargo run --release --bin model
//!
//! Inputs read at render time, from sibling checkouts:
//!     ~/GH/repos/virtua-grotesk/sources/VirtuaGrotesk-{Regular,Bold}.ufo
//!     ~/GH/repos/font-garden-lab/runs/v08/pred.ufo   (pinned in inputs.rs)
//!     ~/GH/repos/google-fonts/ofl/geistmono/GeistMono[wght].ttf

use designbot_render::Renderer;
use kurbo::Shape;
#[allow(unused_imports)]
use virtua_grotesk_figures::*;

// --- fig-model-bolden-n ----------------------------------------------------------

fn fig_bolden_n(
    renderer: &Renderer,
    mono: &str,
    reg: &std::path::Path,
    bold: &std::path::Path,
    pred: &std::path::Path,
    out: &std::path::Path,
) {
    let mut sheet = new_sheet(renderer, mono);
    sheet.ctx.line_cap("round");

    const S: f64 = 1.74;
    const GRID_UNITS: f64 = 32.0;
    const GRID_STEP: f64 = GRID_UNITS * S;
    const GRID_COLUMNS: usize = 20;
    const GRID_ROWS: usize = 21;
    const PANEL_STROKE: f64 = line::HERO;
    // Optical midpoint between the exact chamfer-touch size (21.84 px at this
    // scale and stroke) and the previous 28 px markers.
    const POINT_SIZE: f64 = 24.92;
    const PANEL_W: f64 = GRID_COLUMNS as f64 * GRID_STEP;
    const PANEL_H: f64 = GRID_ROWS as f64 * GRID_STEP;
    const PANEL_INSET: f64 = (H - PANEL_H) / 2.0;
    const PANEL_GAP: f64 = W - 2.0 * PANEL_INSET - 2.0 * PANEL_W;

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

    let panel_lefts = [PANEL_INSET, PANEL_INSET + PANEL_W + PANEL_GAP];
    let outlines = [&o_reg, &o_bold];
    let fills = [role::figure::yellow(), role::figure::blue()];
    let mut placements = Vec::new();

    // Center each visible outline inside an identical panel, then snap the
    // glyph origin and baseline to the shared 32-unit grid. This keeps both
    // panel grids in the same phase without moving source points off-grid.
    for (panel_left, outline) in panel_lefts.iter().zip(outlines) {
        let bounds = outline.path.bounding_box();
        let centered_x = panel_left + PANEL_W / 2.0 - (bounds.x0 + bounds.x1) * S / 2.0;
        let centered_baseline = PANEL_INSET + PANEL_H / 2.0 - (bounds.y0 + bounds.y1) * S / 2.0;
        let x = panel_left + ((centered_x - panel_left) / GRID_STEP).round() * GRID_STEP;
        let baseline =
            PANEL_INSET + ((centered_baseline - PANEL_INSET) / GRID_STEP).round() * GRID_STEP;
        placements.push((x, baseline));
    }

    // Both panels use the same 32-unit source grid phase. The glyph origins
    // above are snapped to this grid, so source coordinates remain truthful.
    let panel_grid = |sheet: &mut Sheet, left: f64| {
        let right = left + PANEL_W;
        let top = PANEL_INSET + PANEL_H;
        sheet
            .ctx
            .no_fill()
            .stroke(with_alpha(role::grid::standard(), 128))
            .stroke_width(line::HERO);
        for column in 0..=GRID_COLUMNS {
            let x = left + column as f64 * GRID_STEP;
            sheet.ctx.line(x, PANEL_INSET, x, top);
        }
        for row in 0..=GRID_ROWS {
            let y = PANEL_INSET + row as f64 * GRID_STEP;
            sheet.ctx.line(left, y, right, y);
        }
    };
    for ((panel_left, (x, baseline)), outline) in
        panel_lefts.iter().zip(placements.iter()).zip(outlines)
    {
        panel_grid(&mut sheet, *panel_left);

        draw_body_styled(
            &mut sheet,
            outline,
            S,
            *x,
            *baseline,
            fills[if *panel_left == panel_lefts[0] { 0 } else { 1 }],
            255,
            role::figure::pen(),
            line::HERO,
        );
    }

    for (outline, (x, baseline)) in outlines.iter().zip(placements.iter()) {
        let point_style = PointStyle {
            smooth_size: POINT_SIZE,
            corner_size: POINT_SIZE,
            off_curve_size: POINT_SIZE,
            correction_filled: false,
            stroke_width: line::HERO,
        };
        let on_color = role::figure::pen();
        let off_color = role::figure::pen();
        let on_fill = role::figure::green();
        let off_fill = role::figure::red();
        draw_points_styled(
            &mut sheet,
            outline,
            S,
            *x,
            *baseline,
            role::figure::pen(),
            on_color,
            off_color,
            on_fill,
            off_fill,
            point_style,
        );
    }

    // Borders are drawn last so the local grids stop cleanly at each panel.
    for panel_left in panel_lefts {
        sheet
            .ctx
            .no_fill()
            .stroke(role::figure::pen())
            .stroke_width(PANEL_STROKE);
        sheet.ctx.rect(panel_left, PANEL_INSET, PANEL_W, PANEL_H);
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

    fig_bolden_n(
        &renderer,
        &mono,
        &reg,
        &bold,
        &pred,
        &outputs.blog("fig-model-bolden-n.png"),
    );
}
