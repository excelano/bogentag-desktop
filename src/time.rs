// Author: David M. Anderson
// Built with AI assistance (Claude, Anthropic)

use chrono::{DateTime, Utc};

pub fn utc(instant: bz::Instant) -> DateTime<Utc> {
    DateTime::from_timestamp_micros((instant.0 * 1e6).round() as i64).unwrap_or_default()
}
