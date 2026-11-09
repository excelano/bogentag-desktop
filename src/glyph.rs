// Author: David M. Anderson
// Built with AI assistance (Claude, Anthropic)

use std::sync::Arc;

use eframe::egui::text::{LayoutJob, TextFormat};
use eframe::egui::{
    Align, Color32, FontFamily, FontId, Galley, Painter, Pos2, Shape, Stroke, pos2,
};

use crate::display::{Glyph, Position};

const DOT_RATIO: f32 = 2.6 / 9.0;
const DECIMAL_RATIO: f32 = 0.6;
const UNIT_RATIO: f32 = 0.4;
const SMALLEST: f32 = 9.0;

/// One of the four glyphs, drawn rather than set in a font.
pub fn paint(
    painter: &Painter,
    glyph: Glyph,
    center: Pos2,
    radius: f32,
    width: f32,
    color: Color32,
) {
    let stroke = Stroke::new(width, color);
    match glyph {
        Glyph::Noon | Glyph::Midnight => {
            painter.circle_stroke(center, radius, stroke);
            if glyph == Glyph::Noon {
                painter.circle_filled(center, radius * DOT_RATIO, color);
            }
        }
        Glyph::Sunrise | Glyph::Sunset => {
            let apex = if glyph == Glyph::Sunrise {
                -radius
            } else {
                radius
            };
            let points = vec![
                pos2(center.x, center.y + apex),
                pos2(center.x - radius, center.y - apex),
                pos2(center.x + radius, center.y - apex),
            ];
            painter.add(Shape::closed_line(points, stroke));
        }
    }
}

/// A number as the readouts set it: the whole part at full size, the
/// decimals smaller on the same line, and the unit smaller still.
pub fn numerals(number: &str, unit: Option<&str>, size: f32, color: Color32) -> LayoutJob {
    let format = |size: f32, family: FontFamily| TextFormat {
        font_id: FontId::new(size, family),
        color,
        valign: Align::BOTTOM,
        ..Default::default()
    };
    let point = number.find('.').unwrap_or(number.len());
    let mut job = LayoutJob::default();
    job.append(&number[..point], 0.0, format(size, FontFamily::Monospace));
    if point < number.len() {
        job.append(
            &number[point..],
            0.0,
            format((size * DECIMAL_RATIO).max(SMALLEST), FontFamily::Monospace),
        );
    }
    if let Some(unit) = unit {
        job.append(
            unit,
            (size * 0.15).max(2.0),
            format((size * UNIT_RATIO).max(SMALLEST), FontFamily::Proportional),
        );
    }
    job
}

pub struct Mark {
    pub size: f32,
    pub glyph_radius: f32,
    pub line_width: f32,
    pub spacing: f32,
    pub color: Color32,
}

impl Mark {
    fn measure(
        &self,
        painter: &Painter,
        position: &Position,
        factor: f32,
    ) -> (Option<Arc<Galley>>, f32, f32) {
        let galley = position.number.as_ref().map(|number| {
            painter.layout_job(numerals(
                number,
                position.unit,
                self.size * factor,
                self.color,
            ))
        });
        let glyph = (2.0 * self.glyph_radius + self.line_width) * factor;
        let total = galley.as_ref().map_or(glyph, |galley| {
            galley.size().x + self.spacing * factor + glyph
        });
        (galley, glyph, total)
    }

    pub fn width(&self, painter: &Painter, position: &Position) -> f32 {
        self.measure(painter, position, 1.0).2
    }

    /// A position centred on `center`: the number on the side of the glyph
    /// where the moment lies, shrunk as far as 0.8 to fit `max_width`.
    pub fn paint(&self, painter: &Painter, position: &Position, center: Pos2, max_width: f32) {
        let mut factor = 1.0;
        let (mut galley, mut glyph, mut total) = self.measure(painter, position, factor);
        if total > max_width {
            factor = (max_width / total).max(0.8);
            (galley, glyph, total) = self.measure(painter, position, factor);
        }
        let left = center.x - total / 2.0;
        let glyph_at = |x: f32| {
            paint(
                painter,
                position.glyph,
                pos2(x + glyph / 2.0, center.y),
                self.glyph_radius * factor,
                self.line_width * factor,
                self.color,
            );
        };
        let Some(galley) = galley else {
            glyph_at(left);
            return;
        };
        let top = center.y - galley.size().y / 2.0;
        if position.before {
            glyph_at(left + galley.size().x + self.spacing * factor);
            painter.galley(pos2(left, top), galley, self.color);
        } else {
            glyph_at(left);
            painter.galley(
                pos2(left + glyph + self.spacing * factor, top),
                galley,
                self.color,
            );
        }
    }
}
