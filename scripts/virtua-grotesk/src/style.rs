//! Editable visual system for all Virtua Grotesk DesignBot figures.
//!
//! Work from top to bottom:
//! 1. `color` contains context-free illustration swatches; `og_color` is an
//!    independent context-free palette reserved for OG/share-card images.
//! 2. `line` and `type_size` contain context-free numeric scales.
//! 3. `role` maps those primitives to jobs in the figures.
//!
//! Figure binaries should use these values instead of embedding RGB or line
//! width literals. Layout and content still belong in each figure file.

use designbot::prelude::Color;

// --- perceptual color -------------------------------------------------------

/// Convert an OKLCH swatch to 8-bit sRGB, reducing chroma at constant
/// lightness and hue when the requested color falls outside the sRGB gamut.
///
/// Why this exists:
/// - sRGB channel values and HSL "saturation" are not perceptually uniform.
///   A red and a green with the same numeric HSL saturation can look wildly
///   different in strength.
/// - OKLCH is substantially more uniform, so a shared chroma is a useful
///   starting point for a visually even palette.
/// - Equal OKLCH chroma is still not a promise of identical appearance. The
///   CIE notes that surround, area, adaptation, and viewing conditions all
///   affect perceived saturation. Always review the palette at final size on
///   its actual background.
/// - Different hues hit the edge of sRGB at different chroma values. Reducing
///   chroma while holding lightness and hue is much safer than clipping an RGB
///   channel to 255, which is what made the old red disproportionately vivid.
///
/// References:
/// https://www.w3.org/TR/css-color-4/#ok-lab
/// https://www.w3.org/TR/css-color-4/#gamut-mapping
/// https://cie.co.at/eilv/198
fn oklch_srgb(lightness: f64, requested_chroma: f64, hue_degrees: f64) -> Color {
    fn linear_srgb(lightness: f64, chroma: f64, hue_degrees: f64) -> [f64; 3] {
        let hue = hue_degrees.to_radians();
        let a = chroma * hue.cos();
        let b = chroma * hue.sin();

        let l_root = lightness + 0.396_337_777_4 * a + 0.215_803_757_3 * b;
        let m_root = lightness - 0.105_561_345_8 * a - 0.063_854_172_8 * b;
        let s_root = lightness - 0.089_484_177_5 * a - 1.291_485_548 * b;
        let l = l_root.powi(3);
        let m = m_root.powi(3);
        let s = s_root.powi(3);

        [
            4.076_741_662_1 * l - 3.307_711_591_3 * m + 0.230_969_929_2 * s,
            -1.268_438_004_6 * l + 2.609_757_401_1 * m - 0.341_319_396_5 * s,
            -0.004_196_086_3 * l - 0.703_418_614_7 * m + 1.707_614_701 * s,
        ]
    }

    fn in_srgb_gamut(rgb: [f64; 3]) -> bool {
        rgb.into_iter()
            .all(|channel| (0.0..=1.0).contains(&channel))
    }

    fn encode_srgb(channel: f64) -> u8 {
        let encoded = if channel <= 0.003_130_8 {
            12.92 * channel
        } else {
            1.055 * channel.powf(1.0 / 2.4) - 0.055
        };
        (encoded.clamp(0.0, 1.0) * 255.0).round() as u8
    }

    // Find the largest in-gamut chroma while preserving the requested L and h.
    // This is the same broad strategy used by the CSS Color 4 gamut-mapping
    // algorithms, without their more elaborate just-noticeable-difference step.
    let mut low = 0.0;
    let mut high = requested_chroma;
    let mut rgb = linear_srgb(lightness, requested_chroma, hue_degrees);
    if !in_srgb_gamut(rgb) {
        for _ in 0..24 {
            let middle = (low + high) / 2.0;
            let candidate = linear_srgb(lightness, middle, hue_degrees);
            if in_srgb_gamut(candidate) {
                low = middle;
                rgb = candidate;
            } else {
                high = middle;
            }
        }
    }

    Color::rgb(
        encode_srgb(rgb[0]),
        encode_srgb(rgb[1]),
        encode_srgb(rgb[2]),
    )
}

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
    pub const XXL: f64 = 40.0;
    pub const XXXL: f64 = 44.0;
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
        Color::rgb(0xff, 0x4a, 0x35)
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

// --- OG / share-card primitive color swatches -------------------------------
// Independent from `color` so art direction here cannot silently change the
// inline illustrations in the post.

pub mod og_color {
    use super::{oklch_srgb, Color};

    // The four hero hues begin with a common OKLCH chroma. Their lightness
    // still varies deliberately to make the sequence read red -> orange ->
    // yellow -> green. Small named optical corrections are allowed after the
    // common baseline has been reviewed in the actual composition; they must
    // stay explicit here rather than becoming unexplained RGB tweaks. Keep
    // this palette in sRGB because social sites commonly normalize uploads to
    // sRGB during recompression.
    const HERO_CHROMA: f64 = 0.16;
    const RED_OPTICAL_CHROMA_BOOST: f64 = 0.015;

    pub fn gray_950() -> Color {
        Color::rgb(0x12, 0x12, 0x12)
    }
    pub fn gray_900() -> Color {
        Color::rgb(0x23, 0x23, 0x23)
    }
    pub fn gray_350() -> Color {
        Color::rgb(0x92, 0x92, 0x8e)
    }
    pub fn gray_300() -> Color {
        Color::rgb(0xb8, 0xb8, 0xb3)
    }
    pub fn gray_225() -> Color {
        Color::rgb(0xef, 0xef, 0xe9)
    }
    pub fn gray_200() -> Color {
        Color::rgb(0xf2, 0xed, 0xe4)
    }
    pub fn gray_850() -> Color {
        Color::rgb(0x34, 0x34, 0x34)
    }
    pub fn gray_625() -> Color {
        Color::rgb(0x50, 0x50, 0x50)
    }
    pub fn red() -> Color {
        oklch_srgb(0.66, HERO_CHROMA + RED_OPTICAL_CHROMA_BOOST, 28.0)
    }
    pub fn blue() -> Color {
        Color::rgb(0x4a, 0x78, 0xff)
    }
    pub fn orange() -> Color {
        oklch_srgb(0.74, HERO_CHROMA, 52.0)
    }
    pub fn yellow() -> Color {
        oklch_srgb(0.88, HERO_CHROMA, 92.0)
    }
    pub fn leaf_green() -> Color {
        oklch_srgb(0.67, HERO_CHROMA, 159.0)
    }
}

// --- semantic mappings -------------------------------------------------------
// These functions contain no RGB values. They only assign base swatches to
// drawing jobs, so palette edits and role edits remain separate decisions.

pub mod role {
    pub mod og {
        use super::super::{og_color, Color};

        pub fn background() -> Color {
            og_color::gray_350()
        }
        pub fn title() -> Color {
            og_color::gray_900()
        }
        pub fn dimension_line() -> Color {
            og_color::gray_900()
        }
        pub fn grid_minor() -> Color {
            og_color::gray_850()
        }
        pub fn grid_major() -> Color {
            og_color::gray_625()
        }
        pub fn structure_point() -> Color {
            og_color::gray_900()
        }
        pub fn correction_point() -> Color {
            og_color::gray_900()
        }
        pub fn structure_point_fill() -> Color {
            og_color::gray_300()
        }
        pub fn correction_point_fill() -> Color {
            og_color::gray_225()
        }
        pub fn glyph() -> Color {
            og_color::yellow()
        }
        pub fn measurement_accent() -> Color {
            og_color::red()
        }
        pub fn construction() -> Color {
            og_color::gray_900()
        }
        pub fn gradient_1() -> Color {
            og_color::red()
        }
        pub fn gradient_2() -> Color {
            og_color::orange()
        }
        pub fn gradient_3() -> Color {
            og_color::yellow()
        }
        pub fn gradient_4() -> Color {
            og_color::leaf_green()
        }
    }

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
