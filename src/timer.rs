// Author: David M. Anderson
// Built with AI assistance (Claude, Anthropic)

use eframe::egui::{self, Align, Key, Layout, RichText, Ui, vec2};

use crate::display::{Position, Precision};
use crate::palette;
use crate::settings::{Place, Settings};
use crate::timing::{self, Rounding, Timer, Timing, Unit};
use crate::widgets;

/// The timer reads at high precision whatever the dial's setting.
const PRECISION: Precision = Precision::High;
const ENDS_AT_SIZE: f32 = 18.0;

#[derive(Default)]
pub struct TimerPanel {
    draft: Option<String>,
}

impl TimerPanel {
    pub fn show(
        &mut self,
        ui: &mut Ui,
        timing: &mut Timing,
        settings: &mut Settings,
        now: bz::Instant,
        place: Place,
    ) {
        ui.vertical_centered(|ui| match timing.timer {
            Some(timer) => running(ui, timing, timer, now, place),
            None => self.setter(ui, timing, settings, now, place),
        });
    }

    fn setter(
        &mut self,
        ui: &mut Ui,
        timing: &mut Timing,
        settings: &mut Settings,
        now: bz::Instant,
        place: Place,
    ) {
        let polar = bz::Reading::at(now, place.latitude, place.longitude)
            .polar()
            .is_some();
        let unit = if polar {
            Unit::Kiloseconds
        } else {
            Unit::of(settings.scale)
        };
        let amount = match unit {
            Unit::Spans => &mut settings.timer_spans,
            Unit::Kiloseconds => &mut settings.timer_seconds,
        };
        widgets::caption(ui, "TIMER");
        if polar {
            widgets::caption(ui, "NO SPANS ON A POLAR DAY");
        }
        ui.add_space(4.0);
        self.amount(ui, amount, unit);
        ui.add_space(6.0);
        let presets: [f64; 4] = match unit {
            Unit::Spans => [1.0, 2.0, 5.0, 10.0],
            Unit::Kiloseconds => [200.0, 500.0, 1000.0, 2000.0],
        };
        widgets::centred_row(ui, presets.len(), |ui, width| {
            for preset in presets {
                let chosen =
                    self.draft.is_none() && (*amount - preset).abs() < resolution(unit) / 2.0;
                if widgets::choice_sized(ui, &timing::trimmed(preset, unit), chosen, true, width)
                    .clicked()
                {
                    self.draft = None;
                    *amount = preset;
                }
            }
        });
        ui.add_space(10.0);
        let amount = *amount;
        let can_start = Timer::end_after(amount, unit, now, place).is_some();
        widgets::centred_row(ui, 1, |ui, width| {
            if widgets::choice_sized(ui, "Start", true, can_start, width).clicked() {
                self.draft = None;
                timing.start_timer(amount, unit, now, place);
            }
        });
    }

    /// − and + either side of the amount; a click on the amount types one in.
    fn amount(&mut self, ui: &mut Ui, amount: &mut f64, unit: Unit) {
        let step = match unit {
            Unit::Spans => 0.1,
            Unit::Kiloseconds => 100.0,
        };
        ui.allocate_ui_with_layout(
            vec2(ui.available_width(), 72.0),
            Layout::left_to_right(Align::Center),
            |ui| {
                let nudge = |ui: &mut Ui,
                             text: &str,
                             sign: f64,
                             amount: &mut f64,
                             draft: &mut Option<String>| {
                    if ui
                        .add(
                            egui::Button::new(RichText::new(text).size(20.0))
                                .min_size(vec2(36.0, 36.0)),
                        )
                        .clicked()
                    {
                        *draft = None;
                        *amount = clamped(*amount + sign * step, unit);
                    }
                };
                nudge(ui, "\u{2212}", -1.0, amount, &mut self.draft);
                let middle = ui.available_width() - 44.0;
                ui.allocate_ui_with_layout(
                    vec2(middle, 72.0),
                    Layout::centered_and_justified(egui::Direction::LeftToRight),
                    |ui| match &mut self.draft {
                        Some(draft) => {
                            let field = ui.add(
                                egui::TextEdit::singleline(draft)
                                    .font(egui::FontId::monospace(36.0))
                                    .horizontal_align(Align::Center)
                                    .desired_width(middle - 20.0),
                            );
                            if !field.has_focus() && !field.lost_focus() {
                                field.request_focus();
                            }
                            if field.changed()
                                && let Ok(typed) = draft.trim().parse::<f64>()
                            {
                                *amount = clamped(
                                    match unit {
                                        Unit::Spans => typed,
                                        Unit::Kiloseconds => typed * 1000.0,
                                    },
                                    unit,
                                );
                            }
                            if field.lost_focus() || ui.input(|i| i.key_pressed(Key::Enter)) {
                                self.draft = None;
                            }
                        }
                        None => {
                            let shown = timing::number(*amount, unit, PRECISION, Rounding::Nearest);
                            let response = widgets::numerals(
                                ui,
                                &shown,
                                timing::unit_name(&shown, unit),
                                48.0,
                                palette::LINE,
                            )
                            .interact(egui::Sense::click())
                            .on_hover_cursor(egui::CursorIcon::Text)
                            .on_hover_text("Type an amount");
                            if response.clicked() {
                                self.draft = Some(timing::trimmed(*amount, unit));
                            }
                        }
                    },
                );
                nudge(ui, "+", 1.0, amount, &mut self.draft);
            },
        );
    }
}

fn running(ui: &mut Ui, timing: &mut Timing, timer: Timer, now: bz::Instant, place: Place) {
    let done = timer.is_done(now);
    let paused = timer.end.is_none();
    widgets::caption(
        ui,
        if paused {
            "PAUSED"
        } else if done {
            "DONE"
        } else {
            "TIMER"
        },
    );
    let shown = timing::number(timer.remaining(now), timer.unit, PRECISION, Rounding::Up);
    widgets::numerals(
        ui,
        &shown,
        timing::unit_name(&shown, timer.unit),
        48.0,
        if done { palette::SUN } else { palette::LINE },
    );
    if let Some(end) = timer.end()
        && !done
    {
        let reading = bz::Reading::at(end, timer.place.latitude, timer.place.longitude);
        let position = Position::of(&reading, timer.unit.scale(), PRECISION);
        ui.horizontal(|ui| {
            let width = 70.0
                + widgets::inline_mark(ENDS_AT_SIZE, palette::SECONDARY)
                    .width(ui.painter(), &position);
            ui.add_space(((ui.available_width() - width) / 2.0).max(0.0));
            widgets::caption(ui, "ENDS AT");
            widgets::mark(ui, &position, ENDS_AT_SIZE, palette::SECONDARY);
        });
    }
    ui.add_space(10.0);
    widgets::centred_row(ui, 2, |ui, width| {
        if done {
            let again = Timer::end_after(timer.amount, timer.unit, now, place).is_some();
            if widgets::choice_sized(ui, "Again", true, again, width).clicked() {
                timing.start_timer(timer.amount, timer.unit, now, place);
            }
            if widgets::choice_sized(ui, "Clear", false, true, width).clicked() {
                timing.timer = None;
            }
        } else if paused {
            let resume = Timer::end_after(timer.left, timer.unit, now, place).is_some();
            if widgets::choice_sized(ui, "Resume", true, resume, width).clicked() {
                timing.resume_timer(now, place);
            }
            if widgets::choice_sized(ui, "Cancel", false, true, width).clicked() {
                timing.timer = None;
            }
        } else {
            if widgets::choice_sized(ui, "Pause", true, true, width).clicked() {
                timing.pause_timer(now);
            }
            if widgets::choice_sized(ui, "Cancel", false, true, width).clicked() {
                timing.timer = None;
            }
        }
    });
}

fn resolution(unit: Unit) -> f64 {
    match unit {
        Unit::Spans => 0.01,
        Unit::Kiloseconds => 1.0,
    }
}

/// To the resolution a timer is set in, between one step and a day.
fn clamped(value: f64, unit: Unit) -> f64 {
    let most = match unit {
        Unit::Spans => 400.0,
        Unit::Kiloseconds => 86_400.0,
    };
    let step = resolution(unit);
    ((value / step).round() * step).clamp(step, most)
}
