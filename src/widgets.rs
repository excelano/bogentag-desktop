// Author: David M. Anderson
// Built with AI assistance (Claude, Anthropic)

use eframe::egui::{
    self, Button, CornerRadius, Label, Response, RichText, Sense, TextWrapMode, Ui, vec2,
};

use crate::display::Position;
use crate::glyph::{self, Mark};
use crate::palette::{self, with_opacity};

const BUTTON_WIDTH: f32 = 88.0;
const GAP: f32 = 6.0;

/// A capsule: the Sun's colour with dark text when chosen, dim otherwise.
pub fn choice(ui: &mut Ui, text: &str, selected: bool, enabled: bool) -> Response {
    choice_sized(ui, text, selected, enabled, BUTTON_WIDTH)
}

pub fn choice_sized(
    ui: &mut Ui,
    text: &str,
    selected: bool,
    enabled: bool,
    width: f32,
) -> Response {
    let (fill, color) = if selected {
        (palette::SUN, palette::NIGHT)
    } else {
        (with_opacity(palette::LINE, 0.12), palette::LINE)
    };
    ui.add_enabled(
        enabled,
        Button::new(RichText::new(text).size(13.0).color(color))
            .fill(fill)
            .corner_radius(CornerRadius::same(255))
            .min_size(vec2(width, 30.0)),
    )
}

/// Buttons side by side, centred in the width there is and narrowed to fit
/// it; `add` is given the width each button should take.
pub fn centred_row(ui: &mut Ui, count: usize, add: impl FnOnce(&mut Ui, f32)) {
    let gaps = count.saturating_sub(1) as f32 * GAP;
    let button = BUTTON_WIDTH.min((ui.available_width() - gaps) / count as f32);
    let width = count as f32 * button + gaps;
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = GAP;
        ui.add_space(((ui.available_width() - width) / 2.0).max(0.0));
        add(ui, button);
    });
}

/// The small spaced capitals over each figure.
pub fn caption(ui: &mut Ui, text: &str) -> Response {
    ui.label(
        RichText::new(text)
            .size(11.0)
            .extra_letter_spacing(1.0)
            .color(palette::LABEL),
    )
}

pub fn numerals(
    ui: &mut Ui,
    number: &str,
    unit: &str,
    size: f32,
    color: egui::Color32,
) -> Response {
    ui.add(
        Label::new(glyph::numerals(number, Some(unit), size, color))
            .wrap_mode(TextWrapMode::Extend)
            .selectable(false),
    )
}

/// A position set in a line of text, its glyph in proportion to the digits.
pub fn inline_mark(size: f32, color: egui::Color32) -> Mark {
    Mark {
        size,
        glyph_radius: size * 0.35,
        line_width: size * 0.08,
        spacing: size * 0.25,
        color,
    }
}

/// A position with its drawn glyph, as a widget.
pub fn mark(ui: &mut Ui, position: &Position, size: f32, color: egui::Color32) -> Response {
    let mark = inline_mark(size, color);
    let width = mark.width(ui.painter(), position);
    let (rect, response) = ui.allocate_exact_size(vec2(width, size * 1.3), Sense::hover());
    mark.paint(ui.painter(), position, rect.center(), width);
    response
}

/// A drawn glyph the size of a line of text.
pub fn glyph(
    ui: &mut Ui,
    glyph: crate::display::Glyph,
    radius: f32,
    color: egui::Color32,
) -> Response {
    let (rect, response) =
        ui.allocate_exact_size(vec2(2.0 * radius + 4.0, 2.0 * radius + 4.0), Sense::hover());
    glyph::paint(
        ui.painter(),
        glyph,
        rect.center(),
        radius,
        radius * 0.24,
        color,
    );
    response
}
