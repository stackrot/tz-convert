use chrono::{DateTime, LocalResult, NaiveDateTime, Offset, TimeZone, Utc};

pub fn resolve<T: TimeZone>(timezone: &T, time: &NaiveDateTime) -> Result<DateTime<Utc>, String> {
    match timezone.from_local_datetime(time) {
        LocalResult::Single(time) => Ok(time.with_timezone(&Utc)),
        LocalResult::None => Err("This time does not exist in the source timezone.".into()),
        LocalResult::Ambiguous(first, second) => Err(format!(
            "This time occurs twice in the source timezone (UTC offsets {} and {}). Choose an unambiguous time.",
            first.offset().fix(),
            second.offset().fix()
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono_tz::{America::New_York, Europe::London};

    fn parse(value: &str) -> NaiveDateTime {
        NaiveDateTime::parse_from_str(value, "%Y-%m-%d %H:%M").unwrap()
    }

    #[test]
    fn rejects_daylight_saving_gaps() {
        for (zone, value) in [(London, "2026-03-29 01:30"), (New_York, "2026-03-08 02:30")] {
            let error = resolve(&zone, &parse(value)).unwrap_err();
            assert!(error.contains("does not exist"));
        }
    }

    #[test]
    fn rejects_ambiguous_times() {
        for (zone, value) in [(London, "2026-10-25 01:30"), (New_York, "2026-11-01 01:30")] {
            let error = resolve(&zone, &parse(value)).unwrap_err();
            assert!(error.contains("occurs twice"));
        }
    }

    #[test]
    fn resolves_winter_and_summer_offsets() {
        for (value, expected) in [
            ("2026-01-15 12:00", "2026-01-15T12:00:00+00:00"),
            ("2026-07-15 12:00", "2026-07-15T11:00:00+00:00"),
        ] {
            assert_eq!(
                resolve(&London, &parse(value)).unwrap().to_rfc3339(),
                expected
            );
        }
    }
}
