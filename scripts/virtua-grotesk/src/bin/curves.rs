//! fig-losscurve.png : the v0.8 A/B as training curves, parsed straight
//! from the run log so the figure cannot drift from the experiment.
//!
//!   left panel  : B1, pretraining on 29k OFL pairs (val = held-out OFL)
//!   right panel : A (control, graded corpus only) vs B2 (finetune from
//!                 the pretrained weights) on the SAME Virtua val set —
//!                 the pretrained arm starts lower and stays lower.
//!
//! Faint line = train loss every 100 steps; dots = val checkpoints
//! every 500; ring = best (saved) checkpoint.
//!
//!     cargo run --release --bin curves

use designbot::prelude::*;
use designbot_render::Renderer;
#[allow(unused_imports)]
use virtua_grotesk_figures::*;

struct Series {
    train: Vec<(f64, f64)>, // (step, loss)
    val: Vec<(f64, f64)>,
}

fn parse_log(path: &str) -> [Series; 3] {
    let text = std::fs::read_to_string(path).expect("run.log");
    let mut out = [
        Series {
            train: vec![],
            val: vec![],
        },
        Series {
            train: vec![],
            val: vec![],
        },
        Series {
            train: vec![],
            val: vec![],
        },
    ];
    let mut stage: Option<usize> = None;
    let mut last_step = [0.0f64; 3];
    for line in text.lines() {
        if line.starts_with("=== A:") {
            stage = Some(0);
        } else if line.starts_with("=== B1:") {
            stage = Some(1);
        } else if line.starts_with("=== B2:") {
            stage = Some(2);
        } else if line.starts_with("=== v0.8 finished") {
            stage = None;
        }
        let Some(s) = stage else { continue };
        let f = line.split_whitespace().collect::<Vec<_>>();
        // "checkpoint step 500 | val loss 3.1574 (best, saved)"
        if line.starts_with("checkpoint step") {
            out[s]
                .val
                .push((f[2].parse().unwrap(), f[6].parse().unwrap()));
        // "time budget reached at step 4937" then "checkpoint final | val loss 1.5300"
        } else if line.starts_with("time budget reached at step") {
            last_step[s] = f[5].parse().unwrap();
        } else if line.starts_with("checkpoint final") {
            out[s].val.push((last_step[s], f[5].parse().unwrap()));
        // "step    100 | loss 4.4714 | 46s"
        } else if line.starts_with("step ") && f.len() >= 5 {
            out[s]
                .train
                .push((f[1].parse().unwrap(), f[4].parse().unwrap()));
            last_step[s] = out[s].train.last().unwrap().0;
        }
    }
    out
}

struct Panel {
    x0: f64,
    y0: f64,
    w: f64,
    h: f64,
    max_step: f64,
    lo: f64,
    hi: f64,
}

impl Panel {
    fn x(&self, step: f64) -> f64 {
        self.x0 + step / self.max_step * self.w
    }
    fn y(&self, loss: f64) -> f64 {
        self.y0 + (loss - self.lo) / (self.hi - self.lo) * self.h
    }
}

fn polyline(sheet: &mut Sheet, p: &Panel, pts: &[(f64, f64)], color: Color, width: f64) {
    sheet.ctx.no_fill().stroke(color).stroke_width(width);
    for pair in pts.windows(2) {
        let (a, b) = (pair[0], pair[1]);
        sheet.ctx.line(p.x(a.0), p.y(a.1), p.x(b.0), p.y(b.1));
    }
}

fn val_dots(sheet: &mut Sheet, p: &Panel, pts: &[(f64, f64)], color: Color) {
    sheet.ctx.fill(color).no_stroke();
    for &(s, l) in pts {
        sheet.ctx.oval(p.x(s) - 10.0, p.y(l) - 10.0, 20.0, 20.0);
    }
    // ring the best checkpoint (the weights that shipped)
    if let Some(&(s, l)) = pts.iter().min_by(|a, b| a.1.total_cmp(&b.1)) {
        sheet.ctx.no_fill().stroke(color).stroke_width(PEN);
        sheet.ctx.oval(p.x(s) - 22.0, p.y(l) - 22.0, 44.0, 44.0);
    }
}

fn axes(sheet: &mut Sheet, p: &Panel, tick: f64) {
    let ink = role::chart::axis();
    let faint = role::chart::grid();
    // Horizontal guides only. The article explains the scale; the graphic is
    // optimized for the trajectories and their endpoints.
    let mut v = (p.lo / tick).ceil() * tick;
    while v <= p.hi + 1e-9 {
        sheet.ctx.no_fill().stroke(faint).stroke_width(PEN_LIGHT);
        sheet.ctx.line(p.x0, p.y(v), p.x0 + p.w, p.y(v));
        v += tick;
    }
    // x ticks every 2k steps
    let mut s = 0.0;
    while s <= p.max_step {
        sheet.ctx.no_fill().stroke(ink).stroke_width(PEN_LIGHT);
        sheet.ctx.line(p.x(s), p.y0, p.x(s), p.y0 - 10.0);
        s += 2000.0;
    }
    // frame: bottom + left rules only
    sheet.ctx.no_fill().stroke(ink).stroke_width(PEN);
    sheet.ctx.line(p.x0, p.y0, p.x0 + p.w, p.y0);
    sheet.ctx.line(p.x0, p.y0, p.x0, p.y0 + p.h);
}

fn range(series: &[&Series]) -> (f64, f64) {
    let mut lo = f64::MAX;
    let mut hi = f64::MIN;
    for s in series {
        for &(_, l) in s.train.iter().chain(s.val.iter()) {
            lo = lo.min(l);
            hi = hi.max(l);
        }
    }
    ((lo - 0.25).max(0.0), hi + 0.25)
}

fn max_step(series: &[&Series]) -> f64 {
    series
        .iter()
        .flat_map(|s| s.train.iter().chain(s.val.iter()))
        .map(|p| p.0)
        .fold(0.0, f64::max)
}

fn main() {
    let mono_path = inputs::geist_mono();
    let log = inputs::font_garden().join(inputs::LOSS_LOG);
    let [control, pretrain, finetune] = parse_log(log.to_str().unwrap());

    let mut renderer = Renderer::new(W as u32, H as u32);
    let mono = load_family(&mut renderer, mono_path.to_str().unwrap());
    let mut sheet = new_sheet(&renderer, &mono);

    let top = H - MARGIN;
    let bottom = MARGIN;
    let gap = 120.0;
    let left_w = (W - 2.0 * MARGIN - gap) / 2.0;
    let right_w = left_w;

    // left: pretraining on OFL (its own val set, so its own panel)
    let (lo, hi) = range(&[&pretrain]);
    let p1 = Panel {
        x0: MARGIN,
        y0: bottom,
        w: left_w,
        h: top - bottom,
        max_step: max_step(&[&pretrain]),
        lo,
        hi,
    };
    axes(&mut sheet, &p1, 1.0);
    polyline(
        &mut sheet,
        &p1,
        &pretrain.train,
        fill_strong(role::figure::red()),
        line::REGULAR,
    );
    polyline(&mut sheet, &p1, &pretrain.val, role::figure::red(), 10.0);
    val_dots(&mut sheet, &p1, &pretrain.val, role::figure::red());
    let best = pretrain
        .val
        .iter()
        .cloned()
        .fold((0.0, f64::MAX), |a, b| if b.1 < a.1 { b } else { a });
    sheet.label_padded(
        &format!("{:.2}", best.1),
        p1.x(best.0) - 22.0,
        p1.y(best.1) + 42.0,
        type_size::XXXL,
        role::figure::pen(),
        1,
    );

    // right: the A/B on the same Virtua val set
    let (lo, hi) = range(&[&control, &finetune]);
    let p2 = Panel {
        x0: MARGIN + left_w + gap,
        y0: bottom,
        w: right_w,
        h: top - bottom,
        max_step: max_step(&[&control, &finetune]),
        lo,
        hi,
    };
    axes(&mut sheet, &p2, 1.0);
    polyline(
        &mut sheet,
        &p2,
        &control.train,
        fill_strong(role::figure::orange()),
        line::REGULAR,
    );
    polyline(
        &mut sheet,
        &p2,
        &finetune.train,
        fill_strong(role::figure::green()),
        line::REGULAR,
    );
    polyline(&mut sheet, &p2, &control.val, role::figure::orange(), 10.0);
    polyline(&mut sheet, &p2, &finetune.val, role::figure::green(), 10.0);
    val_dots(&mut sheet, &p2, &control.val, role::figure::orange());
    val_dots(&mut sheet, &p2, &finetune.val, role::figure::green());
    let a_best = control
        .val
        .iter()
        .cloned()
        .fold((0.0, f64::MAX), |a, b| if b.1 < a.1 { b } else { a });
    let b_best = finetune
        .val
        .iter()
        .cloned()
        .fold((0.0, f64::MAX), |a, b| if b.1 < a.1 { b } else { a });
    sheet.label_padded(
        &format!("{:.2}", a_best.1),
        p2.x(a_best.0) + 26.0,
        p2.y(a_best.1) + 42.0,
        type_size::XXXL,
        role::figure::pen(),
        -1,
    );
    sheet.label_padded(
        &format!("{:.2}", b_best.1),
        p2.x(b_best.0) + 26.0,
        p2.y(b_best.1) - 10.0,
        type_size::XXXL,
        role::figure::pen(),
        -1,
    );

    let outputs = OutputPaths::from_args();
    sheet.save(&renderer, &outputs.blog("fig-losscurve.png"));
}
