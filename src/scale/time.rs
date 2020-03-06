use std::ops::Range;

use crate::array::ticks::TickIncrement;
use crate::error::{Result, ScaleError};
use crate::scale::continuous::*;
use crate::interpolate::{RangeInterpolator, NumberInterpolator};

use chrono::prelude::*;

const RFC_3339_FMT: &str = "%Y-%m-%dT%H:%M:%S";

#[derive(Clone, Debug)]
pub struct ScaleTime<RangeType, InterpolatorType> {
    pub domain: Range<NaiveDateTime>,
    pub range: Range<RangeType>,
    pub clamped: bool,
    pub interpolator: InterpolatorType,
}

impl ScaleTime<f64, NumberInterpolator> {
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
}

impl<'a, RangeType, InterpolatorType> ScaleContinuous<'a, NaiveDateTime, RangeType> for ScaleTime<RangeType, InterpolatorType>
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

    fn clamped(self, clamped: bool) -> Self
    {
        Self { clamped, ..self }
    }

    fn nice<CountType>(self, count: Option<CountType>) -> Result<Self>
    where
        CountType: Into<i32>,
    {
        assert!(
            self.domain.end > self.domain.start,
            "Reverse not supported yet"
        );

        let count = match count {
            Some(count) => count.into(),
            None => 10,
        };

        let increment = self.domain.tick_increment(count);

        let round_start = increment.floor(&self.domain.start);
        let round_end = increment.ceil(&self.domain.end);

        self.domain(round_start..round_end)
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

    fn ticks(&self, _tick_count: Option<i32>) -> Vec<f64> {
        unimplemented!()
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

// tape("time.ticks(count) can generate 1-second ticks", function(test) {
//     var x = scale.scaleTime().domain([date.local(2011, 0, 1, 12, 0, 0), date.local(2011, 0, 1, 12, 0, 4)]);
//     test.deepEqual(x.ticks(4), [
//       date.local(2011, 0, 1, 12, 0, 0),
//       date.local(2011, 0, 1, 12, 0, 1),
//       date.local(2011, 0, 1, 12, 0, 2),
//       date.local(2011, 0, 1, 12, 0, 3),
//       date.local(2011, 0, 1, 12, 0, 4)
//     ]);
//     test.end();
//   });
#[test]
fn one_second_ticks() -> Result<()> {
    let scale = {
        let d0 = NaiveDateTime::parse_from_str("2011-01-01T12:00:00", RFC_3339_FMT).unwrap();
        let d1 = NaiveDateTime::parse_from_str("2011-01-01T12:00:04", RFC_3339_FMT).unwrap();
        ScaleTime::new().domain(d0..d1)?
    };

    // let ticks = scale.ticks(4);

    unimplemented!();
}
