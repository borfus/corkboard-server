//! Pacific wall-clock time, without hardcoding an offset.
//!
//! Event start and end times are stored as *naive* Pacific wall-clock values --
//! what a person typed, with no offset attached. Comparing "now" against them
//! therefore means asking what the wall clock in Los Angeles reads right now.
//!
//! That used to be written as `Utc::now().naive_utc() - Duration::hours(7)`,
//! which is correct only while Pacific is on daylight time. For roughly four
//! months a year the zone is UTC-8, so every event appeared and expired an hour
//! early all winter. A real timezone database knows which offset applies on a
//! given date; an offset constant never can.
//!
//! Formatting is deliberately absent: the server compares times, the bot
//! displays them, and the "PST"/"PDT" labelling lives over there with the rest
//! of the presentation.

use chrono::{DateTime, NaiveDateTime, Utc};
use chrono_tz::America::Los_Angeles;
use chrono_tz::Tz;

pub const PACIFIC: Tz = Los_Angeles;

/// The current instant, expressed in Pacific.
pub fn now() -> DateTime<Tz> {
    Utc::now().with_timezone(&PACIFIC)
}

/// What a Pacific wall clock reads right now.
///
/// Naive on purpose: this is the value to compare against the naive Pacific
/// timestamps in the database.
pub fn now_naive() -> NaiveDateTime {
    now().naive_local()
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{Duration, NaiveDate, TimeZone};

    fn at(y: i32, m: u32, d: u32, h: u32, min: u32) -> NaiveDateTime {
        NaiveDate::from_ymd_opt(y, m, d)
            .unwrap()
            .and_hms_opt(h, min, 0)
            .unwrap()
    }

    #[test]
    fn summer_is_seven_hours_behind_utc() {
        let utc = Utc.with_ymd_and_hms(2026, 7, 4, 19, 0, 0).unwrap();
        assert_eq!(
            utc.with_timezone(&PACIFIC).naive_local(),
            at(2026, 7, 4, 12, 0)
        );
    }

    /// The exact bug the old `- Duration::hours(7)` had: in winter Pacific is
    /// UTC-8, so a fixed seven hour offset lands an hour late and events flip
    /// over early.
    #[test]
    fn winter_is_eight_hours_behind_utc_not_seven() {
        let utc = Utc.with_ymd_and_hms(2026, 1, 15, 20, 0, 0).unwrap();
        let real = utc.with_timezone(&PACIFIC).naive_local();

        assert_eq!(real, at(2026, 1, 15, 12, 0));
        assert_eq!(
            real,
            utc.naive_utc() - Duration::hours(8),
            "January is UTC-8"
        );
        assert_ne!(
            real,
            utc.naive_utc() - Duration::hours(7),
            "a fixed -7 offset must not agree with real Pacific in January"
        );
    }

    /// Both offsets are reachable across a year, which is the whole point of
    /// asking a timezone database instead of subtracting a constant.
    #[test]
    fn both_offsets_occur_within_one_year() {
        let mut offsets: Vec<i32> = (0..12)
            .map(|month| {
                let utc = Utc.with_ymd_and_hms(2026, month + 1, 15, 20, 0, 0).unwrap();
                let shifted = utc.with_timezone(&PACIFIC).naive_local() - utc.naive_utc();
                shifted.num_hours() as i32
            })
            .collect();

        offsets.sort();
        offsets.dedup();
        assert_eq!(offsets, vec![-8, -7]);
    }

    /// Converts a single instant both ways rather than reading the clock twice.
    /// Two reads are microseconds apart, and `num_hours` truncates toward zero,
    /// so a -7 hour gap measured that way comes back as -6.
    #[test]
    fn now_is_pacific_rather_than_utc() {
        let instant = Utc::now();
        let offset = instant.with_timezone(&PACIFIC).naive_local() - instant.naive_utc();

        assert!(
            offset == Duration::hours(-7) || offset == Duration::hours(-8),
            "Pacific should be 7 or 8 hours behind UTC, was {:?}",
            offset
        );
    }
}
