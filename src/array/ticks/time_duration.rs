use std::cmp::Ordering;

use chrono::{Duration, NaiveDateTime};
use date_iterator::{calendar_duration, CalendarDuration};

#[derive(Copy, Clone, Debug)]
pub enum TickDuration {
    Milliseconds(i64),
    Seconds(i64),
    Minutes(i32),
    Hours(i32),
    Days(i32),
    Weeks(i32),
    Months(i32),
    Years(i32),
}

impl From<&TickDuration> for CalendarDuration {
    fn from(tick_duration: &TickDuration) -> CalendarDuration {
        match tick_duration {
            TickDuration::Milliseconds(m) => CalendarDuration::milliseconds(*m),
            TickDuration::Seconds(s) => CalendarDuration::seconds(*s),
            TickDuration::Minutes(m) => CalendarDuration::minutes(*m as i64),
            TickDuration::Hours(h) => CalendarDuration::hours(*h as i64),
            TickDuration::Days(d) => CalendarDuration::days(*d as i64),
            TickDuration::Weeks(w) => CalendarDuration::weeks(*w as i64),
            TickDuration::Months(m) => CalendarDuration::months(*m),
            TickDuration::Years(y) => CalendarDuration::years(*y),
        }
    }
}

impl TickDuration {
    pub fn floor(&self, date_time: &NaiveDateTime) -> NaiveDateTime {
        use crate::chrono::{Datelike, Timelike};
        use std::ops::Sub;

        match self {
            TickDuration::Milliseconds(m) => {
                let remainder = date_time.timestamp_millis() as i64 % m;
                let duration = Duration::milliseconds(remainder);
                date_time.sub(duration).with_nanosecond(0).unwrap()
            },
            TickDuration::Seconds(s) => {
                let remainder = date_time.second() as i64 % s;
                let duration = Duration::seconds(remainder);
                date_time.sub(duration).with_nanosecond(0).unwrap()
            }
            TickDuration::Minutes(m) => {
                let remainder = date_time.minute() as i64 % *m as i64;
                let duration = CalendarDuration::minutes(-remainder);
                calendar_duration::naive_checked_add(&date_time, &duration)
                    .unwrap()
                    .with_second(0)
                    .unwrap()
                    .with_nanosecond(0)
                    .unwrap()
            }
            TickDuration::Hours(h) => {
                let remainder = date_time.hour() as i64 % *h as i64;
                let duration = CalendarDuration::hours(-remainder);
                calendar_duration::naive_checked_add(&date_time, &duration)
                    .unwrap()
                    .with_minute(0)
                    .unwrap()
                    .with_second(0)
                    .unwrap()
                    .with_nanosecond(0)
                    .unwrap()
            }
            TickDuration::Days(d) => {
                let remainder = date_time.ordinal() as i64 % *d as i64;
                let duration = CalendarDuration::days(-remainder);
                calendar_duration::naive_checked_add(&date_time, &duration)
                    .unwrap()
                    .with_hour(0)
                    .unwrap()
                    .with_minute(0)
                    .unwrap()
                    .with_second(0)
                    .unwrap()
                    .with_nanosecond(0)
                    .unwrap()
            }
            TickDuration::Weeks(_w) => unimplemented!(),
            TickDuration::Months(m) => {
                let remainder = date_time.month0() as i32 % m;
                let duration = CalendarDuration::months(-remainder);
                calendar_duration::naive_checked_add(&date_time, &duration)
                    .expect("NaiveCheckedAdd failed")
                    .with_day(1)
                    .expect("with_day failed")
                    .with_minute(0)
                    .expect("with_minute failed")
                    .with_second(0)
                    .expect("with_second failed")
                    .with_nanosecond(0)
                    .expect("with_nanosecond failed")
            }
            TickDuration::Years(y) => {
                let remainder = date_time.year() as i32 % y;
                let duration = CalendarDuration::years(-remainder);
                calendar_duration::naive_checked_add(&date_time, &duration)
                    .expect("NaiveCheckedAdd failed")
                    .with_month0(0)
                    .expect("with_month0 failed")
                    .with_day(1)
                    .expect("with_day failed")
                    .with_hour(0)
                    .expect("with_hour failed")
                    .with_minute(0)
                    .expect("with_minute failed")
                    .with_second(0)
                    .expect("with_second failed")
                    .with_nanosecond(0)
                    .expect("with_nanosecond failed")
            }
        }
    }

    pub fn ceil(&self, date_time: &NaiveDateTime) -> NaiveDateTime {
        calendar_duration::naive_checked_add(&self.floor(date_time), &self.into()).unwrap()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const RFC_3339_FMT: &str = "%Y-%m-%dT%H:%M:%S";

    #[test]
    fn tick_duration_floor_1() {
        {
            let input_date_time =
                NaiveDateTime::parse_from_str("2000-01-01T00:00:08", RFC_3339_FMT).unwrap();
            let output_date_time =
                NaiveDateTime::parse_from_str("2000-01-01T00:00:08", RFC_3339_FMT).unwrap();
            assert_eq!(
                output_date_time,
                TickDuration::Seconds(1).floor(&input_date_time)
            );
        }
    }

    #[test]
    fn tick_duration_floor_2() {
        {
            let input_date_time =
                NaiveDateTime::parse_from_str("2000-01-01T00:00:08", RFC_3339_FMT).unwrap();
            let output_date_time =
                NaiveDateTime::parse_from_str("2000-01-01T00:00:05", RFC_3339_FMT).unwrap();
            assert_eq!(
                output_date_time,
                TickDuration::Seconds(5).floor(&input_date_time)
            );
        }
    }

    #[test]
    fn tick_duration_floor_3() {
        {
            let input_date_time =
                NaiveDateTime::parse_from_str("2000-01-01T00:00:08", RFC_3339_FMT).unwrap();
            let output_date_time =
                NaiveDateTime::parse_from_str("2000-01-01T00:00:00", RFC_3339_FMT).unwrap();
            assert_eq!(
                output_date_time,
                TickDuration::Minutes(1).floor(&input_date_time)
            );
        }
    }

    #[test]
    fn tick_duration_floor_4() {
        {
            let input_date_time =
                NaiveDateTime::parse_from_str("2000-01-01T00:08:08", RFC_3339_FMT).unwrap();
            let output_date_time =
                NaiveDateTime::parse_from_str("2000-01-01T00:05:00", RFC_3339_FMT).unwrap();
            assert_eq!(
                output_date_time,
                TickDuration::Minutes(5).floor(&input_date_time)
            );
        }
    }

    #[test]
    #[ignore]
    fn tick_duration_floor_5() {
        {
            let input_date_time =
                NaiveDateTime::parse_from_str("2000-01-01T00:08:08", RFC_3339_FMT).unwrap();
            let output_date_time =
                NaiveDateTime::parse_from_str("2000-01-01T00:05:00", RFC_3339_FMT).unwrap();
            assert_eq!(
                output_date_time,
                TickDuration::Weeks(5).floor(&input_date_time)
            );
        }
    }

    #[test]
    fn tick_duration_floor_6() {
        {
            let input_date_time =
                NaiveDateTime::parse_from_str("2000-02-24T00:08:08", RFC_3339_FMT).unwrap();
            let output_date_time =
                NaiveDateTime::parse_from_str("2000-01-01T00:00:00", RFC_3339_FMT).unwrap();
            assert_eq!(
                output_date_time,
                TickDuration::Months(3).floor(&input_date_time)
            );
        }
    }

    #[test]
    fn tick_duration_floor_7() {
        {
            let input_date_time =
                NaiveDateTime::parse_from_str("2000-02-24T00:08:08", RFC_3339_FMT).unwrap();
            let output_date_time =
                NaiveDateTime::parse_from_str("2000-02-01T00:00:00", RFC_3339_FMT).unwrap();
            assert_eq!(
                output_date_time,
                TickDuration::Months(1).floor(&input_date_time)
            );
        }
    }

    #[test]
    fn tick_duration_floor_8() {
        {
            let input_date_time =
                NaiveDateTime::parse_from_str("2000-02-24T00:08:08", RFC_3339_FMT).unwrap();
            let output_date_time =
                NaiveDateTime::parse_from_str("2000-02-24T00:00:00", RFC_3339_FMT).unwrap();
            assert_eq!(
                output_date_time,
                TickDuration::Hours(1).floor(&input_date_time)
            );
        }
    }

    #[test]
    fn tick_duration_floor_9() {
        {
            let input_date_time =
                NaiveDateTime::parse_from_str("2000-02-24T04:08:08", RFC_3339_FMT).unwrap();
            let output_date_time =
                NaiveDateTime::parse_from_str("2000-02-24T04:00:00", RFC_3339_FMT).unwrap();
            assert_eq!(
                output_date_time,
                TickDuration::Hours(1).floor(&input_date_time)
            );
        }
    }

    #[test]
    fn tick_duration_floor_10() {
        {
            let input_date_time =
                NaiveDateTime::parse_from_str("2000-02-24T05:08:08", RFC_3339_FMT).unwrap();
            let output_date_time =
                NaiveDateTime::parse_from_str("2000-02-24T03:00:00", RFC_3339_FMT).unwrap();
            assert_eq!(
                output_date_time,
                TickDuration::Hours(3).floor(&input_date_time)
            );
        }
    }
    #[test]
    fn tick_duration_ceil_1() {
        {
            let input_date_time =
                NaiveDateTime::parse_from_str("2000-01-01T00:00:08", RFC_3339_FMT).unwrap();
            let output_date_time =
                NaiveDateTime::parse_from_str("2000-01-01T00:00:09", RFC_3339_FMT).unwrap();
            assert_eq!(
                output_date_time,
                TickDuration::Seconds(1).ceil(&input_date_time)
            );
        }
    }

    #[test]
    fn tick_duration_ceil_2() {
        {
            let input_date_time =
                NaiveDateTime::parse_from_str("2000-01-01T00:00:08", RFC_3339_FMT).unwrap();
            let output_date_time =
                NaiveDateTime::parse_from_str("2000-01-01T00:00:10", RFC_3339_FMT).unwrap();
            assert_eq!(
                output_date_time,
                TickDuration::Seconds(5).ceil(&input_date_time)
            );
        }
    }

    #[test]
    fn tick_duration_ceil_3() {
        {
            let input_date_time =
                NaiveDateTime::parse_from_str("2000-01-01T00:00:08", RFC_3339_FMT).unwrap();
            let output_date_time =
                NaiveDateTime::parse_from_str("2000-01-01T00:01:00", RFC_3339_FMT).unwrap();
            assert_eq!(
                output_date_time,
                TickDuration::Minutes(1).ceil(&input_date_time)
            );
        }
    }

    #[test]
    fn tick_duration_ceil_4() {
        {
            let input_date_time =
                NaiveDateTime::parse_from_str("2000-01-01T00:08:08", RFC_3339_FMT).unwrap();
            let output_date_time =
                NaiveDateTime::parse_from_str("2000-01-01T00:10:00", RFC_3339_FMT).unwrap();
            assert_eq!(
                output_date_time,
                TickDuration::Minutes(5).ceil(&input_date_time)
            );
        }
    }

    #[test]
    #[ignore]
    fn tick_duration_ceil_5() {
        {
            let input_date_time =
                NaiveDateTime::parse_from_str("2000-01-01T00:08:08", RFC_3339_FMT).unwrap();
            let output_date_time =
                NaiveDateTime::parse_from_str("2000-01-01T00:05:00", RFC_3339_FMT).unwrap();
            assert_eq!(
                output_date_time,
                TickDuration::Weeks(5).ceil(&input_date_time)
            );
        }
    }

    #[test]
    fn tick_duration_ceil_6() {
        {
            let input_date_time =
                NaiveDateTime::parse_from_str("2000-02-24T00:08:08", RFC_3339_FMT).unwrap();
            let output_date_time =
                NaiveDateTime::parse_from_str("2000-04-01T00:00:00", RFC_3339_FMT).unwrap();
            assert_eq!(
                output_date_time,
                TickDuration::Months(3).ceil(&input_date_time)
            );
        }
    }

    #[test]
    fn tick_duration_ceil_7() {
        {
            let input_date_time =
                NaiveDateTime::parse_from_str("2000-02-24T00:08:08", RFC_3339_FMT).unwrap();
            let output_date_time =
                NaiveDateTime::parse_from_str("2000-03-01T00:00:00", RFC_3339_FMT).unwrap();
            assert_eq!(
                output_date_time,
                TickDuration::Months(1).ceil(&input_date_time)
            );
        }
    }

    #[test]
    fn tick_duration_ceil_8() {
        {
            let input_date_time =
                NaiveDateTime::parse_from_str("2000-02-24T00:08:08", RFC_3339_FMT).unwrap();
            let output_date_time =
                NaiveDateTime::parse_from_str("2000-02-24T01:00:00", RFC_3339_FMT).unwrap();
            assert_eq!(
                output_date_time,
                TickDuration::Hours(1).ceil(&input_date_time)
            );
        }
    }

    #[test]
    fn tick_duration_ceil_9() {
        {
            let input_date_time =
                NaiveDateTime::parse_from_str("2000-02-24T04:08:08", RFC_3339_FMT).unwrap();
            let output_date_time =
                NaiveDateTime::parse_from_str("2000-02-24T05:00:00", RFC_3339_FMT).unwrap();
            assert_eq!(
                output_date_time,
                TickDuration::Hours(1).ceil(&input_date_time)
            );
        }
    }

    #[test]
    fn tick_duration_ceil_10() {
        {
            let input_date_time =
                NaiveDateTime::parse_from_str("2000-02-24T05:08:08", RFC_3339_FMT).unwrap();
            let output_date_time =
                NaiveDateTime::parse_from_str("2000-02-24T06:00:00", RFC_3339_FMT).unwrap();
            assert_eq!(
                output_date_time,
                TickDuration::Hours(3).ceil(&input_date_time)
            );
        }
    }
}

impl From<&TickDuration> for i64 {
    fn from(duration: &TickDuration) -> Self {
        match duration {
            TickDuration::Milliseconds(s) => *s as i64 / 1000,
            TickDuration::Seconds(s) => *s as i64,
            TickDuration::Minutes(m) => *m as i64 * 60,
            TickDuration::Hours(h) => *h as i64 * 60 * 60,
            TickDuration::Days(d) => *d as i64 * 24 * 60 * 60,
            TickDuration::Weeks(w) => *w as i64 * 7 * 24 * 60 * 60,
            TickDuration::Months(m) => *m as i64 * 30 * 24 * 60 * 60,
            TickDuration::Years(y) => *y as i64 * 365 * 24 * 60 * 60,
        }
    }
}

impl Eq for TickDuration {}

impl Ord for TickDuration {
    fn cmp(&self, other: &Self) -> Ordering {
        let seconds = i64::from(self);
        let other_seconds = i64::from(other);
        seconds.cmp(&other_seconds)
    }
}

impl PartialOrd for TickDuration {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for TickDuration {
    fn eq(&self, other: &Self) -> bool {
        let seconds = i64::from(self);
        let other_seconds = i64::from(other);
        seconds == other_seconds
    }
}
