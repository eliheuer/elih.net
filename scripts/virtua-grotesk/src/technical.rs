//! Shared renderer for the section 03 technical-drawing style.
//!
//! A figure binary should contain only content: source outlines, source-space
//! placement, fill colors, metric values, and intentionally placed
//! measurements. Grid appearance, metric furniture, point language, line
//! weights, label typography, and spacing conventions belong here.
//!
//! `TechnicalStyle::section_three()` is the canonical preset. Use a named
//! modifier only when the figure's argument requires a real semantic change,
//! such as coloring points by grid level. Do not reproduce this renderer with
//! local constants in a figure binary.

use designbot::prelude::Color;

use crate::{
    cell_dividers_colored, draw_body_styled, draw_points_styled, line, marker_with_fill_sized,
    p2sum, role, Frame, Outline, PointStyle, PtRole, Sheet, H, MARGIN, W,
};

/// Meaning carried by point fills. Geometry still distinguishes smooth,
/// corner, and off-curve points in both modes.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TechnicalPointPalette {
    /// The canonical `no` and `HO` treatment: two light-gray point tiers.
    Neutral,
    /// Green points lie on the 8-unit structure grid; red points lie off it.
    GridLevel,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TechnicalGridTone {
    Faint,
    Subtle,
}

/// One reviewed technical-drawing language shared by all glyph sheets.
#[derive(Clone, Copy, Debug)]
pub struct TechnicalStyle {
    pub stroke: f64,
    pub grid_stroke: f64,
    pub grid_unit: f64,
    pub point_size: f64,
    pub measurement_text_size: f64,
    pub measurement_text_weight: f32,
    pub measurement_cap: f64,
    pub measurement_line_gap: f64,
    pub point_end_inset: f64,
    pub edge_end_inset: f64,
    pub point_palette: TechnicalPointPalette,
    pub grid_tone: TechnicalGridTone,
}

impl TechnicalStyle {
    /// The canonical visual preset established by `fig-system-no.png` and
    /// `fig-system-ho.png`.
    pub const fn section_three() -> Self {
        Self {
            stroke: line::HERO,
            grid_stroke: line::FINE,
            grid_unit: 8.0,
            point_size: 20.0,
            measurement_text_size: 32.0,
            measurement_text_weight: 600.0,
            measurement_cap: 12.0,
            measurement_line_gap: 28.0,
            point_end_inset: 30.0,
            edge_end_inset: 20.0,
            point_palette: TechnicalPointPalette::Neutral,
            grid_tone: TechnicalGridTone::Faint,
        }
    }

    /// Preserve the section 03 geometry and typography while making grid
    /// membership the point color's explicit semantic job.
    pub const fn with_grid_level_points(mut self) -> Self {
        self.point_palette = TechnicalPointPalette::GridLevel;
        self
    }

    /// Change the source-grid interval and pen while retaining the rest of
    /// the section 03 drawing language.
    pub const fn with_grid(mut self, unit: f64, stroke: f64) -> Self {
        self.grid_unit = unit;
        self.grid_stroke = stroke;
        self
    }

    pub const fn with_subtle_grid(mut self) -> Self {
        self.grid_tone = TechnicalGridTone::Subtle;
        self
    }

    /// Shift the whole source frame by less than half a grid cell so the
    /// first and last horizontal grid lines have equal canvas margins. Glyphs
    /// and points move with the grid, preserving truthful source alignment.
    pub fn center_grid_vertically(&self, frame: &mut Frame) {
        let step = self.grid_unit * frame.s;
        let target_phase = H.rem_euclid(step) / 2.0;
        let current_phase = frame.y(0.0).rem_euclid(step);
        let mut shift = target_phase - current_phase;
        if shift > step / 2.0 {
            shift -= step;
        } else if shift < -step / 2.0 {
            shift += step;
        }
        frame.baseline += shift;
    }

    fn grid_color(&self) -> Color {
        match self.grid_tone {
            TechnicalGridTone::Faint => role::grid::faint(),
            TechnicalGridTone::Subtle => role::grid::subtle(),
        }
    }

    /// Fit a source-space run inside the canonical outer margin.
    pub fn frame(&self, run: f64, bottom: f64, top: f64) -> Frame {
        let s = ((W - 2.0 * MARGIN) / run).min((H - 2.0 * MARGIN) / (top - bottom));
        Frame {
            s,
            x0: (W - run * s) / 2.0,
            baseline: MARGIN - bottom * s,
        }
    }

    /// Draw the real uniform source grid. Vertical coordinates restart at
    /// every supplied sort, matching a font editor.
    pub fn background_grid(
        &self,
        sheet: &mut Sheet,
        frame: &Frame,
        glyphs: &[(&Outline, f64)],
        bottom: f64,
        top: f64,
    ) {
        let x0 = frame.x(0.0);
        let x1 = frame.x(glyphs
            .last()
            .map(|(outline, origin)| origin + outline.width)
            .unwrap_or(0.0));

        let mut v = bottom;
        while v <= top {
            sheet
                .ctx
                .no_fill()
                .stroke(self.grid_color())
                .stroke_width(self.grid_stroke);
            sheet.ctx.line(x0, frame.y(v), x1, frame.y(v));
            v += self.grid_unit;
        }

        for (outline, origin) in glyphs {
            let mut u = 0.0;
            while u <= outline.width {
                sheet
                    .ctx
                    .no_fill()
                    .stroke(self.grid_color())
                    .stroke_width(self.grid_stroke);
                let x = frame.x(origin + u);
                sheet.ctx.line(x, frame.y(bottom), x, frame.y(top));
                u += self.grid_unit;
            }
        }
    }

    /// Draw one continuous source grid across the canvas. Use this when the
    /// composition aligns several outlines to one shared coordinate phase
    /// instead of restarting the grid at each sort boundary.
    pub fn continuous_background_grid(&self, sheet: &mut Sheet, frame: &Frame, x_anchor: f64) {
        let step = self.grid_unit * frame.s;
        sheet
            .ctx
            .no_fill()
            .stroke(self.grid_color())
            .stroke_width(self.grid_stroke);

        let mut x = frame.x(x_anchor).rem_euclid(step);
        while x <= W {
            if x > self.grid_stroke && x < W - self.grid_stroke {
                sheet.ctx.line(x, 0.0, x, crate::H);
            }
            x += step;
        }

        let mut y = frame.y(0.0).rem_euclid(step);
        while y <= crate::H {
            if y > self.grid_stroke && y < H - self.grid_stroke {
                sheet.ctx.line(0.0, y, W, y);
            }
            y += step;
        }
    }

    /// Draw horizontal font metrics without sort boundaries. This is useful
    /// for a continuous multi-glyph composition such as interpolation.
    pub fn metric_rules(
        &self,
        sheet: &mut Sheet,
        frame: &Frame,
        left: f64,
        right: f64,
        solid: &[f64],
        dashed: &[f64],
    ) {
        let x0 = frame.x(left);
        let x1 = frame.x(right);
        let color = role::figure::pen();

        sheet.ctx.no_fill().stroke(color).stroke_width(self.stroke);
        sheet.ctx.line_dash(&[10.0, 10.0]);
        for &uy in dashed {
            sheet.ctx.line(x0, frame.y(uy), x1, frame.y(uy));
        }
        sheet.ctx.line_dash(&[]);
        for &uy in solid {
            sheet.ctx.line(x0, frame.y(uy), x1, frame.y(uy));
        }
    }

    /// Draw metric rules, advance boundaries, and their construction nodes.
    pub fn metric_system(
        &self,
        sheet: &mut Sheet,
        frame: &Frame,
        run: f64,
        bounds: &[f64],
        solid: &[f64],
        dashed: &[f64],
        top: f64,
        bottom: f64,
    ) {
        let color = role::figure::pen();
        self.metric_rules(sheet, frame, 0.0, run, solid, dashed);

        let xs: Vec<f64> = bounds.iter().map(|bound| frame.x(*bound)).collect();
        cell_dividers_colored(
            sheet,
            &xs,
            frame.y(top),
            frame.y(bottom),
            color,
            role::figure::point_fill(),
            self.stroke,
        );

        let mut ys: Vec<f64> = dashed.iter().map(|uy| frame.y(*uy)).collect();
        ys.extend(solid.iter().map(|uy| frame.y(*uy)));
        for x in xs {
            for &y in &ys {
                marker_with_fill_sized(
                    sheet,
                    x,
                    y,
                    PtRole::Smooth,
                    color,
                    role::figure::point_fill(),
                    self.point_size,
                    self.stroke,
                );
            }
        }
    }

    /// Draw one opaque glyph with the canonical pen and point language.
    pub fn glyph(
        &self,
        sheet: &mut Sheet,
        outline: &Outline,
        frame: &Frame,
        origin: f64,
        fill: Color,
    ) {
        draw_body_styled(
            sheet,
            outline,
            frame.s,
            frame.x(origin),
            frame.baseline,
            fill,
            255,
            role::figure::pen(),
            self.stroke,
        );

        let (on_fill, off_fill) = match self.point_palette {
            TechnicalPointPalette::Neutral => (
                role::figure::point_fill(),
                role::figure::correction_point_fill(),
            ),
            TechnicalPointPalette::GridLevel => (role::figure::green(), role::figure::red()),
        };
        let point_style = PointStyle {
            smooth_size: self.point_size,
            corner_size: self.point_size,
            off_curve_size: self.point_size,
            correction_filled: false,
            stroke_width: self.stroke,
        };
        draw_points_styled(
            sheet,
            outline,
            frame.s,
            frame.x(origin),
            frame.baseline,
            role::figure::pen(),
            role::figure::pen(),
            role::figure::pen(),
            on_fill,
            off_fill,
            point_style,
        );
    }

    /// Draw an OG-style capped measurement with its power-of-two sum.
    pub fn measurement(
        &self,
        sheet: &mut Sheet,
        frame: &Frame,
        origin: f64,
        measurement: TechnicalMeasurement,
    ) {
        let p0 = (
            frame.x(origin + measurement.p0.0),
            frame.y(measurement.p0.1),
        );
        let p1 = (
            frame.x(origin + measurement.p1.0),
            frame.y(measurement.p1.1),
        );
        let dx = p1.0 - p0.0;
        let dy = p1.1 - p0.1;
        let length = (dx * dx + dy * dy).sqrt();
        let direction = (dx / length, dy / length);
        let normal = (-direction.1, direction.0);
        let end_inset = match measurement.ends {
            MeasurementEnds::Points => self.point_end_inset,
            MeasurementEnds::Edges => self.edge_end_inset,
            MeasurementEnds::Custom(value) => value,
        };
        let q0 = (
            p0.0 + direction.0 * end_inset,
            p0.1 + direction.1 * end_inset,
        );
        let q1 = (
            p1.0 - direction.0 * end_inset,
            p1.1 - direction.1 * end_inset,
        );
        let color = role::figure::pen();

        sheet.ctx.no_fill().stroke(color).stroke_width(self.stroke);
        if let Some((gap_center, gap_half_height)) = measurement.line_gap {
            let gap_center = frame.y(gap_center);
            sheet
                .ctx
                .line(q0.0, q0.1, q0.0, gap_center - gap_half_height);
            sheet
                .ctx
                .line(q1.0, gap_center + gap_half_height, q1.0, q1.1);
        } else {
            sheet.ctx.line(q0.0, q0.1, q1.0, q1.1);
        }
        for q in [q0, q1] {
            sheet.ctx.line(
                q.0 - normal.0 * self.measurement_cap,
                q.1 - normal.1 * self.measurement_cap,
                q.0 + normal.0 * self.measurement_cap,
                q.1 + normal.1 * self.measurement_cap,
            );
        }

        let midpoint = ((q0.0 + q1.0) / 2.0, (q0.1 + q1.1) / 2.0);
        let decomposition = p2sum(measurement.value);
        let parts: Vec<&str> = decomposition.split('+').collect();
        let sum_lines =
            if measurement.sum_break_after > 0 && measurement.sum_break_after < parts.len() {
                vec![
                    parts[..measurement.sum_break_after].join("+"),
                    format!("+{}", parts[measurement.sum_break_after..].join("+")),
                ]
            } else {
                vec![decomposition]
            };

        if dx.abs() >= dy.abs() {
            self.measurement_label(
                sheet,
                measurement,
                &measurement.value.to_string(),
                midpoint.0 + measurement.label_shift.0,
                midpoint.1 + 20.0 + measurement.label_shift.1,
                0,
            );
            for (index, line) in sum_lines.iter().enumerate() {
                self.measurement_label(
                    sheet,
                    measurement,
                    line,
                    midpoint.0 + measurement.label_shift.0,
                    midpoint.1 - 48.0 + measurement.label_shift.1
                        - index as f64 * self.measurement_line_gap,
                    0,
                );
            }
        } else {
            self.measurement_label(
                sheet,
                measurement,
                &measurement.value.to_string(),
                midpoint.0 - 16.0 + measurement.label_shift.0,
                midpoint.1 - 12.0 + measurement.label_shift.1,
                1,
            );
            for (index, line) in sum_lines.iter().enumerate() {
                self.measurement_label(
                    sheet,
                    measurement,
                    line,
                    midpoint.0 + 16.0 + measurement.label_shift.0,
                    midpoint.1 - 12.0 + measurement.label_shift.1
                        - index as f64 * self.measurement_line_gap,
                    -1,
                );
            }
        }
    }

    fn measurement_label(
        &self,
        sheet: &mut Sheet,
        measurement: TechnicalMeasurement,
        text: &str,
        x: f64,
        y: f64,
        align: i8,
    ) {
        if measurement.knockout {
            sheet.label_padded_weighted_on(
                text,
                x,
                y,
                self.measurement_text_size,
                role::figure::pen(),
                align,
                role::figure::background(),
                self.measurement_text_weight,
            );
        } else {
            sheet.label_weighted(
                text,
                x,
                y,
                self.measurement_text_size,
                role::figure::pen(),
                align,
                self.measurement_text_weight,
            );
        }
    }
}

#[derive(Clone, Copy, Debug)]
enum MeasurementEnds {
    Points,
    Edges,
    Custom(f64),
}

/// Source-space content for one manually placed measurement.
#[derive(Clone, Copy, Debug)]
pub struct TechnicalMeasurement {
    pub glyph: usize,
    p0: (f64, f64),
    p1: (f64, f64),
    value: i64,
    label_shift: (f64, f64),
    sum_break_after: usize,
    knockout: bool,
    line_gap: Option<(f64, f64)>,
    ends: MeasurementEnds,
}

impl TechnicalMeasurement {
    pub const fn points(glyph: usize, p0: (f64, f64), p1: (f64, f64), value: i64) -> Self {
        Self::new(glyph, p0, p1, value, MeasurementEnds::Points)
    }

    pub const fn edges(glyph: usize, p0: (f64, f64), p1: (f64, f64), value: i64) -> Self {
        Self::new(glyph, p0, p1, value, MeasurementEnds::Edges)
    }

    const fn new(
        glyph: usize,
        p0: (f64, f64),
        p1: (f64, f64),
        value: i64,
        ends: MeasurementEnds,
    ) -> Self {
        Self {
            glyph,
            p0,
            p1,
            value,
            label_shift: (0.0, 0.0),
            sum_break_after: 0,
            knockout: false,
            line_gap: None,
            ends,
        }
    }

    pub const fn counter(mut self) -> Self {
        self.knockout = true;
        self
    }

    pub const fn break_sum_after(mut self, after: usize) -> Self {
        self.sum_break_after = after;
        self
    }

    pub const fn gap_line(mut self, center: f64, half_height: f64) -> Self {
        self.line_gap = Some((center, half_height));
        self
    }

    pub const fn shift_label(mut self, dx: f64, dy: f64) -> Self {
        self.label_shift = (dx, dy);
        self
    }

    pub const fn end_inset(mut self, value: f64) -> Self {
        self.ends = MeasurementEnds::Custom(value);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn section_three_frame_is_centered_and_inside_margin() {
        let style = TechnicalStyle::section_three();
        let frame = style.frame(1152.0, -16.0, 592.0);
        assert!((frame.x(0.0) + frame.x(1152.0) - W).abs() < 0.001);
        assert!(frame.x(0.0) >= MARGIN - 0.001);
        assert!(frame.x(1152.0) <= W - MARGIN + 0.001);
        assert!(frame.y(-16.0) >= MARGIN - 0.001);
        assert!(frame.y(592.0) <= H - MARGIN + 0.001);
    }

    #[test]
    fn section_three_primitives_are_stable() {
        let style = TechnicalStyle::section_three();
        assert_eq!(style.stroke, line::HERO);
        assert_eq!(style.grid_stroke, line::FINE);
        assert_eq!(style.grid_unit, 8.0);
        assert_eq!(style.point_size, 20.0);
        assert_eq!(style.measurement_text_size, 32.0);
        assert_eq!(style.measurement_text_weight, 600.0);
        assert_eq!(style.point_palette, TechnicalPointPalette::Neutral);
        assert_eq!(style.grid_tone, TechnicalGridTone::Faint);
    }

    #[test]
    fn named_measurement_modifiers_preserve_content() {
        let measurement = TechnicalMeasurement::points(2, (8.0, 16.0), (104.0, 16.0), 96)
            .counter()
            .break_sum_after(2)
            .gap_line(48.0, 12.0)
            .shift_label(3.0, -4.0)
            .end_inset(18.0);
        assert_eq!(measurement.glyph, 2);
        assert_eq!(measurement.value, 96);
        assert!(measurement.knockout);
        assert_eq!(measurement.sum_break_after, 2);
        assert_eq!(measurement.line_gap, Some((48.0, 12.0)));
        assert_eq!(measurement.label_shift, (3.0, -4.0));
        assert!(matches!(measurement.ends, MeasurementEnds::Custom(18.0)));
    }

    #[test]
    fn named_grid_modifier_changes_only_grid_primitives() {
        let base = TechnicalStyle::section_three();
        let changed = base.with_grid(64.0, line::HERO).with_subtle_grid();
        assert_eq!(changed.grid_unit, 64.0);
        assert_eq!(changed.grid_stroke, line::HERO);
        assert_eq!(changed.stroke, base.stroke);
        assert_eq!(changed.point_size, base.point_size);
        assert_eq!(changed.point_palette, base.point_palette);
        assert_eq!(changed.grid_tone, TechnicalGridTone::Subtle);
    }
}
