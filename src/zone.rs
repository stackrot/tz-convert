use chrono::{DateTime, Local, NaiveDateTime, Utc};
use chrono_tz::{Tz, TZ_VARIANTS};
use std::fmt;
use std::str::FromStr;

use crate::time;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Zone {
    Local,
    Named(Tz),
}

impl Zone {
    pub fn resolve(self, time: &NaiveDateTime) -> Result<DateTime<Utc>, String> {
        match self {
            Self::Local => time::resolve(&Local, time),
            Self::Named(zone) => time::resolve(&zone, time),
        }
    }

    pub fn format(self, time: DateTime<Utc>, format: &str) -> String {
        match self {
            Self::Local => time.with_timezone(&Local).format(format).to_string(),
            Self::Named(zone) => time.with_timezone(&zone).format(format).to_string(),
        }
    }
}

impl fmt::Display for Zone {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Local => formatter.write_str("Local"),
            Self::Named(zone) => formatter.write_str(zone.name()),
        }
    }
}

impl FromStr for Zone {
    type Err = String;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        if value.eq_ignore_ascii_case("local") {
            return Ok(Self::Local);
        }
        value.parse().map(Self::Named).map_err(|_| {
            format!("Unknown timezone '{value}'. Use --list-timezones to see valid names.")
        })
    }
}

pub fn all() -> Vec<Zone> {
    let mut zones = TZ_VARIANTS.to_vec();
    zones.sort_by_key(|zone| zone.name());
    zones.into_iter().map(Zone::Named).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_timezone_can_be_used_as_a_source() {
        let time = time::parse("2026-01-15 12:00").unwrap();
        for zone in all() {
            let parsed: Zone = zone.to_string().parse().unwrap();
            let instant = parsed.resolve(&time).unwrap();
            assert_eq!(parsed.format(instant, "%Y-%m-%d %H:%M"), "2026-01-15 12:00");
        }
    }
}
