// Author: David M. Anderson
// Built with AI assistance (Claude, Anthropic)

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Scale {
    Spans,
    Kiloseconds,
}

impl Scale {
    pub const ALL: [Scale; 2] = [Scale::Spans, Scale::Kiloseconds];

    pub fn title(self) -> &'static str {
        match self {
            Scale::Spans => "Spans",
            Scale::Kiloseconds => "Kiloseconds",
        }
    }

    pub fn parse(text: &str) -> Option<Scale> {
        Scale::ALL
            .into_iter()
            .find(|scale| scale.title().eq_ignore_ascii_case(text))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Precision {
    Low,
    Medium,
    High,
}

impl Precision {
    pub const ALL: [Precision; 3] = [Precision::Low, Precision::Medium, Precision::High];

    pub fn title(self) -> &'static str {
        match self {
            Precision::Low => "Low",
            Precision::Medium => "Medium",
            Precision::High => "High",
        }
    }

    pub fn parse(text: &str) -> Option<Precision> {
        Precision::ALL
            .into_iter()
            .find(|precision| precision.title().eq_ignore_ascii_case(text))
    }

    pub fn decimals(self, scale: Scale) -> u32 {
        let base = match self {
            Precision::Low => 0,
            Precision::Medium => 1,
            Precision::High => 2,
        };
        match scale {
            Scale::Spans => base,
            Scale::Kiloseconds => base + 1,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Glyph {
    Noon,
    Midnight,
    Sunrise,
    Sunset,
}

impl Glyph {
    pub fn of(anchor: bz::Anchor) -> Glyph {
        match anchor {
            bz::Anchor::Noon => Glyph::Noon,
            bz::Anchor::Midnight => Glyph::Midnight,
        }
    }

    fn horizon_before(anchor: bz::Anchor) -> Glyph {
        match anchor {
            bz::Anchor::Noon => Glyph::Sunrise,
            bz::Anchor::Midnight => Glyph::Sunset,
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            Glyph::Noon => "solar noon",
            Glyph::Midnight => "solar midnight",
            Glyph::Sunrise => "sunrise",
            Glyph::Sunset => "sunset",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Position {
    pub glyph: Glyph,
    pub number: Option<String>,
    pub before: bool,
    pub unit: Option<&'static str>,
}

impl Position {
    /// The reading in one scale, floored at the precision. A polar day has
    /// no spans, so it is given in kiloseconds whatever the scale.
    pub fn of(reading: &bz::Reading, scale: Scale, precision: Precision) -> Position {
        if let (Scale::Spans, Some(span)) = (scale, reading.span()) {
            let decimals = precision.decimals(Scale::Spans);
            let factor = 10i64.pow(decimals);
            let steps = (span * factor as f64).floor() as i64;
            let horizon = steps == -100 * factor;
            return Position {
                glyph: if horizon {
                    Glyph::horizon_before(reading.anchor)
                } else {
                    Glyph::of(reading.anchor)
                },
                number: (steps != 0 && !horizon).then(|| decimal(steps.abs(), decimals)),
                before: steps < 0,
                unit: None,
            };
        }
        let decimals = precision.decimals(Scale::Kiloseconds);
        let steps = (reading.offset_s / 10f64.powi(3 - decimals as i32)).floor() as i64;
        Position {
            glyph: Glyph::of(reading.anchor),
            number: (steps != 0).then(|| decimal(steps.abs(), decimals)),
            before: steps < 0,
            unit: Some("ks"),
        }
    }
}

pub fn decimal(steps: i64, decimals: u32) -> String {
    if decimals == 0 {
        return steps.to_string();
    }
    let factor = 10i64.pow(decimals);
    format!(
        "{}.{:0width$}",
        steps / factor,
        steps % factor,
        width = decimals as usize
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn reading(text: &str) -> bz::Reading {
        let at = bz::Instant::parse_rfc3339(text).unwrap();
        bz::Reading::at(at, 29.76, -95.37)
    }

    #[test]
    fn spans_agree_with_the_engine_form() {
        for text in [
            "2026-11-04T16:20:00Z",
            "2026-11-04T23:59:00Z",
            "2026-11-05T09:00:00Z",
            "2026-06-21T18:00:00Z",
        ] {
            let reading = reading(text);
            for (precision, engine) in [
                (Precision::Low, bz::Precision::Low),
                (Precision::High, bz::Precision::High),
            ] {
                let position = Position::of(&reading, Scale::Spans, precision);
                let glyph = match position.glyph {
                    Glyph::Noon => '☉',
                    Glyph::Midnight => '○',
                    Glyph::Sunrise => '△',
                    Glyph::Sunset => '▽',
                };
                let shown = match &position.number {
                    None => glyph.to_string(),
                    Some(number) if position.before => format!("{number} {glyph}"),
                    Some(number) => format!("{glyph} {number}"),
                };
                assert_eq!(
                    Some(shown),
                    reading.form(bz::Form::Span, Some(engine)),
                    "{text}"
                );
            }
        }
    }

    #[test]
    fn kiloseconds_carry_their_unit() {
        let position = Position::of(
            &reading("2026-11-04T16:20:00Z"),
            Scale::Kiloseconds,
            Precision::Medium,
        );
        assert_eq!(position.unit, Some("ks"));
        assert!(position.before);
        assert_eq!(
            position
                .number
                .as_deref()
                .map(|n| n.split('.').nth(1).unwrap().len()),
            Some(2)
        );
    }

    #[test]
    fn a_polar_day_reads_in_kiloseconds() {
        let at = bz::Instant::parse_rfc3339("2026-06-21T12:00:00Z").unwrap();
        let reading = bz::Reading::at(at, 78.22, 15.65);
        let position = Position::of(&reading, Scale::Spans, Precision::Medium);
        assert_eq!(position.unit, Some("ks"));
    }

    #[test]
    fn decimals_pad() {
        assert_eq!(decimal(1205, 1), "120.5");
        assert_eq!(decimal(7, 2), "0.07");
        assert_eq!(decimal(42, 0), "42");
    }
}
