use std::ops::Range;

use chrono::{Duration, NaiveDateTime};
use date_iterator::CalendarDuration;

use crate::array::ticks::{TickDuration, TickIncrement, TickStep};

impl TickIncrement<Range<NaiveDateTime>, TickDuration> for Range<NaiveDateTime> {
    fn tick_increment(&self, count: i32) -> TickDuration {
        let target: Duration = (self.end - self.start) / count;

        let td_duration = TickDuration::Seconds(target.num_seconds());

        let intervals: &[TickDuration] = &[
            TickDuration::Seconds(1),
            TickDuration::Seconds(5),
            TickDuration::Seconds(15),
            TickDuration::Seconds(30),
            TickDuration::Minutes(1),
            TickDuration::Minutes(5),
            TickDuration::Minutes(15),
            TickDuration::Minutes(30),
            TickDuration::Hours(1),
            TickDuration::Hours(3),
            TickDuration::Hours(6),
            TickDuration::Hours(12),
            TickDuration::Days(1),
            TickDuration::Days(2),
            TickDuration::Weeks(1),
            TickDuration::Months(1),
            TickDuration::Months(3),
            TickDuration::Years(1),
        ];

        let search_result = intervals.binary_search(&td_duration);
        let i: usize = match search_result {
            Ok(n) => n,
            Err(n) => n,
        };

        // If we need an interval > 1 year let's just generate a
        // year interval based on the overall length of the range
        if i == intervals.len() {
            let year_width = i64::from(&TickDuration::Years(1));
            let interval_width = self.end.timestamp() - self.start.timestamp();
            let years = (interval_width / year_width) as i32;
            let tick_step = (0..years).tick_step(count);

            TickDuration::Years(tick_step as i32)
        } else if i > 0 {
            // TODO: CHECK FOR OVERFLOW and clean this up
            let i0 = i64::from(&intervals[i - 1]) as i32;
            let i1 = i64::from(&intervals[i]);

            if (target / i0).num_seconds() < (i1 / target.num_seconds()) {
                intervals[i-1]
            } else {
                intervals[i]
            }
        } else {
            let seconds = CalendarDuration::from(&td_duration).duration_part().num_seconds();
            TickDuration::Milliseconds(seconds * 1000)
        }
    }
}
