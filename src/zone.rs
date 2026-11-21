use jiff::Timestamp;
use jiff::civil::DateTime;
use jiff::tz::{self, TimeZone};
use std::fmt;
use std::str::FromStr;

use crate::time;

#[derive(Clone, Debug)]
pub enum Zone {
    Local,
    Named(TimeZone),
}

impl Zone {
    pub fn resolve(&self, time: DateTime) -> Result<Timestamp, String> {
        time::resolve(&self.timezone(), time)
    }

    pub fn format(&self, time: Timestamp, format: &str) -> String {
        time.to_zoned_datetime(self.timezone())
            .strftime(format)
            .to_string()
    }

    fn timezone(&self) -> TimeZone {
        match self {
            Self::Local => TimeZone::system(),
            Self::Named(zone) => zone.clone(),
        }
    }

    fn name(&self) -> &str {
        match self {
            Self::Local => "Local",
            Self::Named(zone) => zone.iana_name().unwrap_or_default(),
        }
    }
}

// jiff treats aliases such as US/Eastern and America/New_York as equal, but
// each name the user asks for should still get its own row.
impl PartialEq for Zone {
    fn eq(&self, other: &Self) -> bool {
        self.name() == other.name()
    }
}

impl Eq for Zone {}

impl fmt::Display for Zone {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.name())
    }
}

impl FromStr for Zone {
    type Err = String;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        if value.eq_ignore_ascii_case("local") {
            return Ok(Self::Local);
        }
        tz::db().get(value).map(Self::Named).map_err(|_| {
            format!("Unknown timezone '{value}'. Use --list-timezones to see valid names.")
        })
    }
}

pub fn all() -> Vec<Zone> {
    let db = tz::db();
    let mut names: Vec<_> = db.available().map(|name| name.to_string()).collect();
    names.sort_unstable();
    names
        .iter()
        .filter_map(|name| db.get(name).ok())
        .map(Zone::Named)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_timezone_can_be_used_as_a_source() {
        let time = time::parse("2026-01-15 12:00").unwrap();
        for zone in all() {
            let parsed: Zone = zone.to_string().parse().unwrap();
            let instant = parsed.resolve(time).unwrap();
            assert_eq!(parsed.format(instant, "%Y-%m-%d %H:%M"), "2026-01-15 12:00");
        }
    }
}
