//! Pinned external inputs for reproducible figure renders.
//!
//! Edit this file when intentionally moving a figure to a new font, model
//! run, or experiment. Generators must not auto-select the newest local run.

use std::path::PathBuf;

pub const MODEL_NAME: &str = "Virtua-12M-v0.1";
pub const MODEL_PREDICTION: &str = "runs/v08/pred.ufo";
pub const COMPLETION_SVG: &str = "runs/v08/complete-R.svg";
pub const BOLDEN_SVG: &str = "runs/night1/bolden-g.svg";
pub const LOSS_LOG: &str = "runs/v08/run.log";

fn user_root() -> PathBuf {
    std::env::var_os("HOME")
        .map(PathBuf::from)
        .expect("HOME must identify the user checkout root")
}

pub fn geist_mono() -> PathBuf {
    user_root().join("GH/repos/google-fonts/ofl/geistmono/GeistMono[wght].ttf")
}

pub fn virtua_sources() -> PathBuf {
    user_root().join("GH/repos/virtua-grotesk/sources")
}

pub fn font_garden() -> PathBuf {
    user_root().join("GH/repos/font-garden-lab")
}

pub fn model_prediction() -> PathBuf {
    font_garden().join(MODEL_PREDICTION)
}
