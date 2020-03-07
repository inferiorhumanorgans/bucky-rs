use std::ops::Range;

use crate::array::ticks::{TickDuration, TickIncrement};
use crate::error::{Result, ScaleError};
use crate::interpolate::{NumberInterpolator, RangeInterpolator};
use crate::scale::continuous::*;

use chrono::prelude::*;
use date_iterator::{calendar_duration, CalendarDuration};

const RFC_3339_FMT: &str = "%Y-%m-%dT%H:%M:%S";

#[derive(Clone, Debug)]
pub struct ScaleTime<RangeType, InterpolatorType> {
    pub domain: Range<NaiveDateTime>,
    pub range: Range<RangeType>,
    pub clamped: bool,
    pub interpolator: InterpolatorType,
}

impl<'a> ScaleTime<f64, NumberInterpolator> {
    pub fn new() -> Self {
        let epoch_start = NaiveDateTime::parse_from_str("2000-01-01T00:00:00", RFC_3339_FMT)
            .expect("Date parsing failed?");
        let epoch_end = NaiveDateTime::parse_from_str("2000-01-02T00:00:00", RFC_3339_FMT)
            .expect("Date parsing failed?");

        Self {
            domain: epoch_start..epoch_end,
            range: 0.0..1.0,
            clamped: false,
            interpolator: NumberInterpolator::new(),
        }
    }

    pub fn interpolator<NewInterpolator>(
        self,
        interpolator: NewInterpolator,
    ) -> ScaleTime<f64, NewInterpolator>
    where
        NewInterpolator: RangeInterpolator<'a, f64>,
    {
        ScaleTime {
            interpolator,
            domain: self.domain,
            range: self.range,
            clamped: self.clamped,
        }
    }
}

impl<'a, RangeType, InterpolatorType> ScaleContinuous<'a, NaiveDateTime, RangeType>
    for ScaleTime<RangeType, InterpolatorType>
where
    RangeType: std::fmt::Debug,
    InterpolatorType: RangeInterpolator<'a, RangeType>,
{
    fn domain<DomainIntermediateType>(self, domain: Range<DomainIntermediateType>) -> Result<Self>
    where
        DomainIntermediateType: Into<NaiveDateTime> + PartialOrd,
    {
        if domain.start > domain.end {
            return Err(ScaleError::DescendingScale);
        }

        Ok(Self {
            domain: domain.start.into()..domain.end.into(),
            ..self
        })
    }

    fn range<RangeIntermediateType>(self, range: Range<RangeIntermediateType>) -> Result<Self>
    where
        RangeIntermediateType: Into<RangeType>,
    {
        Ok(Self {
            range: range.start.into()..range.end.into(),
            ..self
        })
    }

    fn clamped(self, clamped: bool) -> Self {
        Self { clamped, ..self }
    }

    fn nice<CountType>(self, count: Option<CountType>) -> Result<Self>
    where
        CountType: Into<i32>,
    {
        let count = match count {
            Some(count) => count.into(),
            None => 10,
        };

        match self.domain.tick_increment(count) {
            TickDuration::Milliseconds(0) |
            TickDuration::Seconds(0) => Ok(self),
            increment @ _ => {
                let round_start = increment.floor(&self.domain.start);
                let round_end = increment.ceil(&self.domain.end);

                self.domain(round_start..round_end)
            }
        }
    }

    fn scale<DomainIntermediateType>(&'a self, t: DomainIntermediateType) -> RangeType
    where
        DomainIntermediateType: Into<NaiveDateTime>,
        RangeType: Copy,
    {
        let clamped = match self.clamped {
            true => self.domain.start.max(self.domain.end.min(t.into())),
            false => t.into(),
        };

        // TODO: Proper bounds checking
        let start = (clamped - self.domain.start).num_nanoseconds().unwrap() as f64;
        let end = (self.domain.end - self.domain.start)
            .num_nanoseconds()
            .unwrap() as f64;
        let normalized: f64 = start / end;

        self.interpolator.interpolate_range(&self.range, normalized)
    }

    fn ticks(&self, tick_count: Option<i32>) -> Vec<NaiveDateTime> {
        let tick_count = match tick_count {
            Some(tick_count) => tick_count,
            None => 10,
        };

        let interval : TickDuration = self.domain.tick_increment(tick_count);
        let calendar_duration : CalendarDuration = CalendarDuration::from(&interval);

        let mut ticks = vec![];
        let mut cur = self.domain.start;

        while cur <= self.domain.end {
            ticks.push(cur);
            cur = calendar_duration::naive_checked_add(&cur, &calendar_duration).expect("Date math failed");
        }

        ticks
    }
}

#[test]
fn defaults() -> Result<()> {
    let scale = ScaleTime::new();

    {
        let point = NaiveDateTime::parse_from_str("2000-01-01T00:00:00", RFC_3339_FMT)?;
        assert_eq!(0.0, scale.scale(point));
    }

    {
        let point = NaiveDateTime::parse_from_str("2000-01-01T12:00:00", RFC_3339_FMT)?;
        assert_eq!(0.5, scale.scale(point));
    }

    {
        let point = NaiveDateTime::parse_from_str("2000-01-02T00:00:00", RFC_3339_FMT)?;
        assert_eq!(1.0, scale.scale(point));
    }

    Ok(())
}

#[test]
fn nice_defaults_to_10() -> Result<()> {
    let scale = {
        let d0 = NaiveDateTime::parse_from_str("2000-01-01T00:17:00", RFC_3339_FMT)?;
        let d1 = NaiveDateTime::parse_from_str("2000-01-01T23:42:00", RFC_3339_FMT)?;
        ScaleTime::new().domain(d0..d1)?.nice(None::<i32>)?
    };

    let d0 = NaiveDateTime::parse_from_str("2000-01-01T00:00:00", RFC_3339_FMT)?;
    let d1 = NaiveDateTime::parse_from_str("2000-01-02T00:00:00", RFC_3339_FMT)?;
    assert_eq!(scale.domain.start, d0);
    assert_eq!(scale.domain.end, d1);

    Ok(())
}

#[test]
fn nice_multi_year() -> Result<()> {
    let scale = {
        let d0 = NaiveDateTime::parse_from_str("2001-01-01T00:00:00", RFC_3339_FMT).unwrap();
        let d1 = NaiveDateTime::parse_from_str("2138-01-01T00:42:00", RFC_3339_FMT).unwrap();
        ScaleTime::new().domain(d0..d1)?.nice(None::<i32>)?
    };

    let d0 = NaiveDateTime::parse_from_str("2000-01-01T00:00:00", RFC_3339_FMT).unwrap();
    let d1 = NaiveDateTime::parse_from_str("2140-01-01T00:00:00", RFC_3339_FMT).unwrap();
    assert_eq!(scale.domain.start, d0);
    assert_eq!(scale.domain.end, d1);

    Ok(())
}

#[test]
fn nice_works_on_empty_domain() -> Result<()> {
    let d0 = NaiveDateTime::parse_from_str("2009-01-01T00:12:00", RFC_3339_FMT).unwrap();
    let d1 = NaiveDateTime::parse_from_str("2009-01-01T00:12:00", RFC_3339_FMT).unwrap();

    let scale = ScaleTime::new().domain(d0..d1)?.nice(None::<i32>)?;

    assert_eq!(d0..d1, scale.domain);

    Ok(())
}

#[test]
fn nice_uses_the_specified_tick_count() -> Result<()> {
    let scale = {
        let d0 = NaiveDateTime::parse_from_str("2009-01-01T00:17:00", RFC_3339_FMT).unwrap();
        let d1 = NaiveDateTime::parse_from_str("2009-01-01T23:42:00", RFC_3339_FMT).unwrap();

        ScaleTime::new().domain(d0..d1)?
    };

    {
        let d0 = NaiveDateTime::parse_from_str("2009-01-01T00:15:00", RFC_3339_FMT).unwrap();
        let d1 = NaiveDateTime::parse_from_str("2009-01-01T23:45:00", RFC_3339_FMT).unwrap();
        let domain = scale.clone().nice(Some(100))?.domain;

        assert_eq!(d0..d1, domain);
    }

    {
        let d0 = NaiveDateTime::parse_from_str("2009-01-01T00:00:00", RFC_3339_FMT).unwrap();
        let d1 = NaiveDateTime::parse_from_str("2009-01-02T00:00:00", RFC_3339_FMT).unwrap();
        let domain = scale.clone().nice(Some(10))?.domain;

        assert_eq!(d0..d1, domain);
    }

    Ok(())
}

#[test]
fn nice_empty_domains() -> Result<()> {
    let scale = {
        let d0 = NaiveDateTime::parse_from_str("2009-01-01T00:17:00", RFC_3339_FMT).unwrap();
        let d1 = NaiveDateTime::parse_from_str("2009-01-01T23:42:00", RFC_3339_FMT).unwrap();

        ScaleTime::new().domain(d0..d1)?
    };

    Ok(())
}

#[test]
fn one_second_ticks() -> Result<()> {
    let scale = {
        let d0 = NaiveDateTime::parse_from_str("2011-01-01T12:00:00", RFC_3339_FMT)?;
        let d1 = NaiveDateTime::parse_from_str("2011-01-01T12:00:04", RFC_3339_FMT)?;
        ScaleTime::new().domain(d0..d1)?
    };

    let ticks = scale.ticks(Some(4));

    assert_eq!(
        NaiveDateTime::parse_from_str("2011-01-01T12:00:00", RFC_3339_FMT)?,
        ticks[0]
    );
    assert_eq!(
        NaiveDateTime::parse_from_str("2011-01-01T12:00:01", RFC_3339_FMT)?,
        ticks[1]
    );
    assert_eq!(
        NaiveDateTime::parse_from_str("2011-01-01T12:00:02", RFC_3339_FMT)?,
        ticks[2]
    );
    assert_eq!(
        NaiveDateTime::parse_from_str("2011-01-01T12:00:03", RFC_3339_FMT)?,
        ticks[3]
    );
    assert_eq!(
        NaiveDateTime::parse_from_str("2011-01-01T12:00:04", RFC_3339_FMT)?,
        ticks[4]
    );

    Ok(())
}
