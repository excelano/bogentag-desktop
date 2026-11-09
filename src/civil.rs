// Author: David M. Anderson
// Built with AI assistance (Claude, Anthropic)

use chrono::{Local, NaiveDate, Offset};

use crate::time::utc;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Clock {
    TwelveHour,
    TwentyFourHour,
}

impl Clock {
    /// The local clock to the minute that has begun.
    pub fn time(self, at: bz::Instant) -> String {
        let local = utc(at).with_timezone(&Local);
        match self {
            Clock::TwelveHour => local.format("%-I:%M %p").to_string(),
            Clock::TwentyFourHour => local.format("%H:%M").to_string(),
        }
    }

    /// `11:08 AM CST | UTC−6`, or the offset alone where the zone has no
    /// abbreviation of its own.
    pub fn zoned(self, at: bz::Instant) -> String {
        let time = self.time(at);
        let offset = utc_offset(
            utc(at)
                .with_timezone(&Local)
                .offset()
                .fix()
                .local_minus_utc(),
        );
        match zone_abbreviation(at) {
            Some(name) => format!("{time} {name} | {offset}"),
            None => format!("{time} {offset}"),
        }
    }

    /// `RISE 6:34 AM | SET 5:35 PM`, or what a polar day has instead.
    pub fn horizon(self, day: &bz::SolarDay) -> String {
        match (day.sunrise(), day.sunset()) {
            (Some(rise), Some(set)) => {
                format!("RISE {} | SET {}", self.time(rise), self.time(set))
            }
            _ if day.polar() == Some(bz::Polar::AlwaysAbove) => "SUN UP ALL DAY".to_string(),
            _ => "SUN DOWN ALL DAY".to_string(),
        }
    }
}

fn zone_abbreviation(at: bz::Instant) -> Option<String> {
    let zone: chrono_tz::Tz = iana_time_zone::get_timezone().ok()?.parse().ok()?;
    let name = utc(at).with_timezone(&zone).format("%Z").to_string();
    (!name.starts_with(['+', '-'])).then_some(name)
}

/// `UTC−6`, `UTC+5:30`, `UTC`, with a true minus sign.
pub fn utc_offset(seconds: i32) -> String {
    if seconds == 0 {
        return "UTC".to_string();
    }
    let sign = if seconds < 0 { '\u{2212}' } else { '+' };
    let hours = seconds.abs() / 3600;
    let minutes = seconds.abs() % 3600 / 60;
    if minutes == 0 {
        format!("UTC{sign}{hours}")
    } else {
        format!("UTC{sign}{hours}:{minutes:02}")
    }
}

/// The UTC clock, always 24-hour.
pub fn utc_clock(at: bz::Instant) -> String {
    utc(at).format("%H:%M").to_string()
}

/// The civil date where the computer is, `SUN, 1 NOV 2026`.
pub fn civil_date(at: bz::Instant) -> String {
    utc(at)
        .with_timezone(&Local)
        .format("%a, %-d %b %Y")
        .to_string()
        .to_uppercase()
}

/// A Bogenzeit date, `SUN 1 NOV`.
pub fn bz_day(date: bz::Date) -> String {
    NaiveDate::from_ymd_opt(date.year, date.month, date.day)
        .map(|date| date.format("%a %-d %b").to_string().to_uppercase())
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn offsets() {
        assert_eq!(utc_offset(0), "UTC");
        assert_eq!(utc_offset(-6 * 3600), "UTC\u{2212}6");
        assert_eq!(utc_offset(5 * 3600 + 1800), "UTC+5:30");
    }

    #[test]
    fn bogenzeit_dates() {
        assert_eq!(bz_day(bz::Date::new(2026, 11, 1).unwrap()), "SUN 1 NOV");
    }
}
