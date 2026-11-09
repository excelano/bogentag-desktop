// Author: David M. Anderson
// Built with AI assistance (Claude, Anthropic)

use eframe::egui::{Align, Layout, RichText, Ui};

use crate::civil::{self, Clock};
use crate::dial::{kiloseconds, nachtbogen};
use crate::display::Glyph;
use crate::palette;
use crate::widgets;

/// Today in numbers: the two arcs and their span lengths, and the four
/// events of the day with the civil clock's time for each.
pub fn show(ui: &mut Ui, reading: &bz::Reading, clock: Clock) {
    let day = &reading.day;
    ui.label(
        RichText::new(civil::bz_day(day.date))
            .size(18.0)
            .strong()
            .color(palette::LINE),
    );
    ui.add_space(8.0);
    if let (Some(length), Some(span)) = (day.tagbogen_s(), day.day_span_s()) {
        arc(ui, "DAY ARC", length, span);
    }
    if let (Some(length), Some(half)) = (nachtbogen(reading), day.half_night_evening_s()) {
        arc(ui, "NIGHT ARC", length, half / 100.0);
    }
    match day.polar() {
        Some(bz::Polar::AlwaysAbove) => {
            widgets::caption(ui, "POLAR DAY");
        }
        Some(bz::Polar::AlwaysBelow) => {
            widgets::caption(ui, "POLAR NIGHT");
        }
        None => {}
    }
    ui.add_space(6.0);
    ui.separator();
    ui.add_space(6.0);
    event(ui, "Sunrise", Glyph::Sunrise, day.sunrise(), clock);
    event(ui, "Solar noon", Glyph::Noon, Some(day.noon), clock);
    event(ui, "Sunset", Glyph::Sunset, day.sunset(), clock);
    event(
        ui,
        "Solar midnight",
        Glyph::Midnight,
        Some(day.midnight_end),
        clock,
    );
}

fn arc(ui: &mut Ui, name: &str, length: f64, span: f64) {
    widgets::caption(ui, name);
    ui.horizontal(|ui| {
        widgets::numerals(ui, &kiloseconds(length), "ks", 22.0, palette::LINE);
        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            ui.label(
                RichText::new(format!("span {} s", span.floor()))
                    .size(14.0)
                    .color(palette::SECONDARY),
            );
        });
    });
    ui.add_space(6.0);
}

fn event(ui: &mut Ui, name: &str, glyph: Glyph, at: Option<bz::Instant>, clock: Clock) {
    ui.horizontal(|ui| {
        widgets::glyph(ui, glyph, 7.0, palette::LINE);
        ui.label(RichText::new(name).size(15.0).color(palette::LINE));
        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            ui.label(
                RichText::new(at.map_or("none".to_string(), |at| clock.time(at)))
                    .size(15.0)
                    .monospace()
                    .color(palette::SECONDARY),
            );
        });
    });
    ui.add_space(4.0);
}
