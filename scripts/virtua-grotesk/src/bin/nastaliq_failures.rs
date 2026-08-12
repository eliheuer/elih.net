//! Three unseen-word failures from the distilled Gulzar model.
//!
//! Run from this directory:
//!
//!     cargo run --release --bin nastaliq-failures
//!
//! The teacher is shaped directly with HarfRust. The model outline comes from
//! the post-opentype distillation CLI. Both are drawn as paths with Designbot.

use designbot::prelude::*;
use designbot_render::Renderer;
use kurbo::{Affine, BezPath, Shape};
use serde_json::Value;
use std::path::{Path, PathBuf};
use std::process::Command;
use virtua_grotesk_figures::{color, write_png, OutputPaths};

const WIDTH: f64 = 1800.0;
const HEIGHT: f64 = 720.0;
const MARGIN: f64 = 60.0;
const GAP: f64 = 60.0;
const ROW_HEIGHT: f64 = 280.0;
const TEACHER_CENTER_Y: f64 = 500.0;
const MODEL_CENTER_Y: f64 = 190.0;
const WORDS: [&str; 3] = ["المستشفيات", "المعلومات", "التقليدية"];

#[derive(Default)]
struct OutlineBuilder {
    path: BezPath,
}

impl ttf_parser::OutlineBuilder for OutlineBuilder {
    fn move_to(&mut self, x: f32, y: f32) {
        self.path.move_to((x as f64, y as f64));
    }

    fn line_to(&mut self, x: f32, y: f32) {
        self.path.line_to((x as f64, y as f64));
    }

    fn quad_to(&mut self, x1: f32, y1: f32, x: f32, y: f32) {
        self.path
            .quad_to((x1 as f64, y1 as f64), (x as f64, y as f64));
    }

    fn curve_to(&mut self, x1: f32, y1: f32, x2: f32, y2: f32, x: f32, y: f32) {
        self.path.curve_to(
            (x1 as f64, y1 as f64),
            (x2 as f64, y2 as f64),
            (x as f64, y as f64),
        );
    }

    fn close(&mut self) {
        self.path.close_path();
    }
}

fn site_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("figure crate must live at scripts/virtua-grotesk")
        .to_path_buf()
}

fn source_root() -> PathBuf {
    site_root()
        .parent()
        .expect("site repository must have a parent")
        .join("post-opentype")
}

fn teacher_outline(font_bytes: &[u8], word: &str) -> BezPath {
    let font_ref = harfrust::FontRef::from_index(font_bytes, 0).expect("invalid Gulzar font");
    let shaper_data = harfrust::ShaperData::new(&font_ref);
    let shaper = shaper_data.shaper(&font_ref).build();
    let face = ttf_parser::Face::parse(font_bytes, 0).expect("invalid Gulzar font");

    let mut buffer = harfrust::UnicodeBuffer::new();
    buffer.push_str(word);
    buffer.guess_segment_properties();
    let shaped = shaper.shape(buffer, harfrust::ShapeOptions::default());

    let mut result = BezPath::new();
    let mut pen_x = 0.0;
    let mut pen_y = 0.0;
    for (info, position) in shaped.glyph_infos().iter().zip(shaped.glyph_positions()) {
        let mut builder = OutlineBuilder::default();
        face.outline_glyph(ttf_parser::GlyphId(info.glyph_id as u16), &mut builder);
        let placed = Affine::translate((
            pen_x + position.x_offset as f64,
            pen_y + position.y_offset as f64,
        )) * builder.path;
        result.extend(placed.elements().iter().copied());
        pen_x += position.x_advance as f64;
        pen_y += position.y_advance as f64;
    }
    result
}

fn model_outline(word: &str) -> BezPath {
    let root = site_root();
    let source = source_root();
    let output = Command::new(source.join("target/release/distill"))
        .args([
            "wordjson",
            root.join("public/demos/neuraltype/gulzar.ntf")
                .to_str()
                .unwrap(),
            word,
        ])
        .output()
        .expect("run distill wordjson");
    assert!(output.status.success(), "distill wordjson failed");
    let record: Value = serde_json::from_slice(&output.stdout).expect("wordjson output");
    let d = record["d"].as_str().expect("wordjson path");
    let upm = record["upm"].as_f64().expect("wordjson upm");
    let em_px = record["em_px"].as_f64().expect("wordjson em_px");
    let path = BezPath::from_svg(d).expect("model SVG path");
    Affine::scale_non_uniform(upm / em_px, -(upm / em_px)) * path
}

fn centered(path: &BezPath, x: f64, y: f64, scale: f64) -> BezPath {
    let bounds = path.bounding_box();
    let center = bounds.center();
    Affine::translate((x, y))
        * Affine::scale(scale)
        * Affine::translate((-center.x, -center.y))
        * path.clone()
}

fn main() {
    let source = source_root();
    let font_bytes =
        std::fs::read(source.join("data/Gulzar-Regular.ttf")).expect("read Gulzar-Regular.ttf");
    let pairs: Vec<(BezPath, BezPath)> = WORDS
        .iter()
        .map(|word| (teacher_outline(&font_bytes, word), model_outline(word)))
        .collect();

    let slot_width = (WIDTH - 2.0 * MARGIN - 2.0 * GAP) / WORDS.len() as f64;
    let mut canvas = Canvas::new(WIDTH, HEIGHT);
    canvas.background(color::black_deep()).no_stroke();

    for (index, (teacher, model)) in pairs.iter().enumerate() {
        let teacher_bounds = teacher.bounding_box();
        let model_bounds = model.bounding_box();
        let max_width = teacher_bounds.width().max(model_bounds.width());
        let max_height = teacher_bounds.height().max(model_bounds.height());
        let scale = ((slot_width - 40.0) / max_width).min((ROW_HEIGHT - 30.0) / max_height);
        let center_x = MARGIN + slot_width / 2.0 + index as f64 * (slot_width + GAP);

        canvas.fill(color::gray_475()).draw_path(centered(
            teacher,
            center_x,
            TEACHER_CENTER_Y,
            scale,
        ));
        canvas
            .fill(color::green())
            .draw_path(centered(model, center_x, MODEL_CENTER_Y, scale));
    }

    let renderer = Renderer::new(WIDTH as u32, HEIGHT as u32);
    let outputs = OutputPaths::from_args();
    write_png(
        &renderer,
        &canvas,
        &outputs.blog_post("nastaliq-distilled", "unseen-words.png"),
    );
}
