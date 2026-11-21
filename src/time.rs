use jiff::civil::DateTime;
use jiff::tz::{AmbiguousOffset, TimeZone};
use jiff::Timestamp;

pub fn parse(value: &str) -> Result<DateTime, String> {
    let time = DateTime::strptime("%Y-%m-%d %H:%M", value.trim())
        .map_err(|_| "Invalid time. Please use YYYY-MM-DD HH:MM.".to_string())?;

    if !(1800..2100).contains(&time.year()) {
        return Err("Supported years are 1800–2099.".into());
    }

    Ok(time)
}

pub fn resolve(timezone: &TimeZone, time: DateTime) -> Result<Timestamp, String> {
    match timezone.to_ambiguous_timestamp(time).offset() {
        AmbiguousOffset::Unambiguous { offset } => offset
            .to_timestamp(time)
            .map_err(|_| "This time is outside the supported range.".into()),
        AmbiguousOffset::Gap { .. } => {
            Err("This time does not exist in the source timezone.".into())
        }
        AmbiguousOffset::Fold { before, after } => Err(format!(
            "This time occurs twice in the source timezone (UTC offsets {before} and {after}). Choose an unambiguous time."
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse(value: &str) -> DateTime {
        DateTime::strptime("%Y-%m-%d %H:%M", value).unwrap()
    }

    fn zone(name: &str) -> TimeZone {
        TimeZone::get(name).unwrap()
    }

    #[test]
    fn rejects_daylight_saving_gaps() {
        for (name, value) in [
            ("Europe/London", "2026-03-29 01:30"),
            ("America/New_York", "2026-03-08 02:30"),
        ] {
            let error = resolve(&zone(name), parse(value)).unwrap_err();
            assert!(error.contains("does not exist"));
        }
    }

    #[test]
    fn rejects_ambiguous_times() {
        for (name, value) in [
            ("Europe/London", "2026-10-25 01:30"),
            ("America/New_York", "2026-11-01 01:30"),
        ] {
            let error = resolve(&zone(name), parse(value)).unwrap_err();
            assert!(error.contains("occurs twice"));
        }
    }

    #[test]
    fn resolves_winter_and_summer_offsets() {
        let london = zone("Europe/London");
        for (value, expected) in [
            ("2026-01-15 12:00", "2026-01-15T12:00:00Z"),
            ("2026-07-15 12:00", "2026-07-15T11:00:00Z"),
        ] {
            assert_eq!(
                resolve(&london, parse(value)).unwrap().to_string(),
                expected
            );
        }
    }

    #[test]
    fn rejects_dates_outside_timezone_data() {
        for value in ["1799-12-31 23:59", "2100-01-01 00:00", "2200-07-15 12:00"] {
            assert!(super::parse(value).unwrap_err().contains("1800–2099"));
        }
        assert!(super::parse("1800-01-01 00:00").is_ok());
        assert!(super::parse("2099-12-31 23:59").is_ok());
    }

    #[test]
    fn validates_calendar_dates() {
        for value in ["tomorrow", "2026-02-30 12:00", "2026-01-01 24:00"] {
            assert!(super::parse(value).is_err());
        }
        assert!(super::parse("2024-02-29 00:00").is_ok());
    }
}
