// Author: David M. Anderson
// Built with AI assistance (Claude, Anthropic)

use eframe::egui::{Align, Layout, Response, RichText, Ui, vec2};

use crate::civil::{self, Clock};
use crate::dial::{kiloseconds, nachtbogen};
use crate::display::Glyph;
use crate::palette;
use crate::widgets;

const SPAN_COUNTS: [u32; 4] = [1, 5, 10, 50];
const DURATION_WIDTH: f32 = 80.0;

/// Today in numbers: the two arcs and their span lengths, the four events
/// of the day with the civil clock's time for each, and how long a few
/// spans last by day and by night.
pub fn show(ui: &mut Ui, reading: &bz::Reading, clock: Clock) {
    let day = &reading.day;
    ui.label(
        RichText::new(civil::bz_day(day.date))
            .size(18.0)
            .strong()
            .color(palette::LINE),
    );
    ui.add_space(8.0);
    let day_span = day.day_span_s();
    let night_span = day.half_night_evening_s().map(|half| half / 100.0);
    if let (Some(length), Some(span)) = (day.tagbogen_s(), day_span) {
        arc(ui, "DAY ARC", length, span);
    }
    if let (Some(length), Some(span)) = (nachtbogen(reading), night_span) {
        arc(ui, "NIGHT ARC", length, span);
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
    if day_span.is_some() || night_span.is_some() {
        ui.add_space(6.0);
        ui.separator();
        ui.add_space(6.0);
        durations(ui, day_span, night_span);
    }
}

fn durations(ui: &mut Ui, day_span: Option<f64>, night_span: Option<f64>) {
    ui.horizontal(|ui| {
        widgets::caption(ui, "SPANS IN CLOCK TIME");
        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            cell(ui, |ui| widgets::caption(ui, "NIGHT"));
            cell(ui, |ui| widgets::caption(ui, "DAY"));
        });
    });
    ui.add_space(4.0);
    for count in SPAN_COUNTS {
        let unit = if count == 1 { "span" } else { "spans" };
        ui.horizontal(|ui| {
            ui.label(
                RichText::new(format!("{count} {unit}"))
                    .size(15.0)
                    .color(palette::LINE),
            );
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                cell(ui, |ui| duration(ui, night_span, count));
                cell(ui, |ui| duration(ui, day_span, count));
            });
        });
        ui.add_space(4.0);
    }
}

fn cell(ui: &mut Ui, add: impl FnOnce(&mut Ui) -> Response) {
    ui.allocate_ui_with_layout(
        vec2(DURATION_WIDTH, ui.available_height()),
        Layout::right_to_left(Align::Center),
        |ui| {
            ui.set_min_width(DURATION_WIDTH);
            add(ui)
        },
    );
}

fn duration(ui: &mut Ui, span: Option<f64>, count: u32) -> Response {
    let text = span.map_or("none".to_string(), |span| {
        clock_duration(span * f64::from(count))
    });
    ui.label(
        RichText::new(text)
            .size(15.0)
            .monospace()
            .color(palette::SECONDARY),
    )
}

/// Minutes and seconds under an hour, hours and minutes from one up, the
/// seconds floored like every readout.
fn clock_duration(seconds: f64) -> String {
    let seconds = seconds.floor() as u64;
    if seconds < 3600 {
        format!("{}m {:02}s", seconds / 60, seconds % 60)
    } else {
        format!("{}h {:02}m", seconds / 3600, seconds % 3600 / 60)
    }
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

#[cfg(test)]
mod tests {
    use super::clock_duration;

    #[test]
    fn durations_floor_and_switch_to_hours() {
        assert_eq!(clock_duration(191.9), "3m 11s");
        assert_eq!(clock_duration(3599.9), "59m 59s");
        assert_eq!(clock_duration(9575.0), "2h 39m");
    }
}
