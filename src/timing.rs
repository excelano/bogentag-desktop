// Author: David M. Anderson
// Built with AI assistance (Claude, Anthropic)

use serde::{Deserialize, Serialize};

use crate::dial::DialModel;
use crate::display::{Precision, Scale};
use crate::settings::{self, Place};

const FILE: &str = "timing.json";

/// Time passed, in seconds and, where no polar day lies between, in spans.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Elapsed {
    pub seconds: f64,
    pub spans: Option<f64>,
}

impl Elapsed {
    pub const ZERO: Elapsed = Elapsed {
        seconds: 0.0,
        spans: Some(0.0),
    };

    pub fn between(from: bz::Instant, to: bz::Instant, place: Place) -> Elapsed {
        let duration = bz::SolarDuration::between(from, to, place.latitude, place.longitude);
        Elapsed {
            seconds: duration.seconds,
            spans: duration.spans,
        }
    }

    fn plus(self, other: Elapsed) -> Elapsed {
        Elapsed {
            seconds: self.seconds + other.seconds,
            spans: self.spans.zip(other.spans).map(|(a, b)| a + b),
        }
    }

    pub fn minus(self, other: Elapsed) -> Elapsed {
        Elapsed {
            seconds: self.seconds - other.seconds,
            spans: self.spans.zip(other.spans).map(|(a, b)| a - b),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Unit {
    Spans,
    Kiloseconds,
}

impl Unit {
    pub fn of(scale: Scale) -> Unit {
        match scale {
            Scale::Spans => Unit::Spans,
            Scale::Kiloseconds => Unit::Kiloseconds,
        }
    }

    pub fn scale(self) -> Scale {
        match self {
            Unit::Spans => Scale::Spans,
            Unit::Kiloseconds => Scale::Kiloseconds,
        }
    }

    /// Spans, or seconds for kiloseconds.
    pub fn amount(self, elapsed: Elapsed) -> f64 {
        match self {
            Unit::Spans => elapsed.spans.unwrap_or(0.0),
            Unit::Kiloseconds => elapsed.seconds,
        }
    }
}

/// A countdown in spans or kiloseconds. A span timer ends when the clock has
/// moved that many spans on, fixed at start for the place in use then; a
/// paused one keeps the spans it has left, not the seconds.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Timer {
    pub unit: Unit,
    pub amount: f64,
    pub place: Place,
    pub end: Option<f64>,
    pub left: f64,
}

impl Timer {
    pub fn end_after(
        amount: f64,
        unit: Unit,
        now: bz::Instant,
        place: Place,
    ) -> Option<bz::Instant> {
        match unit {
            Unit::Spans => {
                bz::Reading::at(now, place.latitude, place.longitude).after_spans(amount)
            }
            Unit::Kiloseconds => Some(now + amount),
        }
    }

    pub fn end(&self) -> Option<bz::Instant> {
        self.end.map(bz::Instant)
    }

    pub fn remaining(&self, now: bz::Instant) -> f64 {
        let Some(end) = self.end() else {
            return self.left;
        };
        if now >= end {
            return 0.0;
        }
        match self.unit {
            Unit::Spans => Elapsed::between(now, end, self.place)
                .spans
                .unwrap_or(0.0)
                .max(0.0),
            Unit::Kiloseconds => end - now,
        }
    }

    pub fn is_done(&self, now: bz::Instant) -> bool {
        self.end().is_some_and(|end| now >= end)
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Stopwatch {
    pub place: Place,
    pub since: Option<f64>,
    pub banked: Elapsed,
    pub laps: Vec<Elapsed>,
}

impl Stopwatch {
    pub fn elapsed(&self, now: bz::Instant) -> Elapsed {
        match self.since {
            Some(since) => self
                .banked
                .plus(Elapsed::between(bz::Instant(since), now, self.place)),
            None => self.banked,
        }
    }

    pub fn running(&self) -> bool {
        self.since.is_some()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ArcKind {
    Timer,
    Stopwatch,
}

/// A stretch of the Sun's track, in degrees from the Sun.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Arc {
    pub kind: ArcKind,
    pub from: f64,
    pub to: f64,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Timing {
    pub timer: Option<Timer>,
    pub stopwatch: Option<Stopwatch>,
}

impl Timing {
    pub fn load() -> Timing {
        settings::load(FILE).unwrap_or_default()
    }

    pub fn save(&self) {
        settings::save(FILE, self);
    }

    pub fn start_timer(&mut self, amount: f64, unit: Unit, now: bz::Instant, place: Place) {
        if let Some(end) = Timer::end_after(amount, unit, now, place) {
            self.timer = Some(Timer {
                unit,
                amount,
                place,
                end: Some(end.0),
                left: amount,
            });
        }
    }

    pub fn pause_timer(&mut self, now: bz::Instant) {
        if let Some(timer) = &mut self.timer
            && timer.end.is_some()
        {
            timer.left = timer.remaining(now);
            timer.end = None;
        }
    }

    pub fn resume_timer(&mut self, now: bz::Instant, place: Place) {
        if let Some(timer) = &mut self.timer
            && timer.end.is_none()
            && let Some(end) = Timer::end_after(timer.left, timer.unit, now, place)
        {
            timer.place = place;
            timer.end = Some(end.0);
        }
    }

    pub fn run_stopwatch(&mut self, now: bz::Instant, place: Place) {
        let stopwatch = self.stopwatch.get_or_insert(Stopwatch {
            place,
            since: None,
            banked: Elapsed::ZERO,
            laps: Vec::new(),
        });
        if stopwatch.since.is_none() {
            stopwatch.since = Some(now.0);
        }
    }

    pub fn stop_stopwatch(&mut self, now: bz::Instant) {
        if let Some(stopwatch) = &mut self.stopwatch
            && stopwatch.running()
        {
            stopwatch.banked = stopwatch.elapsed(now);
            stopwatch.since = None;
        }
    }

    pub fn lap(&mut self, now: bz::Instant) {
        if let Some(stopwatch) = &mut self.stopwatch {
            let total = stopwatch.elapsed(now);
            stopwatch.laps.push(total);
        }
    }

    /// A running timer's arc from the Sun to where it ends, and a running
    /// stopwatch's behind the Sun.
    pub fn arcs(&self, model: &DialModel, now: bz::Instant) -> Vec<Arc> {
        let mut arcs = Vec::new();
        if let Some(timer) = &self.timer
            && let Some(end) = timer.end()
            && now < end
        {
            let left = Elapsed::between(now, end, timer.place);
            arcs.push(Arc {
                kind: ArcKind::Timer,
                from: 0.0,
                to: model.sweep(left.seconds, left.spans),
            });
        }
        if let Some(stopwatch) = &self.stopwatch
            && stopwatch.running()
        {
            let elapsed = stopwatch.elapsed(now);
            arcs.push(Arc {
                kind: ArcKind::Stopwatch,
                from: -model.sweep(elapsed.seconds, elapsed.spans),
                to: 0.0,
            });
        }
        arcs
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Rounding {
    Down,
    Nearest,
    Up,
}

/// An amount in spans, or in seconds shown as kiloseconds, at a precision.
/// Remaining amounts round up, so the 0 shows only at the end; elapsed
/// amounts are floored like every readout.
pub fn number(amount: f64, unit: Unit, precision: Precision, rounding: Rounding) -> String {
    let value = match unit {
        Unit::Spans => amount,
        Unit::Kiloseconds => amount / 1000.0,
    };
    let decimals = precision.decimals(unit.scale());
    let factor = 10f64.powi(decimals as i32);
    let scaled = value * factor;
    let steps = match rounding {
        Rounding::Down => (scaled + 1e-6).floor(),
        Rounding::Nearest => scaled.round(),
        Rounding::Up => (scaled - 1e-6).ceil(),
    }
    .max(0.0);
    crate::display::decimal(steps as i64, decimals)
}

pub fn unit_name(number: &str, unit: Unit) -> &'static str {
    match unit {
        Unit::Kiloseconds => "ks",
        Unit::Spans if number == "1" => "span",
        Unit::Spans => "spans",
    }
}

/// The amount as typed or preset, with trailing zeros dropped: `2.5`.
pub fn trimmed(amount: f64, unit: Unit) -> String {
    let shown = number(amount, unit, Precision::High, Rounding::Nearest);
    if shown.contains('.') {
        shown
            .trim_end_matches('0')
            .trim_end_matches('.')
            .to_string()
    } else {
        shown
    }
}

/// `2.5 spans`, `0.5 ks`.
pub fn set_text(amount: f64, unit: Unit) -> String {
    let shown = trimmed(amount, unit);
    let name = unit_name(&shown, unit);
    format!("{shown} {name}")
}

#[cfg(test)]
mod tests {
    use super::*;

    const HOUSTON: Place = Place {
        latitude: 29.76,
        longitude: -95.37,
    };

    fn at(text: &str) -> bz::Instant {
        bz::Instant::parse_rfc3339(text).unwrap()
    }

    #[test]
    fn a_span_timer_takes_the_spans_it_was_set_for() {
        let now = at("2026-11-01T17:08:00Z");
        let mut timing = Timing::default();
        timing.start_timer(20.0, Unit::Spans, now, HOUSTON);
        let timer = timing.timer.unwrap();
        let left = Elapsed::between(now, timer.end().unwrap(), HOUSTON);
        assert!((left.spans.unwrap() - 20.0).abs() < 1e-6);
    }

    #[test]
    fn a_paused_timer_keeps_its_spans() {
        let now = at("2026-11-01T17:08:00Z");
        let mut timing = Timing::default();
        timing.start_timer(10.0, Unit::Spans, now, HOUSTON);
        timing.pause_timer(now + 600.0);
        let paused = timing.timer.unwrap();
        assert_eq!(paused.remaining(now + 99_999.0), paused.left);
        assert!(paused.left > 0.0 && paused.left < 10.0);
    }

    #[test]
    fn remaining_rounds_up_and_elapsed_down() {
        assert_eq!(
            number(0.001, Unit::Spans, Precision::Low, Rounding::Up),
            "1"
        );
        assert_eq!(
            number(1.999, Unit::Spans, Precision::Low, Rounding::Down),
            "1"
        );
        assert_eq!(
            number(1500.0, Unit::Kiloseconds, Precision::Medium, Rounding::Down),
            "1.50"
        );
    }

    #[test]
    fn set_amounts_drop_trailing_zeros() {
        assert_eq!(set_text(1.0, Unit::Spans), "1 span");
        assert_eq!(set_text(2.5, Unit::Spans), "2.5 spans");
        assert_eq!(set_text(500.0, Unit::Kiloseconds), "0.5 ks");
    }

    #[test]
    fn the_stopwatch_banks_what_it_counted() {
        let now = at("2026-11-01T17:08:00Z");
        let mut timing = Timing::default();
        timing.run_stopwatch(now, HOUSTON);
        timing.stop_stopwatch(now + 1000.0);
        let stopwatch = timing.stopwatch.unwrap();
        assert_eq!(stopwatch.elapsed(now + 5000.0).seconds, 1000.0);
    }
}
