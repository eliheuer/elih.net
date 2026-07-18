//! Editable visual system for all Virtua Grotesk DesignBot figures.
//!
//! Work from top to bottom:
//! 1. `color` contains context-free swatches.
//! 2. `line` and `type_size` contain context-free numeric scales.
//! 3. `role` maps those primitives to jobs in the figures.
//!
//! Figure binaries should use these values instead of embedding RGB or line
//! width literals. Layout and content still belong in each figure file.

use designbot::prelude::Color;

// --- canvas -----------------------------------------------------------------

pub const W: f64 = 2520.0;
pub const H: f64 = 1320.0;
pub const MARGIN: f64 = 64.0;
pub const HEADER_RULE_Y: f64 = H - MARGIN;
pub const FOOTER_RULE_Y: f64 = MARGIN;

// --- primitive line-width scale ---------------------------------------------

pub mod line {
    pub const HAIRLINE: f64 = 0.75;
    pub const FINE: f64 = 1.5;
    pub const THIN: f64 = 2.0;
    pub const MEDIUM: f64 = 2.5;
    pub const STRONG: f64 = 3.0;
    pub const CURVE: f64 = 3.5;
    pub const REGULAR: f64 = 4.0;
    pub const HEAVY: f64 = 5.0;
    pub const HERO: f64 = 6.0;
}

// --- primitive type-size scale ----------------------------------------------

pub mod type_size {
    pub const XS: f64 = 24.0;
    pub const SM: f64 = 26.0;
    pub const MD: f64 = 28.0;
    pub const LG: f64 = 30.0;
    pub const XL: f64 = 34.0;
}

// --- primitive color swatches ------------------------------------------------
// Names describe appearance only. No swatch name encodes a drawing purpose.

pub mod color {
    use super::Color;

    pub fn black_deep() -> Color {
        Color::rgb(0x0a, 0x0a, 0x0a)
    }
    pub fn gray_950() -> Color {
        Color::rgb(0x10, 0x10, 0x10)
    }
    pub fn gray_925() -> Color {
        Color::rgb(0x18, 0x18, 0x18)
    }
    pub fn gray_900() -> Color {
        Color::rgb(0x1a, 0x1a, 0x1a)
    }
    pub fn gray_875() -> Color {
        Color::rgb(0x1e, 0x1e, 0x1e)
    }
    pub fn gray_850() -> Color {
        Color::rgb(0x28, 0x28, 0x28)
    }
    pub fn gray_825() -> Color {
        Color::rgb(0x2c, 0x2c, 0x2c)
    }
    pub fn gray_800() -> Color {
        Color::rgb(0x30, 0x30, 0x30)
    }
    pub fn gray_700() -> Color {
        Color::rgb(0x3a, 0x3a, 0x3a)
    }
    pub fn gray_650() -> Color {
        Color::rgb(0x40, 0x40, 0x40)
    }
    pub fn gray_625() -> Color {
        Color::rgb(0x42, 0x42, 0x42)
    }
    pub fn gray_600() -> Color {
        Color::rgb(0x4a, 0x4a, 0x4a)
    }
    pub fn gray_550() -> Color {
        Color::rgb(0x5a, 0x5a, 0x5a)
    }
    pub fn gray_500() -> Color {
        Color::rgb(0x60, 0x60, 0x60)
    }
    pub fn gray_475() -> Color {
        Color::rgb(0x66, 0x66, 0x66)
    }
    pub fn gray_450() -> Color {
        Color::rgb(0x6a, 0x6a, 0x6a)
    }
    pub fn gray_400() -> Color {
        Color::rgb(0x78, 0x78, 0x78)
    }
    pub fn gray_350() -> Color {
        Color::rgb(0x8a, 0x8a, 0x8a)
    }
    pub fn gray() -> Color {
        gray_350()
    }
    pub fn gray_200() -> Color {
        Color::rgb(0xbe, 0xbe, 0xbe)
    }
    pub fn gray_100() -> Color {
        Color::rgb(0xe6, 0xe6, 0xe6)
    }
    pub fn green() -> Color {
        Color::rgb(0x15, 0xc4, 0x74)
    }
    pub fn green_muted() -> Color {
        Color::rgb(0x18, 0xb8, 0x6f)
    }
    pub fn orange() -> Color {
        Color::rgb(0xff, 0x98, 0x22)
    }
    pub fn orange_bright() -> Color {
        Color::rgb(0xff, 0x98, 0x0f)
    }
    pub fn orange_deep() -> Color {
        Color::rgb(0xff, 0x4e, 0x00)
    }
    pub fn yellow() -> Color {
        Color::rgb(0xff, 0xd2, 0x3c)
    }
    pub fn red() -> Color {
        Color::rgb(0xff, 0x45, 0x35)
    }
    pub fn blue() -> Color {
        Color::rgb(0x4a, 0x78, 0xff)
    }
    pub fn ultramarine() -> Color {
        Color::rgb(0x3d, 0x6b, 0xaa)
    }
    pub fn purple() -> Color {
        Color::rgb(0x8c, 0x6c, 0xff)
    }
}

pub use color::*;

// --- semantic mappings -------------------------------------------------------
// These functions contain no RGB values. They only assign base swatches to
// drawing jobs, so palette edits and role edits remain separate decisions.

pub mod role {
    pub mod canvas {
        use super::super::{color, Color};

        pub fn background() -> Color {
            color::gray_950()
        }
    }

    pub mod grid {
        use super::super::{color, Color};

        pub fn standard() -> Color {
            color::gray_850()
        }
        pub fn faint() -> Color {
            color::gray_900()
        }
        pub fn fine() -> Color {
            color::gray_875()
        }
        pub fn flat() -> Color {
            color::gray_800()
        }
        pub fn structure() -> Color {
            color::gray_825()
        }
        pub fn major() -> Color {
            color::gray_625()
        }
    }

    pub mod annotation {
        use super::super::{color, Color};

        pub fn dimensions() -> Color {
            color::gray_550()
        }
        pub fn secondary() -> Color {
            color::gray_350()
        }
    }

    pub mod bezier {
        use super::super::{color, Color};

        pub fn handles() -> Color {
            color::gray_450()
        }
        pub fn curve() -> Color {
            color::gray_100()
        }
    }

    pub mod chart {
        use super::super::{color, Color};

        pub fn axis() -> Color {
            color::gray_700()
        }
        pub fn grid() -> Color {
            color::gray_875()
        }
    }

    pub mod line {
        use super::super::line;

        pub const STANDARD: f64 = line::REGULAR;
        pub const LIGHT: f64 = line::REGULAR;
        pub const GRID: f64 = line::THIN;
    }

    pub mod text {
        use super::super::type_size;

        pub const FRAME: f64 = type_size::XL;
        pub const TAG: f64 = type_size::LG;
        pub const DIMENSION: f64 = type_size::LG;
        pub const LABEL: f64 = type_size::MD;
        pub const LEGEND: f64 = type_size::SM;
        pub const SMALL: f64 = type_size::MD;
    }
}

// Compatibility names for the existing layout code. New style work should
// prefer the explicit `role` modules above.
pub const PEN: f64 = role::line::STANDARD;
pub const PEN_LIGHT: f64 = role::line::LIGHT;
pub const FRAME_TEXT: f64 = role::text::FRAME;
pub const TAG_TEXT: f64 = role::text::TAG;
pub const DIM_TEXT: f64 = role::text::DIMENSION;
pub const LABEL_TEXT: f64 = role::text::LABEL;
pub const LEGEND_TEXT: f64 = role::text::LEGEND;
pub const SMALL_TEXT: f64 = role::text::SMALL;

pub fn with_alpha(color: Color, alpha: u8) -> Color {
    Color::rgba(color.r, color.g, color.b, alpha)
}

pub fn fill_of(stroke: Color) -> Color {
    with_alpha(stroke, 64)
}

pub fn fill_strong(stroke: Color) -> Color {
    with_alpha(stroke, 104)
}
