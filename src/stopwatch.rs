// Author: David M. Anderson
// Built with AI assistance (Claude, Anthropic)

use eframe::egui::{Align, Layout, RichText, ScrollArea, Ui};

use crate::display::{Precision, Scale};
use crate::palette;
use crate::settings::Place;
use crate::timing::{self, Elapsed, Rounding, Timing, Unit};
use crate::widgets;

const PRECISION: Precision = Precision::High;

/// Counts what passes while it runs, in the dial's unit, or in kiloseconds
/// across a polar day, with laps.
pub fn show(ui: &mut Ui, timing: &mut Timing, scale: Scale, now: bz::Instant, place: Place) {
    let unit = |elapsed: Elapsed| {
        if elapsed.spans.is_none() {
            Unit::Kiloseconds
        } else {
            Unit::of(scale)
        }
    };
    ui.vertical_centered(|ui| {
        let elapsed = timing
            .stopwatch
            .as_ref()
            .map_or(Elapsed::ZERO, |stopwatch| stopwatch.elapsed(now));
        let running = timing.stopwatch.as_ref().is_some_and(|s| s.running());
        widgets::caption(ui, "STOPWATCH");
        let shown = timing::number(
            unit(elapsed).amount(elapsed),
            unit(elapsed),
            PRECISION,
            Rounding::Down,
        );
        widgets::numerals(
            ui,
            &shown,
            timing::unit_name(&shown, unit(elapsed)),
            48.0,
            palette::LINE,
        );
        ui.add_space(10.0);
        widgets::centred_row(ui, 2, |ui, width| {
            if running {
                if widgets::choice_sized(ui, "Stop", true, true, width).clicked() {
                    timing.stop_stopwatch(now);
                }
                if widgets::choice_sized(ui, "Lap", false, true, width).clicked() {
                    timing.lap(now);
                }
            } else {
                if widgets::choice_sized(ui, "Start", true, true, width).clicked() {
                    timing.run_stopwatch(now, place);
                }
                if widgets::choice_sized(ui, "Reset", false, timing.stopwatch.is_some(), width)
                    .clicked()
                {
                    timing.stopwatch = None;
                }
            }
        });
    });
    let laps = timing
        .stopwatch
        .as_ref()
        .map(|stopwatch| stopwatch.laps.clone())
        .unwrap_or_default();
    if laps.is_empty() {
        return;
    }
    ui.add_space(8.0);
    ui.separator();
    ScrollArea::vertical().show(ui, |ui| {
        for (index, total) in laps.iter().enumerate().rev() {
            let split = match index {
                0 => *total,
                _ => total.minus(laps[index - 1]),
            };
            let unit = unit(split);
            let shown = timing::number(unit.amount(split), unit, PRECISION, Rounding::Down);
            ui.horizontal(|ui| {
                ui.label(RichText::new(format!("Lap {}", index + 1)).color(palette::LABEL));
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    ui.label(
                        RichText::new(format!("{shown} {}", timing::unit_name(&shown, unit)))
                            .monospace()
                            .color(palette::LINE),
                    );
                });
            });
        }
    });
}
